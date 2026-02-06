use avian3d::prelude::Rotation;
use bevy::prelude::{EulerRot, Quat, Query, Single, Vec2, With};
use leafwing_input_manager::prelude::ActionState;

use crate::{inputs::input::PlayerAction, protocol::CharacterMarker};
const LOOK_DEADZONE_SQUARED: f32 = 0.000001;
pub const MOUSE_SENSIVITY: f32 = 0.002;

pub fn get_mouse_look_delta(action_state: &ActionState<PlayerAction>) -> Vec2 {
    let look_input = action_state.axis_pair(&PlayerAction::Look);
    if look_input.length_squared() < LOOK_DEADZONE_SQUARED {
        Vec2::ZERO
    } else {
        look_input
    }
}

pub fn update_player_rotation_from_input(
    action_state: Single<&ActionState<PlayerAction>>,
    mut player_rotation: Query<&mut Rotation, With<CharacterMarker>>,
) {
    let action_state = action_state.into_inner();
    if !action_state.disabled() {
        let mouse_delta = get_mouse_look_delta(action_state);
        if mouse_delta != Vec2::ZERO {
            let yaw = -mouse_delta.x * MOUSE_SENSIVITY;
            let pitch = -mouse_delta.y * MOUSE_SENSIVITY;
            if let Ok(mut rotation) = player_rotation.single_mut() {
                *rotation = Rotation::from(Quat::from_euler(EulerRot::YXZ, yaw, pitch, 0.0));
            }
        }
    }
}
