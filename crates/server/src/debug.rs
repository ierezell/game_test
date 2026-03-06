use bevy::prelude::{App, IntoScheduleConfigs, Plugin, Update};
use bevy::state::condition::in_state;

use shared::debug::log_gym_wandering_diagnostics;
use shared::gym::update_gym_wandering_npc_targets;

use crate::ServerGameState;

pub struct ServerDebugPlugin;

impl Plugin for ServerDebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            log_gym_wandering_diagnostics
                .after(update_gym_wandering_npc_targets)
                .run_if(in_state(ServerGameState::Playing)),
        );
    }
}
