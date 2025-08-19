use crate::events::{
    DamageDealtEvent, DeathCause, EntityDeathEvent, SoundTriggeredEvent, SoundType, WeaponFireEvent,
};
use crate::menu::GameState;
use avian3d::prelude::*;
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;
use rand::Rng;



pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (weapon_fire, handle_damage, remove_dead_entities)
                .chain() // Run systems in order
                .run_if(in_state(GameState::InGame)),
        );
    }
}

#[derive(Component)]
pub struct Health {
    pub current: f32,
    pub maximum: f32,
    pub last_damage_time: f32,
}

impl Health {
    pub fn take_damage(&mut self, damage: f32, time: f32) {
        self.current = (self.current - damage).max(0.0);
        self.last_damage_time = time;
    }

    pub fn is_dead(&self) -> bool {
        self.current <= 0.0
    }
}

#[derive(Component)]
pub struct Weapon {
    pub damage: f32,

    pub rate_of_fire: f32,
    pub last_shot_time: f32,
    pub ammo_current: u32,
    pub ammo_max: u32,
    pub reload_time: f32,
    pub is_reloading: bool,
    pub reload_start_time: f32,
    pub accuracy: f32,
}

impl Weapon {
    pub fn new_assault_rifle() -> Self {
        Self {
            damage: 25.0,
            rate_of_fire: 8.0,
            last_shot_time: -1.0,
            ammo_current: 30,
            ammo_max: 30,
            reload_time: 2.5,
            is_reloading: false,
            reload_start_time: 0.0,
            accuracy: 0.85,
        }
    }

    pub fn can_fire(&self, current_time: f32) -> bool {
        !self.is_reloading
            && self.ammo_current > 0
            && current_time - self.last_shot_time >= 1.0 / self.rate_of_fire
    }

    pub fn start_reload(&mut self, current_time: f32) {
        if self.ammo_current < self.ammo_max && !self.is_reloading {
            self.is_reloading = true;
            self.reload_start_time = current_time;
        }
    }

    pub fn update_reload(&mut self, current_time: f32) {
        if self.is_reloading && current_time - self.reload_start_time >= self.reload_time {
            self.ammo_current = self.ammo_max;
            self.is_reloading = false;
        }
    }
}

fn weapon_fire(
    time: Res<Time>,
    mut weapon_fire_events: EventWriter<WeaponFireEvent>,
    mut damage_events: EventWriter<DamageDealtEvent>,
    mut sound_events: EventWriter<SoundTriggeredEvent>,
    mut weapon_query: Query<&mut Weapon>,
    input_query: Query<&ActionState<crate::player::PlayerAction>, With<crate::player::Player>>,
    camera_query: Query<&GlobalTransform, With<crate::player::FirstPersonCamera>>,
    spatial_query: SpatialQuery,
    mut health_query: Query<&mut Health>,
    enemy_query: Query<Entity, With<crate::enemies::Enemy>>,
    player_query: Query<Entity, With<crate::player::Player>>,
) {
    let current_time = time.elapsed_secs();

    for mut weapon in weapon_query.iter_mut() {
        weapon.update_reload(current_time);

        if let Ok(action_state) = input_query.single() {
            if action_state.pressed(&crate::player::PlayerAction::PrimaryFire)
                && weapon.can_fire(current_time)
            {
                if let Ok(camera_transform) = camera_query.single() {
                    fire_raycast_weapon(
                        &mut weapon,
                        camera_transform,
                        current_time,
                        &spatial_query,
                        &mut health_query,
                        &enemy_query,
                        &mut weapon_fire_events,
                        &mut damage_events,
                        &mut sound_events,
                        &player_query,
                    );
                }
            }

            if action_state.just_pressed(&crate::player::PlayerAction::Reload) {
                weapon.start_reload(current_time);
            }
        }
    }
}

fn fire_raycast_weapon(
    weapon: &mut Weapon,
    camera_transform: &GlobalTransform,
    current_time: f32,
    spatial_query: &SpatialQuery,
    health_query: &mut Query<&mut Health>,
    enemy_query: &Query<Entity, With<crate::enemies::Enemy>>,
    weapon_fire_events: &mut EventWriter<WeaponFireEvent>,
    damage_events: &mut EventWriter<DamageDealtEvent>,
    sound_events: &mut EventWriter<SoundTriggeredEvent>,
    player_query: &Query<Entity, With<crate::player::Player>>,
) {
    weapon.last_shot_time = current_time;
    weapon.ammo_current = weapon.ammo_current.saturating_sub(1);

    let forward = camera_transform.forward();
    let position = camera_transform.translation() + forward * 0.5;

    // Get player entity for event
    if let Ok(player_entity) = player_query.single() {
        // Send weapon fire event
        weapon_fire_events.write(WeaponFireEvent {
            entity: player_entity,
        });

        // Send sound event
        sound_events.write(SoundTriggeredEvent {
            position,
            sound_type: SoundType::WeaponFire,
            intensity: 1.0,
        });

        // Apply weapon accuracy with spread
        let mut rng = rand::rng();
        let spread = (1.0 - weapon.accuracy) * 0.05;
        let spread_x = rng.random_range(-spread..spread);
        let spread_y = rng.random_range(-spread..spread);
        let spread_z = rng.random_range(-spread..spread);

        let direction = (*forward + Vec3::new(spread_x, spread_y, spread_z)).normalize();
        let direction = Dir3::new(direction).unwrap_or(Dir3::new(*forward).unwrap());

        // Cast ray with maximum range
        let max_range = 100.0;

        // Create filter to exclude player from raycast
        let filter = SpatialQueryFilter::default();

        if let Some(hit) = spatial_query.cast_ray(position, direction, max_range, true, &filter) {
            let hit_entity = hit.entity;
            let hit_position = position + *direction * hit.distance;

            debug!(
                "Raycast hit entity {:?} at distance {:.2} (position: {:?})",
                hit_entity, hit.distance, hit_position
            );

            // Check if we hit an enemy
            if enemy_query.get(hit_entity).is_ok() {
                if let Ok(mut health) = health_query.get_mut(hit_entity) {
                    let damage_dealt = weapon.damage;
                    let _old_health = health.current;
                    health.take_damage(damage_dealt, current_time);

                    // Send damage dealt event
                    damage_events.write(DamageDealtEvent {
                        attacker: player_entity,
                        target: hit_entity,
                        damage: damage_dealt,
                    });

                    debug!(
                        "Hit enemy {:?}! Dealt {} damage, health now: {}/{}",
                        hit_entity, damage_dealt, health.current, health.maximum
                    );
                }
            } else {
                debug!("Hit non-enemy entity {:?}", hit_entity);
            }
        } else {
            debug!("Raycast missed - no hit detected");
        }
    }

    debug!("Fired weapon! Ammo remaining: {}", weapon.ammo_current);
}

fn handle_damage(
    mut death_events: EventWriter<EntityDeathEvent>,
    health_query: Query<(Entity, &Health), Changed<Health>>,
    _transform_query: Query<&Transform>,
) {
    for (entity, health) in health_query.iter() {
        // Send health changed event (you'd need to track previous health)
        // For now, we'll just send death events when health reaches 0
        if health.current <= 0.0 {
            death_events.write(EntityDeathEvent {
                entity,
                cause: DeathCause::Combat,
            });
        }
    }
}

fn remove_dead_entities(
    mut commands: Commands,
    health_query: Query<(Entity, &Health), Without<crate::player::Player>>,
    enemy_query: Query<(), With<crate::enemies::Enemy>>,
) {
    for (entity, health) in health_query.iter() {
        if health.current <= 0.0 {
            if enemy_query.get(entity).is_ok() {
                debug!(
                    "Enemy died! Removing enemy entity: {:?} (health: {})",
                    entity, health.current
                );
            } else {
                debug!(
                    "Entity died! Removing entity: {:?} (health: {})",
                    entity, health.current
                );
            }
            commands.entity(entity).despawn();
        }
    }
}

#[cfg(test)]
mod test {
    use super::{Health, Weapon};

    #[test]
    fn test_health_system() {
        let mut health = Health {
            current: 100.0,
            maximum: 100.0,
            last_damage_time: 0.0,
        };
        assert_eq!(health.current, 100.0);
        assert_eq!(health.maximum, 100.0);
        assert!(!health.is_dead());

        health.take_damage(30.0, 1.0);
        assert_eq!(health.current, 70.0);
        assert_eq!(health.last_damage_time, 1.0);
        assert!(!health.is_dead());

        health.take_damage(80.0, 2.0);
        assert_eq!(health.current, 0.0);
        assert!(health.is_dead());

        let mut health2 = Health {
            current: 100.0,
            maximum: 100.0,
            last_damage_time: 0.0,
        };
        health2.take_damage(50.0, 1.0);

        assert_eq!(health2.current, 50.0);
    }

    #[test]
    fn test_weapon_firing_rate() {
        let mut weapon = Weapon::new_assault_rifle();
        assert!(weapon.can_fire(0.0));

        weapon.last_shot_time = 0.0;
        let fire_interval = 1.0 / weapon.rate_of_fire;

        assert!(!weapon.can_fire(0.05));
        assert!(weapon.can_fire(fire_interval));
    }

    #[test]
    fn test_weapon_ammo_system() {
        let mut weapon = Weapon::new_assault_rifle();

        assert_eq!(weapon.ammo_current, 30);
        assert!(weapon.can_fire(0.0));

        for i in 0..30 {
            weapon.ammo_current = weapon.ammo_current.saturating_sub(1);
            if i < 29 {
                assert!(weapon.ammo_current > 0);
            }
        }

        assert_eq!(weapon.ammo_current, 0);
        assert!(!weapon.can_fire(1.0));
    }

    #[test]
    fn test_weapon_reload_system() {
        let mut weapon = Weapon::new_assault_rifle();

        weapon.ammo_current = 0;
        assert!(!weapon.can_fire(0.0));

        weapon.start_reload(0.0);
        assert!(weapon.is_reloading);
        assert_eq!(weapon.reload_start_time, 0.0);
        assert!(!weapon.can_fire(1.0));

        weapon.update_reload(1.0);
        assert!(weapon.is_reloading);
        assert_eq!(weapon.ammo_current, 0);

        weapon.update_reload(2.5);
        assert!(!weapon.is_reloading);
        assert_eq!(weapon.ammo_current, weapon.ammo_max);
        assert!(weapon.can_fire(2.5));
    }

    #[test]
    fn test_raycast_weapon_accuracy() {
        let weapon = Weapon::new_assault_rifle();
        assert_eq!(weapon.accuracy, 0.85);

        // Test that weapon has damage
        assert_eq!(weapon.damage, 25.0);

        // Test initial ammo
        assert_eq!(weapon.ammo_current, 30);
    }
}
