use bevy::app::Update;
use bevy::prelude::{
    App, ButtonInput, IntoScheduleConfigs, KeyCode, MessageReader, MouseButton, OnEnter, OnExit,
    Plugin, Query, Res, With, in_state,
};
use bevy::window::WindowFocused;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};
use leafwing_input_manager::prelude::{ActionState, InputMap};

use crate::ClientGameState;
use crate::inputs::input_map::get_player_input_map;
use shared::inputs::input::PlayerAction;

pub struct ClientWindowPlugin;

impl Plugin for ClientWindowPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(ClientGameState::Playing),
            capture_cursor_for_gameplay,
        );
        app.add_systems(
            OnExit(ClientGameState::Playing),
            release_cursor_after_gameplay,
        );
        app.add_systems(
            Update,
            (handle_focus, handle_cursor_capture_hotkeys)
                .chain()
                .run_if(in_state(ClientGameState::Playing)),
        );
    }
}

pub fn is_cursor_locked(cursor_options_query: &Query<&CursorOptions, With<PrimaryWindow>>) -> bool {
    cursor_options_query
        .single()
        .is_ok_and(|cursor_options| cursor_options.grab_mode == CursorGrabMode::Locked)
}

fn capture_cursor_for_gameplay(
    mut cursor_options_query: Query<&mut CursorOptions, With<PrimaryWindow>>,
    mut player_inputs: Query<(&mut ActionState<PlayerAction>, &mut InputMap<PlayerAction>)>,
) {
    apply_capture_state(&mut cursor_options_query, &mut player_inputs, true);
}

fn release_cursor_after_gameplay(
    mut cursor_options_query: Query<&mut CursorOptions, With<PrimaryWindow>>,
    mut player_inputs: Query<(&mut ActionState<PlayerAction>, &mut InputMap<PlayerAction>)>,
) {
    apply_capture_state(&mut cursor_options_query, &mut player_inputs, false);
}

fn set_cursor_capture_state(cursor_options: &mut CursorOptions, captured: bool) {
    if captured {
        cursor_options.grab_mode = CursorGrabMode::Locked;
        cursor_options.visible = false;
    } else {
        cursor_options.grab_mode = CursorGrabMode::None;
        cursor_options.visible = true;
    }
}

fn apply_capture_state(
    cursor_options_query: &mut Query<&mut CursorOptions, With<PrimaryWindow>>,
    player_inputs: &mut Query<(&mut ActionState<PlayerAction>, &mut InputMap<PlayerAction>)>,
    captured: bool,
) {
    if let Ok(mut cursor_options) = cursor_options_query.single_mut() {
        set_cursor_capture_state(&mut cursor_options, captured);
    }

    set_player_input_state(player_inputs, captured);
}

fn set_player_input_state(
    player_inputs: &mut Query<(&mut ActionState<PlayerAction>, &mut InputMap<PlayerAction>)>,
    captured: bool,
) {
    for (mut action_state, mut input_map) in player_inputs.iter_mut() {
        if captured {
            action_state.enable();
            *input_map = get_player_input_map();
        } else {
            action_state.disable();
            *input_map = get_player_input_map();
        }

        action_state.reset_all();
    }
}

fn handle_cursor_capture_hotkeys(
    keys: Res<ButtonInput<KeyCode>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut cursor_options_query: Query<&mut CursorOptions, With<PrimaryWindow>>,
    mut player_inputs: Query<(&mut ActionState<PlayerAction>, &mut InputMap<PlayerAction>)>,
) {
    let mut should_capture = false;
    let should_release = keys.just_pressed(KeyCode::Escape);

    if !should_release
        && mouse_buttons.just_pressed(MouseButton::Left)
        && let Ok(cursor_options) = cursor_options_query.single_mut()
    {
        should_capture = cursor_options.grab_mode != CursorGrabMode::Locked;
    }

    if should_release {
        apply_capture_state(&mut cursor_options_query, &mut player_inputs, false);
    } else if should_capture {
        apply_capture_state(&mut cursor_options_query, &mut player_inputs, true);
    }
}

fn handle_focus(
    mut cursor_options_query: Query<&mut CursorOptions, With<PrimaryWindow>>,
    mut focus_events: MessageReader<WindowFocused>,
    mut player_inputs: Query<(&mut ActionState<PlayerAction>, &mut InputMap<PlayerAction>)>,
) {
    for event in focus_events.read() {
        if event.focused {
            let captured = cursor_options_query
                .single_mut()
                .is_ok_and(|cursor_options| cursor_options.grab_mode == CursorGrabMode::Locked);
            set_player_input_state(&mut player_inputs, captured);
        } else {
            apply_capture_state(&mut cursor_options_query, &mut player_inputs, false);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{set_cursor_capture_state, set_player_input_state};
    use bevy::prelude::{App, MinimalPlugins, Update};
    use bevy::window::{CursorGrabMode, CursorOptions};
    use leafwing_input_manager::prelude::{ActionState, InputMap};
    use shared::inputs::input::PlayerAction;

    fn enable_inputs(
        mut player_inputs: bevy::prelude::Query<(
            &mut ActionState<PlayerAction>,
            &mut InputMap<PlayerAction>,
        )>,
    ) {
        set_player_input_state(&mut player_inputs, true);
    }

    fn disable_inputs(
        mut player_inputs: bevy::prelude::Query<(
            &mut ActionState<PlayerAction>,
            &mut InputMap<PlayerAction>,
        )>,
    ) {
        set_player_input_state(&mut player_inputs, false);
    }

    #[test]
    fn cursor_capture_state_toggles_visibility_and_lock() {
        let mut cursor_options = CursorOptions::default();

        set_cursor_capture_state(&mut cursor_options, true);
        assert_eq!(cursor_options.grab_mode, CursorGrabMode::Locked);
        assert!(!cursor_options.visible, "Captured cursor should be hidden");

        set_cursor_capture_state(&mut cursor_options, false);
        assert_eq!(cursor_options.grab_mode, CursorGrabMode::None);
        assert!(cursor_options.visible, "Released cursor should be visible");
    }

    #[test]
    fn player_input_state_enables_and_disables_actions() {
        let mut enable_app = App::new();
        enable_app.add_plugins(MinimalPlugins);

        let enable_player = enable_app
            .world_mut()
            .spawn((
                ActionState::<PlayerAction>::default(),
                InputMap::<PlayerAction>::default(),
            ))
            .id();

        enable_app.add_systems(Update, enable_inputs);
        enable_app.update();
        let action_state_after_enable = enable_app
            .world()
            .get::<ActionState<PlayerAction>>(enable_player)
            .expect("Player should keep ActionState after enable");
        assert!(
            !action_state_after_enable.disabled(),
            "Input capture should enable player actions"
        );

        let mut disable_app = App::new();
        disable_app.add_plugins(MinimalPlugins);
        let disable_player = disable_app
            .world_mut()
            .spawn((
                ActionState::<PlayerAction>::default(),
                InputMap::<PlayerAction>::default(),
            ))
            .id();

        disable_app.add_systems(Update, disable_inputs);
        disable_app.update();
        let action_state_after_disable = disable_app
            .world()
            .get::<ActionState<PlayerAction>>(disable_player)
            .expect("Player should keep ActionState after disable");
        assert!(
            action_state_after_disable.disabled(),
            "Input release should disable player actions"
        );
    }
}
