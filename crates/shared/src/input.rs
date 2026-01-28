use bevy::prelude::Reflect;
use leafwing_input_manager::Actionlike;
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
    Sprint,

    #[actionlike(Button)]
    Shoot,

    #[actionlike(Button)]
    Aim,

    #[actionlike(Button)]
    ToggleFlashlight,
}

/// Constants for player physics and input
pub const PLAYER_CAPSULE_RADIUS: f32 = 0.5;
pub const PLAYER_CAPSULE_HEIGHT: f32 = 1.5;
