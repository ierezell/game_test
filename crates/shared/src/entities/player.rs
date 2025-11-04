use avian3d::prelude::{
    AngularDamping, Collider, Friction, LinearDamping, LockedAxes, Mass, Restitution, RigidBody,
};

use crate::input::{PLAYER_CAPSULE_HEIGHT, PLAYER_CAPSULE_RADIUS};
use bevy::prelude::{Bundle, Color};

#[derive(Bundle)]
pub struct PlayerPhysicsBundle {
    pub rigid_body: RigidBody,
    pub collider: Collider,
    pub mass: Mass,
    pub restitution: Restitution,
    pub friction: Friction,
    pub linear_damping: LinearDamping,
    pub angular_damping: AngularDamping,
    pub locked_axes: LockedAxes, // Prevent capsizing
}

impl Default for PlayerPhysicsBundle {
    fn default() -> Self {
        Self {
            rigid_body: RigidBody::Dynamic,
            collider: Collider::capsule(PLAYER_CAPSULE_HEIGHT, PLAYER_CAPSULE_RADIUS),
            mass: Mass(80.0),
            restitution: Restitution::ZERO,
            friction: Friction::ZERO,
            linear_damping: LinearDamping(1.0),
            angular_damping: AngularDamping(8.0),
            locked_axes: LockedAxes::ROTATION_LOCKED.unlock_rotation_y(),
        }
    }
}

/// Generate a unique color from an ID using the golden ratio for good distribution
/// This creates visually distinct colors for different entities
pub fn color_from_id(id: u64) -> Color {
    let hue = (id as f32 * 137.508) % 360.0; // Golden ratio * 360
    Color::hsl(hue, 0.8, 0.6)
}
