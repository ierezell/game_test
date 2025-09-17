use avian3d::prelude::Position;
use bevy::prelude::{Component, Reflect, Res, Time, Vec2, Vec3, debug};

use leafwing_input_manager::Actionlike;

use leafwing_input_manager::prelude::ActionState;

use serde::{Deserialize, Serialize};

use bevy::prelude::*;

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PlayerMarker;

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PlayerColor(pub Color);

#[derive(
    Clone, Copy, PartialEq, Eq, Hash, Debug, Reflect, Serialize, Deserialize, Actionlike, Default,
)]
pub enum PlayerAction {
    #[default]
    #[actionlike(DualAxis)]
    Move,

    #[actionlike(Button)]
    Jump,

    #[actionlike(Button)]
    Shoot,
}

pub const PLAYER_CAPSULE_RADIUS: f32 = 0.5;
pub const PLAYER_CAPSULE_HEIGHT: f32 = 1.5;
pub const MAX_SPEED: f32 = 5.0;
pub const JUMP_HEIGHT: f32 = 1.5;

#[derive(Component, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlayerVelocity(pub Vec3);

pub fn apply_player_action(
    time: &Res<Time>,
    action_state: &ActionState<PlayerAction>,
    position: &mut Position,
    velocity: &mut PlayerVelocity,
) {
    let dt = time.delta_secs();

    let move_dir = action_state
        .axis_pair(&PlayerAction::Move)
        .clamp_length_max(1.0);

    if move_dir != Vec2::ZERO {
        debug!("🎮 MOVEMENT INPUT DETECTED: {:?}", move_dir);
    }

    let move_dir_3d = Vec3::new(move_dir.x, 0.0, -move_dir.y);

    velocity.0.x = move_dir_3d.x * MAX_SPEED;
    velocity.0.z = move_dir_3d.z * MAX_SPEED;

    if action_state.just_pressed(&PlayerAction::Jump) {
        velocity.0.y = (2.0 * 9.81 * JUMP_HEIGHT).sqrt();
        debug!("🦘 JUMP TRIGGERED!");
    }

    velocity.0.y -= 9.81 * dt;

    let old_position = position.0;
    position.0 += velocity.0 * dt;

    if position.0.y < 1.0 {
        position.0.y = 1.0;
        if velocity.0.y < 0.0 {
            velocity.0.y = 0.0;
        }
    }

    if move_dir != Vec2::ZERO {
        debug!(
            "🚶 PLAYER MOVEMENT - Input: {:?}, Velocity: {:?}, Old Pos: {:?}, New Pos: {:?}",
            move_dir, velocity.0, old_position, position.0
        );
    }
}
