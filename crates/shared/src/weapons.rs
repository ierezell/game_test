use crate::health::{DamageEvent, DamageType};
use avian3d::prelude::{Collider, LinearVelocity, Position, RigidBody};
use bevy::prelude::*;
use lightyear::prelude::Tick;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

pub struct WeaponPlugin;

impl Plugin for WeaponPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<FireWeaponEvent>()
            .add_message::<SpawnProjectileEvent>()
            .add_message::<ProjectileHitEvent>()
            .add_systems(
                FixedUpdate,
                (
                    process_weapon_fire_events,
                    process_spawn_projectile_events,
                    update_projectiles,
                    process_projectile_hits,
                    update_weapon_cooldowns,
                    update_projectile_spawn_buffers,
                ),
            );
    }
}

/// Component for entities that can hold weapons
#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct WeaponHolder {
    pub current_weapon: Option<WeaponType>,
    pub weapons: Vec<Weapon>,
    pub ammo: std::collections::HashMap<AmmoType, u32>,
}

impl Default for WeaponHolder {
    fn default() -> Self {
        Self {
            current_weapon: Some(WeaponType::Pistol),
            weapons: vec![
                Weapon::new(WeaponType::Pistol),
                Weapon::new(WeaponType::Rifle),
            ],
            ammo: [
                (AmmoType::Pistol, 100),
                (AmmoType::Rifle, 60),
                (AmmoType::Shotgun, 30),
            ]
            .iter()
            .cloned()
            .collect(),
        }
    }
}

/// Different weapon types available in the game
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WeaponType {
    Pistol,
    Rifle,
    Shotgun,
    Sniper,
}

/// Different ammunition types
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AmmoType {
    Pistol,
    Rifle,
    Shotgun,
    Sniper,
}

/// Projectile replication methods for different weapon types
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProjectileReplicationMode {
    /// Only send spawn parameters (position, direction, speed) - best for linear projectiles
    DirectionOnly,
    /// Send full entity with updates - best for complex projectiles (homing, bouncing)
    FullEntity,
    /// Use ring buffer in weapon component for batched spawning
    RingBuffer,
}

/// Data for spawning a projectile using direction-only replication
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectileSpawn {
    pub tick: Tick,
    pub position: Vec3,
    pub direction: Vec3,
    pub speed: f32,
    pub damage: f32,
    pub weapon_type: WeaponType,
    pub shooter: Entity,
    pub lifetime: f32,
}

/// Ring buffer component for efficient projectile spawn batching
#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct ProjectileSpawnBuffer {
    pub spawns: VecDeque<ProjectileSpawn>,
    pub max_size: usize,
}

impl Default for ProjectileSpawnBuffer {
    fn default() -> Self {
        Self {
            spawns: VecDeque::new(),
            max_size: 32, // Buffer up to 32 projectile spawns
        }
    }
}

impl ProjectileSpawnBuffer {
    pub fn add_spawn(&mut self, spawn: ProjectileSpawn) {
        self.spawns.push_back(spawn);
        while self.spawns.len() > self.max_size {
            self.spawns.pop_front();
        }
    }

    pub fn get_recent_spawns(&self) -> &VecDeque<ProjectileSpawn> {
        &self.spawns
    }

    pub fn clear_old_spawns(&mut self, current_tick: u32, max_age_ticks: u32) {
        while let Some(front) = self.spawns.front() {
            if current_tick.wrapping_sub(front.tick.0.into()) > max_age_ticks {
                self.spawns.pop_front();
            } else {
                break;
            }
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Weapon {
    pub weapon_type: WeaponType,
    pub damage: f32,
    pub fire_rate: f32,   // Rounds per second
    pub reload_time: f32, // Seconds
    pub magazine_size: u32,
    pub current_ammo: u32,
    pub range: f32,
    pub accuracy: f32, // 0.0 to 1.0
    pub projectile_speed: f32,
    pub ammo_type: AmmoType,
    pub replication_mode: ProjectileReplicationMode,
    pub cooldown_timer: Timer,
    pub reload_timer: Timer,
    pub is_reloading: bool,
}

impl Weapon {
    pub fn new(weapon_type: WeaponType) -> Self {
        let (
            damage,
            fire_rate,
            reload_time,
            magazine_size,
            range,
            accuracy,
            projectile_speed,
            ammo_type,
            replication_mode,
        ) = match weapon_type {
            WeaponType::Pistol => (
                25.0,
                3.0,
                1.5,
                12,
                15.0,
                0.85,
                20.0,
                AmmoType::Pistol,
                ProjectileReplicationMode::DirectionOnly,
            ),
            WeaponType::Rifle => (
                35.0,
                8.0,
                2.5,
                30,
                30.0,
                0.90,
                35.0,
                AmmoType::Rifle,
                ProjectileReplicationMode::DirectionOnly,
            ),
            WeaponType::Shotgun => (
                60.0,
                1.2,
                2.0,
                8,
                8.0,
                0.70,
                15.0,
                AmmoType::Shotgun,
                ProjectileReplicationMode::RingBuffer,
            ),
            WeaponType::Sniper => (
                100.0,
                0.8,
                3.0,
                5,
                50.0,
                0.95,
                50.0,
                AmmoType::Sniper,
                ProjectileReplicationMode::DirectionOnly,
            ),
        };

        Self {
            weapon_type,
            damage,
            fire_rate,
            reload_time,
            magazine_size,
            current_ammo: magazine_size,
            range,
            accuracy,
            projectile_speed,
            ammo_type,
            replication_mode,
            cooldown_timer: Timer::from_seconds(1.0 / fire_rate, TimerMode::Once),
            reload_timer: Timer::from_seconds(reload_time, TimerMode::Once),
            is_reloading: false,
        }
    }

    pub fn can_fire(&self) -> bool {
        !self.is_reloading && self.current_ammo > 0 && self.cooldown_timer.is_finished()
    }

    pub fn start_reload(&mut self) {
        if self.current_ammo < self.magazine_size && !self.is_reloading {
            self.is_reloading = true;
            self.reload_timer.reset();
        }
    }

    pub fn fire(&mut self) -> bool {
        if self.can_fire() {
            self.current_ammo -= 1;
            self.cooldown_timer.reset();
            return true;
        }
        false
    }
}

/// Component for projectiles
#[derive(Component, Clone, Debug)]
pub struct Projectile {
    pub damage: f32,
    pub damage_type: DamageType,
    pub shooter: Entity,
    pub lifetime: Timer,
    pub has_hit: bool,
}

/// Event for firing weapons with enhanced replication
#[derive(Message, Clone, Debug, Serialize, Deserialize)]
pub struct FireWeaponEvent {
    pub shooter: Entity,
    pub weapon_type: WeaponType,
    pub origin: Vec3,
    pub direction: Vec3,
    pub tick: Tick,
}

/// Event for direction-only projectile spawning (more efficient)
#[derive(Message, Clone, Debug, Serialize, Deserialize)]
pub struct SpawnProjectileEvent {
    pub spawn_data: ProjectileSpawn,
}

/// Event for projectile hits
#[derive(Message, Clone, Debug, Serialize, Deserialize)]
pub struct ProjectileHitEvent {
    pub projectile: Entity,
    pub target: Entity,
    pub hit_point: Vec3,
    pub damage: f32,
    pub damage_type: DamageType,
    pub shooter: Entity,
}

/// System to process weapon fire events and handle different replication modes
fn process_weapon_fire_events(
    mut fire_events: MessageReader<FireWeaponEvent>,
    mut weapon_query: Query<(&mut WeaponHolder, Option<&mut ProjectileSpawnBuffer>)>,
    mut spawn_events: MessageWriter<SpawnProjectileEvent>,
    mut commands: Commands,
    _time: Res<Time>,
) {
    for fire_event in fire_events.read() {
        if let Ok((mut weapon_holder, spawn_buffer)) = weapon_query.get_mut(fire_event.shooter) {
            if let Some(current_weapon_type) = weapon_holder.current_weapon.clone() {
                // Find the current weapon
                if let Some(weapon) = weapon_holder
                    .weapons
                    .iter_mut()
                    .find(|w| w.weapon_type == current_weapon_type)
                {
                    if weapon.fire() {
                        let spawn_data = ProjectileSpawn {
                            tick: fire_event.tick,
                            position: fire_event.origin,
                            direction: fire_event.direction.normalize(),
                            speed: weapon.projectile_speed,
                            damage: weapon.damage,
                            weapon_type: weapon.weapon_type.clone(),
                            shooter: fire_event.shooter,
                            lifetime: weapon.range / weapon.projectile_speed,
                        };

                        match weapon.replication_mode {
                            ProjectileReplicationMode::DirectionOnly => {
                                // Send spawn event for efficient replication
                                spawn_events.write(SpawnProjectileEvent {
                                    spawn_data: spawn_data.clone(),
                                });

                                // Also spawn locally
                                spawn_projectile_from_data(&mut commands, &spawn_data);
                            }
                            ProjectileReplicationMode::RingBuffer => {
                                // Add to spawn buffer for batched replication
                                if let Some(mut buffer) = spawn_buffer {
                                    buffer.add_spawn(spawn_data.clone());
                                }

                                // Spawn locally
                                spawn_projectile_from_data(&mut commands, &spawn_data);
                            }
                            ProjectileReplicationMode::FullEntity => {
                                // Traditional full entity replication (fall back to old method)
                                spawn_full_entity_projectile(&mut commands, &spawn_data);
                            }
                        }

                        info!(
                            "Weapon fired by {:?} using {:?} replication",
                            fire_event.shooter, weapon.replication_mode
                        );
                    }
                }
            }
        }
    }
}

/// System to process direction-only projectile spawn events
fn process_spawn_projectile_events(
    mut spawn_events: MessageReader<SpawnProjectileEvent>,
    mut commands: Commands,
) {
    for spawn_event in spawn_events.read() {
        spawn_projectile_from_data(&mut commands, &spawn_event.spawn_data);
    }
}

/// Helper function to spawn a projectile from spawn data
fn spawn_projectile_from_data(commands: &mut Commands, spawn_data: &ProjectileSpawn) {
    let projectile_entity = commands
        .spawn((
            Name::new("Projectile"),
            Position(spawn_data.position),
            LinearVelocity(spawn_data.direction * spawn_data.speed),
            RigidBody::Kinematic,
            Collider::sphere(0.1),
            Projectile {
                damage: spawn_data.damage,
                damage_type: DamageType::Physical,
                shooter: spawn_data.shooter,
                lifetime: Timer::from_seconds(spawn_data.lifetime, TimerMode::Once),
                has_hit: false,
            },
        ))
        .id();

    debug!("Spawned projectile {:?} from spawn data", projectile_entity);
}

/// Helper function to spawn traditional full-entity projectile
fn spawn_full_entity_projectile(commands: &mut Commands, spawn_data: &ProjectileSpawn) {
    let projectile_entity = commands
        .spawn((
            Name::new("FullEntityProjectile"),
            Position(spawn_data.position),
            LinearVelocity(spawn_data.direction * spawn_data.speed),
            RigidBody::Kinematic,
            Collider::sphere(0.1),
            Projectile {
                damage: spawn_data.damage,
                damage_type: DamageType::Physical,
                shooter: spawn_data.shooter,
                lifetime: Timer::from_seconds(spawn_data.lifetime, TimerMode::Once),
                has_hit: false,
            },
            // Add replication components for full entity mode
            // Replicate::default(), // Would add in actual implementation
        ))
        .id();

    debug!("Spawned full entity projectile {:?}", projectile_entity);
}

/// System to update and clean projectile spawn buffers
fn update_projectile_spawn_buffers(
    mut buffer_query: Query<&mut ProjectileSpawnBuffer>,
    _time: Res<Time>,
) {
    // This would use actual game tick in real implementation
    let current_tick = 0u32; // Placeholder - should use actual tick counter
    let max_age_ticks = 120; // ~2 seconds at 60Hz

    for mut buffer in buffer_query.iter_mut() {
        buffer.clear_old_spawns(current_tick, max_age_ticks);
    }
}

/// System to update projectile lifetime and remove expired projectiles
fn update_projectiles(
    mut projectile_query: Query<(Entity, &mut Projectile)>,
    mut commands: Commands,
    time: Res<Time>,
) {
    for (entity, mut projectile) in projectile_query.iter_mut() {
        projectile.lifetime.tick(time.delta());

        if projectile.lifetime.is_finished() || projectile.has_hit {
            commands.entity(entity).despawn();
        }
    }
}

/// System to process projectile collision hits
fn process_projectile_hits(
    mut hit_events: MessageReader<ProjectileHitEvent>,
    mut projectile_query: Query<&mut Projectile>,
    mut damage_events: MessageWriter<DamageEvent>,
) {
    for hit_event in hit_events.read() {
        // Mark projectile as hit
        if let Ok(mut projectile) = projectile_query.get_mut(hit_event.projectile) {
            projectile.has_hit = true;
        }

        // Send damage event
        damage_events.write(DamageEvent {
            target: hit_event.target,
            amount: hit_event.damage,
            source: Some(hit_event.shooter),
            damage_type: hit_event.damage_type.clone(),
            ignore_invulnerability: false,
        });

        info!(
            "Projectile {:?} hit target {:?} for {:.1} damage",
            hit_event.projectile, hit_event.target, hit_event.damage
        );
    }
}

/// System to update weapon cooldowns and reloading
fn update_weapon_cooldowns(mut weapon_query: Query<&mut WeaponHolder>, time: Res<Time>) {
    for mut weapon_holder in weapon_query.iter_mut() {
        // First pass: update timers and collect reload information
        let mut reload_info = Vec::new();

        for (_i, weapon) in weapon_holder.weapons.iter_mut().enumerate() {
            weapon.cooldown_timer.tick(time.delta());

            if weapon.is_reloading {
                weapon.reload_timer.tick(time.delta());

                if weapon.reload_timer.is_finished() {
                    weapon.is_reloading = false;

                    // Calculate ammo needed and store info for later processing
                    let ammo_needed = weapon.magazine_size - weapon.current_ammo;
                    reload_info.push((
                        weapon.ammo_type.clone(),
                        ammo_needed,
                        weapon.weapon_type.clone(),
                    ));
                }
            }
        }

        // Second pass: process reloads using collected information
        for (ammo_type, ammo_needed, weapon_type) in reload_info {
            let available_ammo = weapon_holder.ammo.get(&ammo_type).copied().unwrap_or(0);
            let ammo_to_reload = ammo_needed.min(available_ammo);

            // Update weapon ammo
            if let Some(weapon) = weapon_holder
                .weapons
                .iter_mut()
                .find(|w| w.weapon_type == weapon_type)
            {
                weapon.current_ammo += ammo_to_reload;
            }

            // Update available ammo
            if let Some(available_ammo) = weapon_holder.ammo.get_mut(&ammo_type) {
                *available_ammo -= ammo_to_reload;

                info!("Reloaded {:?} - used {} ammo", weapon_type, ammo_to_reload);
            }
        }
    }
}

/// Helper function to add weapon holder to an entity
pub fn add_weapon_holder(commands: &mut Commands, entity: Entity) {
    commands.entity(entity).insert(WeaponHolder::default());
}

/// Helper function to trigger weapon fire
pub fn fire_weapon(
    fire_events: &mut MessageWriter<FireWeaponEvent>,
    shooter: Entity,
    weapon_type: WeaponType,
    origin: Vec3,
    direction: Vec3,
    tick: Tick,
) {
    fire_events.write(FireWeaponEvent {
        shooter,
        weapon_type,
        origin,
        direction,
        tick,
    });
}

/// Helper function to trigger projectile hit
pub fn projectile_hit(
    hit_events: &mut MessageWriter<ProjectileHitEvent>,
    projectile: Entity,
    target: Entity,
    hit_point: Vec3,
    damage: f32,
    damage_type: DamageType,
    shooter: Entity,
) {
    hit_events.write(ProjectileHitEvent {
        projectile,
        target,
        hit_point,
        damage,
        damage_type,
        shooter,
    });
}
