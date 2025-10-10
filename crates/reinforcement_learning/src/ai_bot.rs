use bevy::prelude::*;
use leafwing_input_manager::prelude::ActionState;
use shared::ai_bot::{AIBot, BotObservation, get_bot_observation};
use shared::input::PlayerAction;
use shared::protocol::PlayerId;

// Re-export external agent control functions from shared crate
pub use shared::ai_bot::{
    set_external_agent_jump, set_external_agent_look, set_external_agent_movement,
    set_external_agent_shoot, spawn_external_agent_bot,
};

/// Get observation data for an external agent bot (RL-specific interface)
pub fn get_external_agent_observation(
    bot_query: Query<
        (
            &AIBot,
            &avian3d::prelude::Position,
            &avian3d::prelude::Rotation,
            &avian3d::prelude::LinearVelocity,
            &shared::health::Health,
            &ActionState<PlayerAction>,
        ),
        With<PlayerId>,
    >,
    bot_id: u32,
) -> Option<BotObservation> {
    get_bot_observation(bot_query, bot_id)
}
