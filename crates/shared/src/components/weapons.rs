use crate::components::health::DamageEvent;
use crate::inputs::input::PlayerAction;
use avian3d::prelude::{
    Collider, LinearVelocity, Position, RigidBody, Rotation, SpatialQueryFilter,
    SpatialQueryPipeline,
};
use bevy::ecs::query::With;
use bevy::prelude::{
    Commands, Component, Dir3, Entity, MessageWriter, Query, Res, Time, Timer, TimerMode, Vec3,
    info,
};
use leafwing_input_manager::prelude::ActionState;
use lightyear::prelude::ControlledBy;
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
    pub magazine_size: u32,
    pub ammo_in_magazine: u32,
    pub reload_timer: Timer,
    pub is_reloading: bool,
}

impl Default for Gun {
    fn default() -> Self {
        let magazine_size = 8;
        Self {
            cooldown: Timer::from_seconds(0.3, TimerMode::Once), // ~3 shots/sec
            damage: 25.0,
            range: 100.0,
            magazine_size,
            ammo_in_magazine: magazine_size,
            reload_timer: Timer::from_seconds(1.2, TimerMode::Once),
            is_reloading: false,
        }
    }
}

impl Gun {
    pub fn start_reload(&mut self) {
        if self.is_reloading || self.ammo_in_magazine >= self.magazine_size {
            return;
        }

        self.is_reloading = true;
        self.reload_timer.reset();
    }

    pub fn tick_reload(&mut self, delta: std::time::Duration) {
        if !self.is_reloading {
            return;
        }

        self.reload_timer.tick(delta);
        if self.reload_timer.is_finished() {
            self.is_reloading = false;
            self.ammo_in_magazine = self.magazine_size;
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
    mut query: Query<
        (
            Entity,
            &mut Gun,
            &Position,
            &Rotation,
            &ActionState<PlayerAction>,
        ),
        With<ControlledBy>,
    >,
    spatial_query: Res<SpatialQueryPipeline>,
    mut damage_writer: MessageWriter<DamageEvent>,
    time: Res<Time>,
) {
    for (shooter_entity, mut gun, pos, rot, action_state) in query.iter_mut() {
        gun.cooldown.tick(time.delta());

        if action_state.disabled() {
            continue;
        }

        if action_state.just_pressed(&PlayerAction::Reload) {
            gun.start_reload();
        }

        gun.tick_reload(time.delta());

        if action_state.pressed(&PlayerAction::Shoot) && gun.cooldown.is_finished() {
            if gun.is_reloading {
                continue;
            }

            if gun.ammo_in_magazine == 0 {
                gun.start_reload();
                continue;
            }

            // Calculate shooting direction from current player look rotation.
            let direction = shoot_direction(rot);

            // Create raycast filter to exclude the shooter
            let filter = SpatialQueryFilter::default().with_excluded_entities([shooter_entity]);

            // Perform raycast from camera position (eye level)
            let eye_height = 1.5; // Approximate player eye height
            let shoot_origin = pos.0 + Vec3::new(0.0, eye_height, 0.0);

            // Perform raycast
            if let Some(hit) = spatial_query.cast_ray(
                shoot_origin,
                Dir3::new(direction).unwrap_or(Dir3::NEG_Z),
                gun.range,
                true, // solid hits only
                &filter,
            ) {
                let hit_entity = hit.entity;
                let hit_point = shoot_origin + direction * hit.distance;

                info!(
                    "🔫 Gun hit entity {:?} at distance {:.2}m point {:?}",
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
            } else {
                info!("🔫 Gun fired but missed (no hit detected)");
            }

            gun.ammo_in_magazine = gun.ammo_in_magazine.saturating_sub(1);
            gun.cooldown.reset();
        }
    }
}

fn shoot_direction(rotation: &Rotation) -> Vec3 {
    (rotation.0 * Vec3::NEG_Z).normalize_or_zero()
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

        if action_state.disabled() {
            continue;
        }

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

#[cfg(test)]
mod tests {
    use super::{Gun, shoot_direction};
    use avian3d::prelude::Rotation;
    use bevy::prelude::{Quat, Vec3};
    use std::time::Duration;

    #[test]
    fn shoot_direction_is_finite_and_normalized() {
        let rotation = Rotation::from(Quat::from_rotation_y(0.6) * Quat::from_rotation_x(-0.3));
        let direction = shoot_direction(&rotation);

        assert!(
            direction.is_finite(),
            "Shoot direction should be finite, got {:?}",
            direction
        );
        assert!(
            direction.length_squared() > 0.99 && direction.length_squared() < 1.01,
            "Shoot direction should be normalized, len2={}",
            direction.length_squared()
        );
    }

    #[test]
    fn hit_point_math_is_finite_when_using_shoot_direction() {
        let rotation = Rotation::from(Quat::from_rotation_x(-0.2));
        let direction = shoot_direction(&rotation);
        let shoot_origin = Vec3::new(4.0, 2.0, -1.0);
        let hit_point = shoot_origin + direction * 21.03;

        assert!(hit_point.is_finite(), "Hit point should be finite");
    }

    #[test]
    fn gun_reload_refills_magazine_after_duration() {
        let mut gun = Gun {
            ammo_in_magazine: 0,
            ..Gun::default()
        };

        gun.start_reload();
        assert!(gun.is_reloading);

        gun.tick_reload(Duration::from_secs_f32(0.6));
        assert!(gun.is_reloading);
        assert_eq!(gun.ammo_in_magazine, 0);

        gun.tick_reload(Duration::from_secs_f32(1.0));
        assert!(!gun.is_reloading);
        assert_eq!(gun.ammo_in_magazine, gun.magazine_size);
    }

    #[test]
    fn start_reload_is_noop_when_magazine_already_full() {
        let mut gun = Gun::default();
        gun.start_reload();

        assert!(!gun.is_reloading);
        assert_eq!(gun.ammo_in_magazine, gun.magazine_size);
    }
}
