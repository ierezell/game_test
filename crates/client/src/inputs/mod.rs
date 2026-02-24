pub mod input;
pub mod window;

use bevy::prelude::{App, IntoScheduleConfigs, Last, Plugin, PostUpdate, PreUpdate};
use bevy::state::condition::in_state;

use crate::inputs::window::{
    InputCaptureState, enforce_lobby_cursor_and_input, enforce_playing_cursor_and_input,
    enforce_uncaptured_hard_lock, freeze_local_rotation_while_unlocked, grab_cursor,
    handle_focus_change, sync_window_focus_state,
};
use crate::ClientGameState;

pub struct ClientInputPlugin;

impl Plugin for ClientInputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InputCaptureState>();

        // Run before shared Update systems so escape/focus changes are applied
        // before look/movement input is consumed for the frame.
        app.add_systems(
            PreUpdate,
            (
                sync_window_focus_state,
                handle_focus_change,
                enforce_playing_cursor_and_input,
                freeze_local_rotation_while_unlocked,
            )
                .chain()
                .run_if(in_state(ClientGameState::Playing)),
        );
        app.add_systems(
            PostUpdate,
            freeze_local_rotation_while_unlocked.run_if(in_state(ClientGameState::Playing)),
        );
        app.add_systems(
            Last,
            enforce_uncaptured_hard_lock.run_if(in_state(ClientGameState::Playing)),
        );
        app.add_systems(
            PreUpdate,
            enforce_lobby_cursor_and_input.run_if(in_state(ClientGameState::Lobby)),
        );
        app.add_observer(grab_cursor);
    }
}
