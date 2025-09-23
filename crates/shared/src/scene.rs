use avian3d::prelude::*;

use crate::input::{PLAYER_CAPSULE_HEIGHT, PLAYER_CAPSULE_RADIUS};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
pub const ROOM_SIZE: f32 = 20.0;
pub const WALL_HEIGHT: f32 = 3.0;
pub const WALL_THICKNESS: f32 = 0.5;
pub const FLOOR_THICKNESS: f32 = 1.0;

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct FloorMarker;

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct WallMarker;

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CeilingMarker;

#[derive(Bundle)]
pub struct FloorPhysicsBundle {
    pub collider: Collider,
    pub rigid_body: RigidBody,
    pub restitution: Restitution,
}

impl Default for FloorPhysicsBundle {
    fn default() -> Self {
        Self {
            collider: Collider::cuboid(ROOM_SIZE, FLOOR_THICKNESS, ROOM_SIZE),
            rigid_body: RigidBody::Static,
            restitution: Restitution::ZERO,
        }
    }
}

#[derive(Bundle)]
pub struct WallPhysicsBundle {
    pub collider: Collider,
    pub rigid_body: RigidBody,
}

impl Default for WallPhysicsBundle {
    fn default() -> Self {
        Self {
            collider: Collider::cuboid(WALL_THICKNESS, WALL_HEIGHT, ROOM_SIZE),
            rigid_body: RigidBody::Static,
        }
    }
}

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

pub fn color_from_id(id: u64) -> Color {
    let hue = (id as f32 * 137.508) % 360.0;
    Color::hsl(hue, 0.8, 0.6)
}
