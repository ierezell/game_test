use bevy::prelude::{KeyCode, MouseButton};

use leafwing_input_manager::prelude::{InputMap, MouseMove, VirtualDPad};

use shared::inputs::input::PlayerAction;

pub fn get_player_input_map() -> InputMap<PlayerAction> {
    InputMap::<PlayerAction>::default()
        .with(PlayerAction::Jump, KeyCode::Space)
        .with(PlayerAction::Shoot, MouseButton::Left)
        .with(PlayerAction::Aim, MouseButton::Right)
        .with(PlayerAction::Sprint, KeyCode::ShiftLeft)
        .with(PlayerAction::ToggleFlashlight, KeyCode::KeyF)
        .with_dual_axis(PlayerAction::Move, VirtualDPad::wasd())
        .with_dual_axis(PlayerAction::Move, VirtualDPad::arrow_keys())
        .with_dual_axis(PlayerAction::Look, MouseMove::default())
}
