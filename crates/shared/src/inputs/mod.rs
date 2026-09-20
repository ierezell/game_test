use bevy::prelude::{
    Bundle, Component, GamepadAxis, GamepadButton, KeyCode, MouseButton, Plugin, Reflect, Vec2,
};
pub use bevy_enhanced_input::action::relationship::Actions;
use bevy_enhanced_input::prelude::*;
use serde::{Deserialize, Serialize};

pub mod look;
pub mod movement;

#[derive(Component, Reflect, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PlayerActions;

#[derive(InputAction)]
#[action_output(Vec2)]
pub struct Move;

#[derive(InputAction)]
#[action_output(Vec2)]
pub struct Look;

#[derive(InputAction)]
#[action_output(bool)]
pub struct Jump;

#[derive(InputAction)]
#[action_output(bool)]
pub struct Sprint;

#[derive(InputAction)]
#[action_output(bool)]
pub struct Shoot;

#[derive(InputAction)]
#[action_output(bool)]
pub struct Aim;

#[derive(InputAction)]
#[action_output(bool)]
pub struct Reload;

#[derive(InputAction)]
#[action_output(bool)]
pub struct ToggleFlashlight;

pub const PLAYER_CAPSULE_RADIUS: f32 = 0.5;
pub const PLAYER_CAPSULE_HEIGHT: f32 = 1.5;
pub const PITCH_LIMIT_RADIANS: f32 = std::f32::consts::FRAC_PI_2 - 0.01;

pub struct SharedInputPlugin;

impl Plugin for SharedInputPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins(EnhancedInputPlugin)
            .add_input_context::<PlayerActions>();
    }
}
pub fn get_player_actions() -> impl Bundle {
    (
        PlayerActions,
        actions!(PlayerActions[
            (Action::<Move>::new(), bindings![
                (KeyCode::KeyW, SwizzleAxis::YXZ),
                (KeyCode::KeyA, Negate::all()),
                (KeyCode::KeyS, Negate::all(), SwizzleAxis::YXZ),
                KeyCode::KeyD,
                GamepadAxis::LeftStickX,
                (GamepadAxis::LeftStickY, SwizzleAxis::YXZ),
            ]),
            (Action::<Look>::new(), bindings![
                Binding::mouse_motion(),
                GamepadAxis::RightStickX,
                (GamepadAxis::RightStickY, SwizzleAxis::YXZ),
            ]),
            (Action::<Jump>::new(), bindings![KeyCode::Space, GamepadButton::South]),
            (Action::<Sprint>::new(), bindings![KeyCode::ShiftLeft, GamepadButton::LeftTrigger2]),
            (Action::<Shoot>::new(), bindings![MouseButton::Left, GamepadButton::RightTrigger2]),
            (Action::<Aim>::new(), bindings![MouseButton::Right, GamepadButton::LeftTrigger2]),
            (Action::<Reload>::new(), bindings![KeyCode::KeyR]),
            (Action::<ToggleFlashlight>::new(), bindings![KeyCode::KeyF]),
        ]),
    )
}
