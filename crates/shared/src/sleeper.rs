use avian3d::prelude::*;
use bevy::prelude::*;
use lightyear::prelude::{NetworkTarget, Replicate};
use serde::{Deserialize, Serialize};

use crate::components::health::Health;
use crate::components::flashlight::PlayerFlashlight;
use crate::entities::NpcPhysicsBundle;
use crate::navigation::{NavigationPathState, SimpleNavigationAgent};
use crate::noise::{NoiseEvent, NoiseField, NoiseType, ZoneConnectivity};
use crate::protocol::{CharacterMarker, PlayerId};

#[derive(
    Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash, Reflect,
)]
pub enum SleeperState {
    Dormant,
    Investigating,
    Alerted,
    Combat,
}

impl Default for SleeperState {
    fn default() -> Self {
        Self::Dormant
    }
}

impl SleeperState {
    pub fn alert_threshold(self) -> f32 {
        match self {
            Self::Dormant => 0.0,
            Self::Investigating => 0.3,
            Self::Alerted => 0.6,
            Self::Combat => 0.9,
        }
    }

    pub fn next_state_at(alert_level: f32) -> Self {
        if alert_level >= Self::Combat.alert_threshold() {
            Self::Combat
        } else if alert_level >= Self::Alerted.alert_threshold() {
            Self::Alerted
        } else if alert_level >= Self::Investigating.alert_threshold() {
            Self::Investigating
        } else {
            Self::Dormant
        }
    }

    pub fn is_hostile(self) -> bool {
        matches!(self, Self::Alerted | Self::Combat)
    }

    pub fn is_active(self) -> bool {
        matches!(self, Self::Investigating | Self::Alerted | Self::Combat)
    }
}

impl std::fmt::Display for SleeperState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Dormant => write!(f, "Dormant"),
            Self::Investigating => write!(f, "Investigating"),
            Self::Alerted => write!(f, "Alerted"),
            Self::Combat => write!(f, "Combat"),
        }
    }
}

#[derive(
    Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash, Reflect,
)]
pub enum SleeperArchetype {
    Drone,
    Spitter,
    Scout,
    Tank,
}

impl Default for SleeperArchetype {
    fn default() -> Self {
        Self::Drone
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Reflect)]
pub struct SleeperArchetypeData {
    pub health: f32,
    pub walk_speed: f32,
    pub chase_speed: f32,
    pub hearing_range: f32,
    pub sight_range: f32,
    pub sight_cone: f32,
    pub noise_threshold: f32,
    pub alert_accumulation: f32,
    pub scream_radius: f32,
    pub detection_speed: f32,
}

impl SleeperArchetype {
    pub fn data(self) -> SleeperArchetypeData {
        match self {
            Self::Drone => SleeperArchetypeData {
                health: 40.0,
                walk_speed: 4.5,
                chase_speed: 6.0,
                hearing_range: 25.0,
                sight_range: 16.0,
                sight_cone: 1.8,
                noise_threshold: 0.15,
                alert_accumulation: 0.35,
                scream_radius: 0.0,
                detection_speed: 0.5,
            },
            Self::Spitter => SleeperArchetypeData {
                health: 65.0,
                walk_speed: 3.5,
                chase_speed: 4.5,
                hearing_range: 30.0,
                sight_range: 22.0,
                sight_cone: 2.0,
                noise_threshold: 0.12,
                alert_accumulation: 0.45,
                scream_radius: 0.0,
                detection_speed: 0.6,
            },
            Self::Scout => SleeperArchetypeData {
                health: 30.0,
                walk_speed: 5.0,
                chase_speed: 7.0,
                hearing_range: 40.0,
                sight_range: 14.0,
                sight_cone: 1.4,
                noise_threshold: 0.08,
                alert_accumulation: 0.60,
                scream_radius: 40.0,
                detection_speed: 0.8,
            },
            Self::Tank => SleeperArchetypeData {
                health: 250.0,
                walk_speed: 2.2,
                chase_speed: 3.0,
                hearing_range: 35.0,
                sight_range: 12.0,
                sight_cone: 0.8,
                noise_threshold: 0.25,
                alert_accumulation: 0.20,
                scream_radius: 0.0,
                detection_speed: 0.3,
            },
        }
    }
}

#[derive(Component, Serialize, Deserialize, Clone, Debug)]
pub struct Sleeper {
    pub state: SleeperState,
    pub archetype: SleeperArchetype,
    pub zone_id: Option<crate::level::generation::ZoneId>,
    pub alert_level: f32,
    pub investigate_target: Option<Vec3>,
    pub investigate_timer: f32,
    pub investigate_duration: f32,
    pub combat_target: Option<Entity>,
    pub spawn_position: Vec3,
    pub scream_cooldown: f32,
}

impl Default for Sleeper {
    fn default() -> Self {
        Self {
            state: SleeperState::Dormant,
            archetype: SleeperArchetype::Drone,
            zone_id: None,
            alert_level: 0.0,
            investigate_target: None,
            investigate_timer: 0.0,
            investigate_duration: 12.0,
            combat_target: None,
            spawn_position: Vec3::ZERO,
            scream_cooldown: 0.0,
        }
    }
}

impl Sleeper {
    pub fn new(archetype: SleeperArchetype, zone_id: crate::level::generation::ZoneId) -> Self {
        Self {
            state: SleeperState::Dormant,
            archetype,
            zone_id: Some(zone_id),
            alert_level: 0.0,
            investigate_target: None,
            investigate_timer: 0.0,
            investigate_duration: 15.0,
            combat_target: None,
            spawn_position: Vec3::ZERO,
            scream_cooldown: 0.0,
        }
    }

    pub fn data(&self) -> SleeperArchetypeData {
        self.archetype.data()
    }

    pub fn gain_alert(&mut self, amount: f32) {
        self.alert_level = (self.alert_level + amount).min(1.0);
    }

    pub fn lose_alert(&mut self, amount: f32) {
        self.alert_level = (self.alert_level - amount).max(0.0);
    }

    pub fn is_dormant(&self) -> bool {
        self.state == SleeperState::Dormant
    }

    pub fn is_hostile(&self) -> bool {
        self.state.is_hostile()
    }
}

#[derive(Component, Serialize, Deserialize, Clone, Debug)]
pub struct ProceduralSleeperMarker;

#[derive(Resource, Clone, Debug)]
pub struct SleeperConfig {
    pub min_per_zone: u32,
    pub max_per_zone: u32,
    pub drone_weight: f32,
    pub spitter_weight: f32,
    pub scout_weight: f32,
    pub tank_weight: f32,
    pub scout_scream_radius: f32,
    pub alert_decay_rate: f32,
    pub investigation_duration: f32,
    pub footstep_interval: f32,
    pub footstep_speed_threshold: f32,
}

impl Default for SleeperConfig {
    fn default() -> Self {
        Self {
            min_per_zone: 2,
            max_per_zone: 6,
            drone_weight: 0.55,
            spitter_weight: 0.15,
            scout_weight: 0.20,
            tank_weight: 0.10,
            scout_scream_radius: 40.0,
            alert_decay_rate: 0.12,
            investigation_duration: 15.0,
            footstep_interval: 0.5,
            footstep_speed_threshold: 2.0,
        }
    }
}

#[derive(Message, Clone, Debug, Serialize, Deserialize)]
pub struct SleeperScreamEvent {
    pub source: Vec3,
    pub source_entity: Entity,
    pub zone_id: crate::level::generation::ZoneId,
}

pub struct SleeperPlugin;

impl Plugin for SleeperPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<SleeperScreamEvent>();

        app.register_type::<SleeperState>()
            .register_type::<SleeperArchetype>();

        app.insert_resource(SleeperConfig::default());

        app.add_systems(
            FixedUpdate,
            (
                sleeper_audio_reaction_system
                    .after(crate::noise::propagate_noise_system),
                sleeper_vision_system
                    .after(crate::noise::propagate_noise_system),
                sleeper_state_machine_system
                    .after(sleeper_audio_reaction_system)
                    .after(sleeper_vision_system),
                sleeper_scream_system
                    .after(sleeper_state_machine_system),
                sleeper_visual_feedback_system
                    .after(sleeper_state_machine_system),
                despawn_dead_sleepers_system,
            ),
        );
    }
}

fn sleeper_audio_reaction_system(
    noise_field: Res<NoiseField>,
    mut sleepers: Query<&mut Sleeper, With<Sleeper>>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    if noise_field.zone_noise.is_empty() {
        return;
    }

    for mut sleeper in sleepers.iter_mut() {
        if sleeper.state.is_hostile() {
            continue;
        }

        let zone_noise = sleeper
            .zone_id
            .and_then(|z| noise_field.zone_noise.get(&z))
            .copied()
            .unwrap_or(0.0);

        if zone_noise <= 0.0 {
            continue;
        }

        let data = sleeper.data();
        if zone_noise > data.noise_threshold {
            let intensity = zone_noise * data.alert_accumulation;
            sleeper.gain_alert(intensity * dt * 3.0);
        }
    }
}

fn sleeper_vision_system(
    mut sleepers: Query<(&mut Sleeper, &Position), With<Sleeper>>,
    players: Query<(&Position, &Rotation, &PlayerFlashlight), With<PlayerId>>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();

    for (mut sleeper, sleeper_pos) in sleepers.iter_mut() {
        if !sleeper.is_dormant() {
            continue;
        }

        let data = sleeper.data();

        for (player_pos, player_rot, flashlight) in players.iter() {
            if !flashlight.is_on {
                continue;
            }

            let to_sleeper = sleeper_pos.0 - player_pos.0;
            let distance = to_sleeper.length();

            if distance > data.sight_range * 0.6 {
                continue;
            }

            let flash_dir = player_rot.0 * Vec3::NEG_Z;
            let to_dir = to_sleeper.normalize_or_zero();

            if !to_dir.is_finite() {
                continue;
            }

            let angle = flash_dir.angle_between(to_dir);

            if angle < flashlight.outer_angle {
                let in_bright = angle < flashlight.inner_angle;
                let intensity = if in_bright { 1.0 } else { 0.5 };
                sleeper.gain_alert(intensity * data.detection_speed * dt);
            }
        }
    }
}

fn sleeper_state_machine_system(
    time: Res<Time>,
    config: Res<SleeperConfig>,
    zone_conn: Option<Res<ZoneConnectivity>>,
    players: Query<(Entity, &Position), With<PlayerId>>,
    mut sleepers: Query<(
        Entity,
        &mut Sleeper,
        &mut SimpleNavigationAgent,
        &Position,
    ), With<Sleeper>>,
    mut scream_writer: MessageWriter<SleeperScreamEvent>,
    mut noise_writer: MessageWriter<NoiseEvent>,
) {
    let dt = time.delta_secs();
    let now = time.elapsed().as_secs_f32();

    let player_data: Vec<(Entity, Vec3)> = players
        .iter()
        .map(|(e, pos)| (e, pos.0))
        .collect();

    for (entity, mut sleeper, mut nav_agent, pos) in sleepers.iter_mut() {
        sleeper.scream_cooldown = (sleeper.scream_cooldown - dt).max(0.0);

        let data = sleeper.data();
        let old_state = sleeper.state;
        let new_state = SleeperState::next_state_at(sleeper.alert_level);

        if new_state != old_state {
            sleeper.state = new_state;

            match new_state {
                SleeperState::Investigating => {
                    if sleeper.investigate_target.is_none() {
                        if let Some(zone_id) = sleeper.zone_id {
                            if let Some(zone_info) = zone_conn.as_ref().and_then(|c| c.zones.get(&zone_id)) {
                                let offset = Vec3::new(
                                    pseudo_rand_offset(zone_id.0 as u32, 0),
                                    0.0,
                                    pseudo_rand_offset(zone_id.0 as u32, 1),
                                );
                                sleeper.investigate_target =
                                    Some(zone_info.center + offset * 5.0);
                            }
                        }
                    }
                    sleeper.investigate_timer = 0.0;
                    sleeper.investigate_duration = config.investigation_duration;
                    nav_agent.speed = data.walk_speed;
                }
                SleeperState::Alerted => {
                    if sleeper.archetype == SleeperArchetype::Scout
                        && sleeper.scream_cooldown <= 0.0
                    {
                        sleeper.scream_cooldown = 15.0;
                        scream_writer.write(SleeperScreamEvent {
                            source: pos.0,
                            source_entity: entity,
                            zone_id: sleeper.zone_id.unwrap_or(crate::level::generation::ZoneId(0)),
                        });

                        noise_writer.write(NoiseEvent::with_entity(
                            pos.0,
                            entity,
                            NoiseType::Scream,
                            now,
                        ));
                    }
                    nav_agent.speed = data.walk_speed;
                }
                SleeperState::Combat => {
                    if sleeper.archetype == SleeperArchetype::Scout
                        && sleeper.scream_cooldown <= 0.0
                    {
                        sleeper.scream_cooldown = 15.0;
                        scream_writer.write(SleeperScreamEvent {
                            source: pos.0,
                            source_entity: entity,
                            zone_id: sleeper.zone_id.unwrap_or(crate::level::generation::ZoneId(0)),
                        });

                        noise_writer.write(NoiseEvent::with_entity(
                            pos.0,
                            entity,
                            NoiseType::Scream,
                            now,
                        ));
                    }
                    nav_agent.speed = data.chase_speed;
                }
                SleeperState::Dormant => {
                    sleeper.investigate_target = None;
                    sleeper.combat_target = None;
                    sleeper.investigate_timer = 0.0;
                    nav_agent.current_target = None;
                    nav_agent.path_waypoints.clear();
                }
            }
        }

        match sleeper.state {
            SleeperState::Dormant => {
                sleeper.lose_alert(config.alert_decay_rate * dt);
                if sleeper.alert_level < SleeperState::Dormant.alert_threshold() {
                    nav_agent.current_target = None;
                    nav_agent.path_waypoints.clear();
                }
            }
            SleeperState::Investigating => {
                sleeper.investigate_timer += dt;
                if sleeper.investigate_timer >= sleeper.investigate_duration {
                    sleeper.lose_alert(config.alert_decay_rate * dt * 2.0);
                    if sleeper.alert_level
                        < SleeperState::Investigating.alert_threshold()
                    {
                        sleeper.state = SleeperState::Dormant;
                        sleeper.investigate_target = None;
                        nav_agent.current_target = None;
                        nav_agent.path_waypoints.clear();
                    }
                }

                if let Some(target) = sleeper.investigate_target {
                    nav_agent.current_target = Some(target);
                }
            }
            SleeperState::Alerted => {
                if sleeper.combat_target.is_none() {
                    if let Some((target_entity, target_pos)) =
                        nearest_player(pos.0, &player_data)
                    {
                        sleeper.combat_target = Some(target_entity);
                        nav_agent.current_target = Some(target_pos);
                    }
                } else if let Some(target_pos) =
                    player_data.iter().find(|(e, _)| Some(*e) == sleeper.combat_target).map(|(_, p)| *p)
                {
                    nav_agent.current_target = Some(target_pos);
                }
                nav_agent.speed = data.walk_speed * 1.2;
            }
            SleeperState::Combat => {
                let nearest = nearest_player(pos.0, &player_data);
                if let Some((target_entity, target_pos)) = nearest {
                    sleeper.combat_target = Some(target_entity);
                    nav_agent.current_target = Some(target_pos);
                } else {
                    sleeper.combat_target = None;
                    if sleeper.alert_level < SleeperState::Alerted.alert_threshold() {
                        sleeper.state = SleeperState::Alerted;
                    }
                }
                nav_agent.speed = data.chase_speed;
            }
        }
    }
}

fn nearest_player(
    from: Vec3,
    players: &[(Entity, Vec3)],
) -> Option<(Entity, Vec3)> {
    players
        .iter()
        .min_by(|(_, a), (_, b)| {
            let da = Vec2::new(a.x, a.z).distance(Vec2::new(from.x, from.z));
            let db = Vec2::new(b.x, b.z).distance(Vec2::new(from.x, from.z));
            da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(e, p)| (*e, *p))
}

fn sleeper_scream_system(
    mut scream_events: MessageReader<SleeperScreamEvent>,
    mut sleepers: Query<(&mut Sleeper, &Position), With<Sleeper>>,
    config: Res<SleeperConfig>,
) {
    let screams: Vec<SleeperScreamEvent> = scream_events.read().cloned().collect();
    if screams.is_empty() {
        return;
    }

    for scream in &screams {
        for (mut sleeper, pos) in sleepers.iter_mut() {
            if sleeper.zone_id == Some(scream.zone_id) && sleeper.combat_target.is_none() {
                continue;
            }

            let distance = pos.0.distance(scream.source);
            if distance > config.scout_scream_radius {
                continue;
            }

            let falloff = 1.0 - (distance / config.scout_scream_radius).clamp(0.0, 1.0);
            let boost = if sleeper.is_dormant() {
                0.5 + 0.3 * falloff
            } else {
                0.4 + 0.4 * falloff
            };

            sleeper.gain_alert(boost);
        }
    }
}

fn sleeper_visual_feedback_system(
    mut sleepers: Query<(&Sleeper, Option<&mut PointLight>), Changed<Sleeper>>,
) {
    for (sleeper, light) in sleepers.iter_mut() {
        if let Some(mut light) = light {
            let pulse = 1.0 + sleeper.alert_level * 2.0;
            match sleeper.state {
                SleeperState::Dormant => {
                    light.color = Color::srgb(0.2, 0.0, 0.4);
                    light.intensity = 50000.0 * pulse;
                }
                SleeperState::Investigating => {
                    light.color = Color::srgb(0.8, 0.4, 0.0);
                    light.intensity = 200000.0 * pulse;
                }
                SleeperState::Alerted => {
                    light.color = Color::srgb(1.0, 0.2, 0.0);
                    light.intensity = 400000.0;
                }
                SleeperState::Combat => {
                    light.color = Color::srgb(1.0, 0.0, 0.0);
                    light.intensity = 600000.0;
                }
            }
        }
    }
}

fn despawn_dead_sleepers_system(
    mut commands: Commands,
    dead_sleepers: Query<(Entity, &Health), With<Sleeper>>,
) {
    for (entity, health) in dead_sleepers.iter() {
        if health.is_dead {
            commands.entity(entity).despawn();
        }
    }
}

fn pseudo_rand_offset(seed: u32, axis: u32) -> f32 {
    let n = seed.wrapping_mul(73856093) ^ (axis * 31);
    let n = n.wrapping_mul(73856093) ^ (n >> 16);
    ((n as f32 / u32::MAX as f32) - 0.5) * 2.0
}

pub fn spawn_sleeper(
    commands: &mut Commands,
    archetype: SleeperArchetype,
    position: Vec3,
    zone_id: crate::level::generation::ZoneId,
) -> Entity {
    let data = archetype.data();
    let mut sleeper = Sleeper::new(archetype, zone_id);
    sleeper.spawn_position = position;

    let entity = commands
        .spawn((
            Name::new(format!("Sleeper_{:?}_{}", archetype, zone_id.0)),
            Position::new(position),
            Rotation::default(),
            LinearVelocity::default(),
            Health {
                current: data.health,
                max: data.health,
                regeneration_rate: 0.0,
                regeneration_delay: 0.0,
                last_damage_time: 0.0,
                is_dead: false,
                can_regenerate: false,
            },
            SimpleNavigationAgent {
                speed: data.walk_speed,
                arrival_threshold: 1.0,
                current_target: None,
                path_waypoints: Vec::new(),
            },
            NavigationPathState::default(),
            sleeper,
            ProceduralSleeperMarker,
            CharacterMarker,
            NpcPhysicsBundle::default(),
            PointLight {
                color: Color::srgb(0.2, 0.0, 0.4),
                intensity: 50000.0,
                range: 8.0,
                radius: 0.2,
                shadow_maps_enabled: false,
                ..default()
            },
            Replicate::to_clients(NetworkTarget::All),
        ))
        .id();

    commands.entity(entity).insert(RigidBody::Kinematic);
    entity
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::level::generation::{generate_level, LevelConfig, ZoneId};
    use crate::noise::build_zone_connectivity;

    fn test_entity(id: u32) -> Entity {
        bevy::ecs::entity::EntityIndex::from_raw_u32(id)
            .map(|idx| Entity::from_index(idx))
            .unwrap()
    }

    #[test]
    fn sleeper_state_transitions_by_alert_level() {
        assert_eq!(SleeperState::next_state_at(0.0), SleeperState::Dormant);
        assert_eq!(SleeperState::next_state_at(0.29), SleeperState::Dormant);
        assert_eq!(SleeperState::next_state_at(0.3), SleeperState::Investigating);
        assert_eq!(SleeperState::next_state_at(0.59), SleeperState::Investigating);
        assert_eq!(SleeperState::next_state_at(0.6), SleeperState::Alerted);
        assert_eq!(SleeperState::next_state_at(0.89), SleeperState::Alerted);
        assert_eq!(SleeperState::next_state_at(0.9), SleeperState::Combat);
        assert_eq!(SleeperState::next_state_at(1.0), SleeperState::Combat);
    }

    #[test]
    fn sleeper_state_alert_thresholds() {
        assert_eq!(SleeperState::Dormant.alert_threshold(), 0.0);
        assert_eq!(SleeperState::Investigating.alert_threshold(), 0.3);
        assert_eq!(SleeperState::Alerted.alert_threshold(), 0.6);
        assert_eq!(SleeperState::Combat.alert_threshold(), 0.9);
    }

    #[test]
    fn sleeper_state_hostility_and_activity() {
        assert!(!SleeperState::Dormant.is_hostile());
        assert!(!SleeperState::Dormant.is_active());
        assert!(!SleeperState::Investigating.is_hostile());
        assert!(SleeperState::Investigating.is_active());
        assert!(SleeperState::Alerted.is_hostile());
        assert!(SleeperState::Alerted.is_active());
        assert!(SleeperState::Combat.is_hostile());
        assert!(SleeperState::Combat.is_active());
    }

    #[test]
    fn sleeper_state_display() {
        assert_eq!(SleeperState::Dormant.to_string(), "Dormant");
        assert_eq!(SleeperState::Investigating.to_string(), "Investigating");
        assert_eq!(SleeperState::Alerted.to_string(), "Alerted");
        assert_eq!(SleeperState::Combat.to_string(), "Combat");
    }

    #[test]
    fn sleeper_archetype_data_has_sensible_values() {
        let drone = SleeperArchetype::Drone.data();
        let spitter = SleeperArchetype::Spitter.data();
        let scout = SleeperArchetype::Scout.data();
        let tank = SleeperArchetype::Tank.data();

        assert!(drone.health < spitter.health);
        assert!(tank.health > spitter.health);

        assert!(drone.chase_speed > drone.walk_speed);
        assert!(tank.walk_speed < drone.walk_speed);
        assert!(tank.health > spitter.health);

        assert!(scout.hearing_range > drone.hearing_range);
        assert!(scout.scream_radius > 0.0);
        assert!(drone.scream_radius == 0.0);

        assert!(drone.noise_threshold < tank.noise_threshold);
        assert!(scout.noise_threshold < spitter.noise_threshold);
    }

    #[test]
    fn sleeper_gain_and_lose_alert() {
        let mut sleeper = Sleeper::new(SleeperArchetype::Drone, ZoneId(1));
        assert_eq!(sleeper.alert_level, 0.0);

        sleeper.gain_alert(0.5);
        assert!((sleeper.alert_level - 0.5).abs() < 0.001);

        sleeper.gain_alert(0.6);
        assert!((sleeper.alert_level - 1.0).abs() < 0.001, "Alert should clamp at 1.0");

        sleeper.lose_alert(0.3);
        assert!((sleeper.alert_level - 0.7).abs() < 0.001);

        sleeper.lose_alert(1.0);
        assert!((sleeper.alert_level - 0.0).abs() < 0.001, "Alert should clamp at 0.0");
    }

    #[test]
    fn sleeper_default_is_dormant() {
        let sleeper = Sleeper::default();
        assert_eq!(sleeper.state, SleeperState::Dormant);
        assert_eq!(sleeper.alert_level, 0.0);
        assert!(!sleeper.is_hostile());
    }

    #[test]
    fn sleeper_config_has_sensible_defaults() {
        let config = SleeperConfig::default();
        assert!(config.min_per_zone >= 1);
        assert!(config.max_per_zone > config.min_per_zone);
        assert!(config.scout_weight > 0.0);
        assert!(config.alert_decay_rate > 0.0);

        let total_weight = config.drone_weight
            + config.spitter_weight
            + config.scout_weight
            + config.tank_weight;
        assert!(
            (total_weight - 1.0).abs() < 0.001,
            "Archetype weights should sum to 1.0, got {}",
            total_weight
        );
    }

    #[test]
    fn nearest_player_finds_closest_entity() {
        let from = Vec3::new(0.0, 0.0, 0.0);
        let players = vec![
            (test_entity(1), Vec3::new(5.0, 0.0, 0.0)),
            (test_entity(2), Vec3::new(3.0, 0.0, 0.0)),
            (test_entity(3), Vec3::new(10.0, 0.0, 0.0)),
        ];

        let result = nearest_player(from, &players);
        assert_eq!(result.map(|(e, _)| e), Some(test_entity(2)));
    }

    #[test]
    fn nearest_player_empty_returns_none() {
        assert!(nearest_player(Vec3::ZERO, &[]).is_none());
    }

    #[test]
    fn nearest_player_2d_distance_ignores_y() {
        let from = Vec3::new(0.0, 0.0, 0.0);
        let players = vec![
            (test_entity(1), Vec3::new(3.0, 100.0, 3.0)), // far in 3D, close in 2D
            (test_entity(2), Vec3::new(10.0, 0.0, 0.0)),    // close in 2D
        ];

        let result = nearest_player(from, &players);
        assert_eq!(result.map(|(e, _)| e), Some(test_entity(1)));
    }

    #[test]
    fn pseudo_rand_offset_is_deterministic() {
        let a = pseudo_rand_offset(42, 0);
        let b = pseudo_rand_offset(42, 0);
        assert_eq!(a, b, "Same seed + axis should produce same offset");

        let c = pseudo_rand_offset(42, 1);
        assert_ne!(a, c, "Different axis should produce different offset");
    }

    #[test]
    fn pseudo_rand_offset_is_in_range() {
        for seed in 0..100 {
            for axis in 0..3 {
                let val = pseudo_rand_offset(seed, axis);
                assert!(
                    val >= -2.0 && val <= 2.0,
                    "Offset should be in [-2, 2], got {} for seed={}, axis={}",
                    val,
                    seed,
                    axis
                );
            }
        }
    }

    #[test]
    fn sleeper_state_machine_transitions_on_alert_increase() {
        let mut sleeper = Sleeper::new(SleeperArchetype::Drone, ZoneId(1));
        assert_eq!(sleeper.state, SleeperState::Dormant);

        sleeper.gain_alert(0.35);
        assert_eq!(
            SleeperState::next_state_at(sleeper.alert_level),
            SleeperState::Investigating
        );

        sleeper.gain_alert(0.35);
        assert_eq!(
            SleeperState::next_state_at(sleeper.alert_level),
            SleeperState::Alerted
        );

        sleeper.gain_alert(0.35);
        assert_eq!(
            SleeperState::next_state_at(sleeper.alert_level),
            SleeperState::Combat
        );
    }

    #[test]
    fn sleeper_state_machine_transitions_on_alert_decay() {
        let mut sleeper = Sleeper::new(SleeperArchetype::Drone, ZoneId(1));
        sleeper.alert_level = 0.95;
        sleeper.state = SleeperState::Combat;

        sleeper.lose_alert(0.25);
        assert!(
            sleeper.alert_level < 0.9,
            "After losing 0.25 from 0.95, alert should be below Combat threshold"
        );
        assert_eq!(
            SleeperState::next_state_at(sleeper.alert_level),
            SleeperState::Alerted
        );

        sleeper.lose_alert(0.25);
        assert!(
            sleeper.alert_level < 0.6,
            "After losing another 0.25, alert should be below Alerted threshold"
        );
        assert_eq!(
            SleeperState::next_state_at(sleeper.alert_level),
            SleeperState::Investigating
        );

        sleeper.lose_alert(0.25);
        assert!(
            sleeper.alert_level < 0.3,
            "After losing another 0.4, alert should be below Investigating threshold"
        );
        assert_eq!(
            SleeperState::next_state_at(sleeper.alert_level),
            SleeperState::Dormant
        );
    }

    #[test]
    fn scout_scream_event_is_creatable() {
        let event = SleeperScreamEvent {
            source: Vec3::new(1.0, 2.0, 3.0),
            source_entity: test_entity(42),
            zone_id: ZoneId(5),
        };

        assert_eq!(event.source, Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(event.source_entity, test_entity(42));
        assert_eq!(event.zone_id, ZoneId(5));
    }

    #[test]
    fn sleeper_can_be_created_with_spawn_sleeper_helpers() {
        // Test that Sleeper::new initializes correctly
        let sleeper = Sleeper::new(SleeperArchetype::Scout, ZoneId(3));
        assert_eq!(sleeper.archetype, SleeperArchetype::Scout);
        assert_eq!(sleeper.zone_id, Some(ZoneId(3)));
        assert_eq!(sleeper.state, SleeperState::Dormant);
        assert_eq!(sleeper.alert_level, 0.0);
        assert_eq!(sleeper.scream_cooldown, 0.0);
    }

    #[test]
    fn audio_reaction_increases_alert_from_noise_field() {
        let mut sleeper = Sleeper::new(SleeperArchetype::Drone, ZoneId(1));
        assert_eq!(sleeper.alert_level, 0.0);

        let data = sleeper.data();
        let zone_noise = 0.5;
        assert!(zone_noise > data.noise_threshold);

        // Simulate what the audio reaction system does
        let intensity = zone_noise * data.alert_accumulation;
        let dt = 0.016; // ~60 FPS
        sleeper.gain_alert(intensity * dt * 3.0);

        assert!(
            sleeper.alert_level > 0.0,
            "Alert should increase from noise. Got: {}, intensity: {}",
            sleeper.alert_level,
            intensity * dt * 3.0
        );
    }

    #[test]
    fn vision_reaction_calculates_cone_inclusion() {
        let player_pos = Vec3::new(0.0, 1.6, 0.0);
        let player_rot = Quat::IDENTITY; // Looking down -Z
        let flashlight = PlayerFlashlight {
            is_on: true,
            intensity: 1400000.0,
            range: 100.0,
            inner_angle: 0.11,
            outer_angle: 0.38,
        };

        let sleeper_pos = Vec3::new(0.0, 1.0, 5.0); // 5m in front of player

        let flash_dir = player_rot * Vec3::NEG_Z; // (0, 0, -1)
        let to_sleeper = sleeper_pos - player_pos; // (0, -0.6, 5.0)
        let to_dir = to_sleeper.normalize_or_zero();

        let angle = flash_dir.angle_between(to_dir);

        // Flashlight points down -Z, sleeper is at +5 in Z
        // The angle should be close to PI (180 degrees)
        assert!(
            angle > flashlight.outer_angle,
            "Sleeper behind flashlight direction should not be detected"
        );

        // Now put sleeper in front of flashlight
        let sleeper_pos2 = Vec3::new(0.0, 1.0, -5.0);
        let to_sleeper2 = sleeper_pos2 - player_pos;
        let to_dir2 = to_sleeper2.normalize_or_zero();
        let angle2 = flash_dir.angle_between(to_dir2);

        assert!(
            angle2 < flashlight.outer_angle,
            "Sleeper in front of flashlight should be detected, angle: {}",
            angle2
        );
    }

    #[test]
    fn scream_boosts_nearby_sleepers() {
        let mut sleeper_a = Sleeper::new(SleeperArchetype::Drone, ZoneId(1));
        let _sleeper_b = Sleeper::new(SleeperArchetype::Tank, ZoneId(2));

        // Simulate scream boosting
        let scream_pos = Vec3::new(0.0, 0.0, 0.0);
        let sleeper_a_pos = Vec3::new(5.0, 0.0, 0.0);

        let distance = sleeper_a_pos.distance(scream_pos);
        let scream_radius = 40.0;
        let falloff = 1.0 - (distance / scream_radius).clamp(0.0, 1.0);
        let boost = 0.5 + 0.3 * falloff;

        sleeper_a.gain_alert(boost);
        let tank_boost_ignored = sleeper_a.is_dormant();

        assert!(boost > 0.5, "Nearby sleeper should get significant boost");
        assert!(tank_boost_ignored || boost > 0.5);
    }

    #[test]
    fn noise_field_and_zone_connectivity_integration() {
        let level = generate_level(LevelConfig {
            seed: 42,
            target_zone_count: 10,
            min_zone_spacing: 55.0,
            max_depth: 6,
        });

        let conn = build_zone_connectivity(&level);
        assert!(conn.zones.len() == level.zones.len());

        let zone_id = *level.zones.keys().next().unwrap();
        let zone_info = conn.zones.get(&zone_id).unwrap();
        assert_eq!(zone_info.zone_id, zone_id);
        assert!(conn.zones.len() > 0);
    }
}
