use crate::components::health::DamageEvent;
use crate::input::PlayerAction;
use avian3d::prelude::{
    Collider, LinearVelocity, Position, RigidBody, Rotation, SpatialQueryFilter,
    SpatialQueryPipeline,
};
use bevy::prelude::{
    Commands, Component, Dir3, Entity, MessageWriter, Query, Res, Time, Timer, TimerMode, Vec3,
    info,
};
use leafwing_input_manager::prelude::ActionState;
use serde::{Deserialize, Serialize};

pub struct WeaponsPlugin;

impl bevy::prelude::Plugin for WeaponsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(
            bevy::prelude::FixedUpdate,
            (
                fire_gun_system,
                fire_projectile_gun_system,
                update_simple_projectiles,
                process_hit_events,
            ),
        );
    }
}

#[derive(Component, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Gun {
    pub cooldown: Timer,
    pub damage: f32,
    pub range: f32,
}

impl Default for Gun {
    fn default() -> Self {
        Self {
            cooldown: Timer::from_seconds(0.3, TimerMode::Once), // ~3 shots/sec
            damage: 25.0,
            range: 100.0,
        }
    }
}

#[derive(Component, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct HitEvent {
    pub damage: f32,
    pub hit_entity: Entity,
    pub shooter: Entity,
    pub hit_point: Vec3,
}

// Gun use raycast to detect hits. ProjectileGun spawns projectile entities.
pub fn fire_gun_system(
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &mut Gun,
        &Position,
        &Rotation,
        &ActionState<PlayerAction>,
    )>,
    spatial_query: Res<SpatialQueryPipeline>,
    mut damage_writer: MessageWriter<DamageEvent>,
    time: Res<Time>,
) {
    for (shooter_entity, mut gun, pos, rot, action_state) in query.iter_mut() {
        gun.cooldown.tick(time.delta());
        if action_state.pressed(&PlayerAction::Shoot) && gun.cooldown.is_finished() {
            let direction = rot.0 * Vec3::NEG_Z; // Forward direction

            // Create raycast filter to exclude the shooter
            let filter = SpatialQueryFilter::default().with_excluded_entities([shooter_entity]);

            // Perform raycast
            if let Some(hit) = spatial_query.cast_ray(
                pos.0,
                Dir3::new(direction).unwrap_or(Dir3::NEG_Z),
                gun.range,
                true, // solid hits only
                &filter,
            ) {
                let hit_entity = hit.entity;
                let hit_point = pos.0 + direction.normalize() * hit.distance;

                info!(
                    "Gun hit entity {:?} at distance {} point {:?}",
                    hit_entity, hit.distance, hit_point
                );

                // Send damage event - the health system will handle it
                damage_writer.write(DamageEvent {
                    target: hit_entity,
                    amount: gun.damage,
                    source: Some(shooter_entity),
                });

                // Spawn hit event for further processing (effects, sounds, etc.)
                commands.spawn(HitEvent {
                    damage: gun.damage,
                    hit_entity,
                    shooter: shooter_entity,
                    hit_point,
                });
            }

            gun.cooldown.reset();
        }
    }
}

#[derive(Component, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ProjectileGun {
    pub cooldown: Timer,
}

impl Default for ProjectileGun {
    fn default() -> Self {
        Self {
            cooldown: Timer::from_seconds(0.3, TimerMode::Once), // ~3 shots/sec
        }
    }
}

#[derive(Component, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Projectile {
    pub damage: f32,
    pub shooter: Entity,
    pub lifetime: Timer,
    pub has_hit: bool,
}

pub fn fire_projectile_gun_system(
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &mut ProjectileGun,
        &Position,
        &Rotation,
        &ActionState<PlayerAction>,
    )>,
    time: Res<Time>,
) {
    for (entity, mut gun, pos, rot, action_state) in query.iter_mut() {
        gun.cooldown.tick(time.delta());
        if action_state.pressed(&PlayerAction::Shoot) && gun.cooldown.is_finished() {
            let direction = rot.0 * Vec3::NEG_Z;
            commands.spawn((
                Position(pos.0),
                LinearVelocity(direction * 20.0),
                RigidBody::Kinematic,
                Collider::sphere(0.1),
                Projectile {
                    damage: 25.0,
                    shooter: entity,
                    lifetime: Timer::from_seconds(1.0, TimerMode::Once),
                    has_hit: false,
                },
            ));
            gun.cooldown.reset();
        }
    }
}

pub fn update_simple_projectiles(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Projectile)>,
    time: Res<Time>,
) {
    for (entity, mut proj) in query.iter_mut() {
        proj.lifetime.tick(time.delta());
        if proj.lifetime.is_finished() || proj.has_hit {
            commands.entity(entity).despawn();
        }
    }
}

/// System to handle hit events and apply effects (sound, particles, etc.)
pub fn process_hit_events(mut commands: Commands, hit_events: Query<(Entity, &HitEvent)>) {
    for (event_entity, hit_event) in hit_events.iter() {
        // Here you can add visual/audio effects for hits
        info!(
            "Processing hit event: {} damage to {:?} from {:?} at {:?}",
            hit_event.damage, hit_event.hit_entity, hit_event.shooter, hit_event.hit_point
        );

        commands.entity(event_entity).despawn();
    }
}

// Death handling is managed by the health system
