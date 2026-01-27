use avian3d::prelude::{
    AngularDamping, Collider, Friction, LinearDamping, LockedAxes, Mass, Restitution, RigidBody,
};

use crate::input::{PLAYER_CAPSULE_HEIGHT, PLAYER_CAPSULE_RADIUS};
use bevy::prelude::{Bundle, Color};

/// Physics bundle specialized for player-controlled entities.
/// Used for Predicted entities and Server entities (full physics simulation)
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
            linear_damping: LinearDamping(2.0),
            angular_damping: AngularDamping(8.0),
            locked_axes: LockedAxes::ROTATION_LOCKED.unlock_rotation_y(),
        }
    }
}

/// Physics bundle specialized for NPCs / AI-controlled entities.
/// Used for Server NPC entities (full physics simulation)
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
            linear_damping: LinearDamping(1.5),
            angular_damping: AngularDamping(6.0),
            locked_axes: LockedAxes::ROTATION_LOCKED.unlock_rotation_y(),
        }
    }
}

/// Kinematic bundle for interpolated entities (remote players/NPCs on client)
/// These entities don't simulate physics - they only display replicated Position
/// Kinematic RigidBody allows Position → Transform sync without physics simulation
#[derive(Bundle)]
pub struct KinematicDisplayBundle {
    pub rigid_body: RigidBody,
    pub collider: Collider,
}

impl Default for KinematicDisplayBundle {
    fn default() -> Self {
        Self {
            rigid_body: RigidBody::Kinematic, // Kinematic = no physics, just positioning
            collider: Collider::capsule(PLAYER_CAPSULE_HEIGHT, PLAYER_CAPSULE_RADIUS),
        }
    }
}

pub fn color_from_id(id: u64) -> Color {
    let hue = (id as f32 * 137.508) % 360.0;
    Color::hsl(hue, 0.8, 0.6)
}
