use avian3d::prelude::{LinearVelocity, Rotation};
use bevy::prelude::{Quat, Reflect, Vec2, Vec3};
use leafwing_input_manager::Actionlike;
use leafwing_input_manager::prelude::ActionState;
use serde::{Deserialize, Serialize};

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
const LOOK_DEADZONE_SQUARED: f32 = 0.000001; // 0.001^2
pub const MOUSE_SENSITIVITY: f32 = 0.0025;
const MOVEMENT_DEADZONE_SQUARED: f32 = 0.000001;
pub const PITCH_LIMIT_RADIANS: f32 = std::f32::consts::FRAC_PI_2 - 0.01;
pub const ROTATION_SMOOTHING_RATE: f32 = 25.0; // Higher = more responsive

pub fn shared_player_movement(
    action_state: &ActionState<PlayerAction>,
    rotation: &mut Rotation,
    velocity: &mut LinearVelocity,
) {
    let move_input = get_movement_input(action_state);

    if let Some(mouse_delta) = get_look_input(action_state) {
        update_player_rotation(rotation, mouse_delta);
    }

    update_player_velocity(velocity, rotation, move_input);
}

pub fn shared_player_movement_with_stamina(
    action_state: &ActionState<PlayerAction>,
    rotation: &mut Rotation,
    velocity: &mut LinearVelocity,
) {
    let move_input = get_movement_input(action_state);

    if let Some(mouse_delta) = get_look_input(action_state) {
        update_player_rotation(rotation, mouse_delta);
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

fn update_player_rotation(rotation: &mut Rotation, mouse_delta: Vec2) {
    let yaw_delta = -mouse_delta.x * MOUSE_SENSITIVITY;
    let yaw_delta_quat = Quat::from_rotation_y(yaw_delta);
    rotation.0 = yaw_delta_quat * rotation.0;
    rotation.0 = rotation.0.normalize();
}

fn update_player_velocity(velocity: &mut LinearVelocity, rotation: &Rotation, move_input: Vec2) {
    let yaw_rotation = rotation.0;
    let input_direction = Vec3::new(move_input.x, 0.0, -move_input.y);
    let world_direction = yaw_rotation * input_direction;
    let desired_velocity = world_direction * MAX_SPEED;
    velocity.0 = Vec3::new(desired_velocity.x, velocity.0.y, desired_velocity.z);
}
