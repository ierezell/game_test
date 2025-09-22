use avian3d::prelude::{LinearVelocity, Rotation};
use bevy::prelude::{Reflect, Res, Time, Vec2, Vec3};

use leafwing_input_manager::Actionlike;

use leafwing_input_manager::prelude::ActionState;

use serde::{Deserialize, Serialize};

use bevy::prelude::*;

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
const LOOK_DEADZONE_SQUARED: f32 = 0.000001; // 0.001^2
const MOVEMENT_DEADZONE_SQUARED: f32 = 0.000001;
pub const PITCH_LIMIT_RADIANS: f32 = std::f32::consts::FRAC_PI_2 - 0.01;
pub const ROTATION_SMOOTHING_RATE: f32 = 25.0; // Higher = more responsive

pub fn shared_player_movement(
    time: &Res<Time>,
    action_state: &ActionState<PlayerAction>,
    rotation: &mut Rotation,
    velocity: &mut LinearVelocity,
) {
    let dt = time.delta_secs();

    let move_input = get_movement_input(action_state);

    if let Some(mouse_delta) = get_look_input(action_state) {
        update_player_rotation(rotation, mouse_delta, dt);
    }

    update_player_velocity(velocity, rotation, move_input);
}

#[inline]
fn get_movement_input(action_state: &ActionState<PlayerAction>) -> Vec2 {
    let move_input = action_state
        .axis_pair(&PlayerAction::Move)
        .clamp_length_max(1.0);

    if move_input.length_squared() < MOVEMENT_DEADZONE_SQUARED {
        Vec2::ZERO
    } else {
        move_input
    }
}

#[inline]
fn get_look_input(action_state: &ActionState<PlayerAction>) -> Option<Vec2> {
    let mouse_delta = action_state.axis_pair(&PlayerAction::Look);

    if mouse_delta.length_squared() < LOOK_DEADZONE_SQUARED {
        None
    } else {
        Some(mouse_delta)
    }
}

fn update_player_rotation(rotation: &mut Rotation, mouse_delta: Vec2, dt: f32) {
    let yaw_delta = -mouse_delta.x * MOUSE_SENSITIVITY;
    // let pitch_delta = mouse_delta.y * MOUSE_SENSITIVITY;

    let (mut yaw, _, _) = rotation.0.to_euler(EulerRot::YXZ);
    yaw = (yaw + yaw_delta).rem_euclid(std::f32::consts::TAU);
    // pitch = (pitch + pitch_delta).clamp(-PITCH_LIMIT_RADIANS, PITCH_LIMIT_RADIANS);

    let target_rotation = Quat::from_euler(EulerRot::YXZ, yaw, 0.0, 0.0);

    let smoothing_factor = 1.0 - (-ROTATION_SMOOTHING_RATE * dt).exp();
    rotation.0 = rotation.0.slerp(target_rotation, smoothing_factor);
}

fn update_player_velocity(velocity: &mut LinearVelocity, rotation: &Rotation, move_input: Vec2) {
    if move_input == Vec2::ZERO {
        velocity.0.x = 0.0;
        velocity.0.z = 0.0;
        return;
    }

    let (yaw, _, _) = rotation.0.to_euler(EulerRot::YXZ);
    let yaw_rotation = Quat::from_rotation_y(yaw);

    let input_direction = Vec3::new(move_input.x, 0.0, -move_input.y);
    let world_direction = yaw_rotation * input_direction;

    velocity.0.x = world_direction.x * MAX_SPEED;
    velocity.0.z = world_direction.z * MAX_SPEED;
}
