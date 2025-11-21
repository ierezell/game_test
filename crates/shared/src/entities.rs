use avian3d::prelude::{
    AngularDamping, Collider, Friction, LinearDamping, LockedAxes, Mass, Restitution, RigidBody,
};

use crate::input::{PLAYER_CAPSULE_HEIGHT, PLAYER_CAPSULE_RADIUS};
use bevy::prelude::{Bundle, Color};

/// Physics bundle specialized for player-controlled entities.
#[derive(Bundle)]
pub struct PlayerPhysicsBundle {
    pub rigid_body: RigidBody,
    pub collider: Collider,
    pub mass: Mass,
    pub restitution: Restitution,
    pub friction: Friction,
    pub linear_damping: LinearDamping,
    pub angular_damping: AngularDamping,
    pub locked_axes: LockedAxes,
}

impl Default for PlayerPhysicsBundle {
    fn default() -> Self {
        Self {
            rigid_body: RigidBody::Dynamic,
            collider: Collider::capsule(PLAYER_CAPSULE_HEIGHT, PLAYER_CAPSULE_RADIUS),
            mass: Mass(80.0),
            restitution: Restitution::ZERO,
            friction: Friction::new(0.5),
            linear_damping: LinearDamping(1.0),
            angular_damping: AngularDamping(8.0),
            locked_axes: LockedAxes::ROTATION_LOCKED.unlock_rotation_y(),
        }
    }
}

/// Physics bundle specialized for NPCs / AI-controlled entities.
#[derive(Bundle)]
pub struct NpcPhysicsBundle {
    pub rigid_body: RigidBody,
    pub collider: Collider,
    pub mass: Mass,
    pub restitution: Restitution,
    pub friction: Friction,
    pub linear_damping: LinearDamping,
    pub angular_damping: AngularDamping,
    pub locked_axes: LockedAxes,
}

impl Default for NpcPhysicsBundle {
    fn default() -> Self {
        Self {
            rigid_body: RigidBody::Dynamic,
            collider: Collider::capsule(PLAYER_CAPSULE_HEIGHT, PLAYER_CAPSULE_RADIUS),
            mass: Mass(70.0),
            restitution: Restitution::ZERO,
            friction: Friction::new(0.2),
            linear_damping: LinearDamping(0.8),
            angular_damping: AngularDamping(6.0),
            locked_axes: LockedAxes::ROTATION_LOCKED.unlock_rotation_y(),
        }
    }
}

pub fn color_from_id(id: u64) -> Color {
    let hue = (id as f32 * 137.508) % 360.0;
    Color::hsl(hue, 0.8, 0.6)
}
