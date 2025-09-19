use avian3d::prelude::{LinearVelocity, Position, Rotation};
use bevy::prelude::{Component, Reflect, Res, Time, Vec2, Vec3, debug};

use leafwing_input_manager::Actionlike;

use leafwing_input_manager::prelude::ActionState;

use serde::{Deserialize, Serialize};

use bevy::prelude::*;

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PlayerColor(pub Color);

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct PlayerCamera {
    pub yaw: f32,
    pub pitch: f32,
}

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct MouseInput {
    pub delta: Vec2,
}

#[derive(
    Clone, Copy, PartialEq, Eq, Hash, Debug, Reflect, Serialize, Deserialize, Actionlike, Default,
)]
pub enum PlayerAction {
    #[default]
    #[actionlike(DualAxis)]
    Move,

    #[actionlike(DualAxis)]
    Look,

    #[actionlike(Button)]
    Jump,

    #[actionlike(Button)]
    Shoot,
}

pub const PLAYER_CAPSULE_RADIUS: f32 = 0.5;
pub const PLAYER_CAPSULE_HEIGHT: f32 = 1.5;
pub const MAX_SPEED: f32 = 5.0;
pub const JUMP_HEIGHT: f32 = 1.5;
pub const MOUSE_SENSITIVITY: f32 = 0.005;

pub fn shared_player_movement(
    _time: &Res<Time>,
    action_state: &ActionState<PlayerAction>,
    position: &mut Position,
    rotation: &Rotation,
    velocity: &mut LinearVelocity,
) {
    // Handle movement relative to player rotation
    let move_dir = action_state
        .axis_pair(&PlayerAction::Move)
        .clamp_length_max(1.0);

    if move_dir != Vec2::ZERO {
        debug!("🎮 MOVEMENT INPUT DETECTED: {:?}", move_dir);
    }

    // Convert WASD input to movement relative to player rotation
    let input_dir = Vec3::new(move_dir.x, 0.0, -move_dir.y);

    // Apply player rotation to movement direction
    let world_dir = rotation.0 * input_dir;

    // Apply horizontal movement
    velocity.0.x = world_dir.x * MAX_SPEED;
    velocity.0.z = world_dir.z * MAX_SPEED;

    // Handle jumping - add upward velocity if on ground
    if action_state.just_pressed(&PlayerAction::Jump) {
        // Improved ground check - player capsule radius + small tolerance
        let ground_threshold = PLAYER_CAPSULE_RADIUS + 0.1;
        if position.0.y <= ground_threshold && velocity.0.y.abs() < 0.1 {
            velocity.0.y = (2.0 * 9.81 * JUMP_HEIGHT).sqrt();
            debug!(
                "🦘 JUMP TRIGGERED! Position Y: {:.2}, Threshold: {:.2}",
                position.0.y, ground_threshold
            );
        }
    }

    if move_dir != Vec2::ZERO {
        debug!(
            "🚶 SHARED MOVEMENT - Input: {:?}, Player Rotation: {:?}, Velocity: {:?}, Position: {:?}",
            move_dir, rotation.0, velocity.0, position.0
        );
    }
}
