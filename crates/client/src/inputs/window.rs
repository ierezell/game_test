use bevy::app::Update;
use bevy::prelude::{App, IntoScheduleConfigs, MessageReader, Plugin, Query, With, in_state};
use bevy::window::WindowFocused;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};
use leafwing_input_manager::prelude::ActionState;

use crate::ClientGameState;
use shared::inputs::input::PlayerAction;

pub struct ClientWindowPlugin;

impl Plugin for ClientWindowPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (handle_focus).run_if(in_state(ClientGameState::Playing)),
        );
    }
}

pub fn is_cursor_locked(cursor_options_query: &Query<&CursorOptions, With<PrimaryWindow>>) -> bool {
    cursor_options_query
        .single()
        .is_ok_and(|cursor_options| cursor_options.grab_mode == CursorGrabMode::Locked)
}

fn handle_focus(
    mut cursor_options_query: Query<&mut CursorOptions, With<PrimaryWindow>>,
    mut focus_events: MessageReader<WindowFocused>,
    mut player_actions: Query<&mut ActionState<PlayerAction>>,
) {
    for event in focus_events.read() {
        if let Ok(mut cursor_options) = cursor_options_query.single_mut() {
            if event.focused {
                // Window gained focus - grab cursor
                cursor_options.grab_mode = CursorGrabMode::Locked;
                cursor_options.visible = false;
            } else {
                // Window lost focus - release cursor
                cursor_options.grab_mode = CursorGrabMode::None;
                cursor_options.visible = true;
            }
        }

        for mut action_state in &mut player_actions {
            if event.focused {
                action_state.enable();
            } else {
                action_state.disable();
            }

            // Clear all action states after focus transitions.
            action_state.reset_all();
        }
    }
}
