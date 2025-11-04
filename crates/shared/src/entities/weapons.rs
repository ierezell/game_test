use crate::input::PlayerAction;
use avian3d::prelude::{Collider, LinearVelocity, Position, RigidBody, Rotation};
use bevy::prelude::{Commands, Component, Entity, Name, Query, Res, Time, Timer, TimerMode, Vec3};
use leafwing_input_manager::prelude::ActionState;
use serde::{Deserialize, Serialize};
use std::time::Duration;
/// Minimal weapon component for a single hardcoded gun (Pistol)
#[derive(Component, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SimpleGun {
    pub cooldown: Timer,
}

impl Default for SimpleGun {
    fn default() -> Self {
        Self {
            cooldown: Timer::from_seconds(0.3, TimerMode::Once), // ~3 shots/sec
        }
    }
}

/// Minimal projectile component
#[derive(Component, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SimpleProjectile {
    pub damage: f32,
    pub shooter: Entity,
    pub lifetime: Timer,
    pub has_hit: bool,
}

/// System to fire a simple gun
pub fn fire_simple_gun(
    mut commands: Commands,
    mut query: Query<(Entity, &mut SimpleGun, &Position, &Rotation)>,
    action_state: Res<ActionState<PlayerAction>>,
) {
    for (entity, mut gun, pos, rot) in query.iter_mut() {
        gun.cooldown.tick(Duration::from_secs_f32(1.0 / 60.0));
        if action_state.pressed(&PlayerAction::Shoot) && gun.cooldown.is_finished() {
            // Fire a projectile
            let direction = rot.0 * Vec3::Z;
            commands.spawn((
                Name::new("SimpleProjectile"),
                Position(pos.0),
                LinearVelocity(direction * 20.0),
                RigidBody::Kinematic,
                Collider::sphere(0.1),
                SimpleProjectile {
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

/// System to update and despawn projectiles
pub fn update_simple_projectiles(
    mut commands: Commands,
    mut query: Query<(Entity, &mut SimpleProjectile)>,
    time: Res<Time>,
) {
    for (entity, mut proj) in query.iter_mut() {
        proj.lifetime.tick(time.delta());
        if proj.lifetime.is_finished() || proj.has_hit {
            commands.entity(entity).despawn();
        }
    }
}

pub fn add_weapon_holder(commands: &mut Commands, player_entity: Entity) {
    commands.entity(player_entity).with_children(|parent| {
        parent.spawn((Name::new("SimpleGun"), SimpleGun::default()));
    });
}
