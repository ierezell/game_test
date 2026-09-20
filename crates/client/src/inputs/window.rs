use bevy::prelude::{
    App, ButtonInput, IntoScheduleConfigs, KeyCode, MessageReader, MouseButton, OnEnter, OnExit,
    Plugin, Query, Res, Update, With, in_state,
};
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow, WindowFocused};

use crate::ClientGameState;

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
) {
    if let Ok(mut cursor_options) = cursor_options_query.single_mut() {
        cursor_options.grab_mode = CursorGrabMode::Locked;
        cursor_options.visible = false;
    }
}

fn release_cursor_after_gameplay(
    mut cursor_options_query: Query<&mut CursorOptions, With<PrimaryWindow>>,
) {
    if let Ok(mut cursor_options) = cursor_options_query.single_mut() {
        cursor_options.grab_mode = CursorGrabMode::None;
        cursor_options.visible = true;
    }
}

fn handle_cursor_capture_hotkeys(
    keys: Res<ButtonInput<KeyCode>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut cursor_options_query: Query<&mut CursorOptions, With<PrimaryWindow>>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        if let Ok(mut cursor_options) = cursor_options_query.single_mut() {
            cursor_options.grab_mode = CursorGrabMode::None;
            cursor_options.visible = true;
        }
        return;
    }

    if mouse_buttons.just_pressed(MouseButton::Left)
        && let Ok(mut cursor_options) = cursor_options_query.single_mut()
        && cursor_options.grab_mode != CursorGrabMode::Locked
    {
        cursor_options.grab_mode = CursorGrabMode::Locked;
        cursor_options.visible = false;
    }
}

fn handle_focus(
    mut cursor_options_query: Query<&mut CursorOptions, With<PrimaryWindow>>,
    mut focus_events: MessageReader<WindowFocused>,
) {
    for event in focus_events.read() {
        if let Ok(mut cursor_options) = cursor_options_query.single_mut() {
            if event.focused {
                // Restore previous capture state
            } else {
                cursor_options.grab_mode = CursorGrabMode::None;
                cursor_options.visible = true;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{capture_cursor_for_gameplay, release_cursor_after_gameplay};
    use bevy::prelude::{App, MinimalPlugins, Update, With};
    use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};

    fn capture_inputs(
        cursor_options: bevy::prelude::Query<&mut CursorOptions, With<PrimaryWindow>>,
    ) {
        capture_cursor_for_gameplay(cursor_options);
    }

    fn release_inputs(
        cursor_options: bevy::prelude::Query<&mut CursorOptions, With<PrimaryWindow>>,
    ) {
        release_cursor_after_gameplay(cursor_options);
    }

    #[test]
    fn cursor_capture_state_toggles_visibility_and_lock() {
        use bevy::prelude::default;

        let mut capture_app = App::new();
        capture_app.add_plugins(MinimalPlugins);
        let entity = capture_app
            .world_mut()
            .spawn((CursorOptions::default(), PrimaryWindow::default()))
            .id();
        capture_app.add_systems(Update, capture_inputs);
        capture_app.update();
        let cursor_options = capture_app
            .world()
            .get::<CursorOptions>(entity)
            .expect("Should have cursor options");
        assert_eq!(cursor_options.grab_mode, CursorGrabMode::Locked);
        assert!(!cursor_options.visible, "Captured cursor should be hidden");

        let mut release_app = App::new();
        release_app.add_plugins(MinimalPlugins);
        let release_entity = release_app
            .world_mut()
            .spawn((
                CursorOptions {
                    grab_mode: CursorGrabMode::Locked,
                    visible: false,
                    ..default()
                },
                PrimaryWindow::default(),
            ))
            .id();
        release_app.add_systems(Update, release_inputs);
        release_app.update();
        let cursor_options = release_app
            .world()
            .get::<CursorOptions>(release_entity)
            .expect("Should have cursor options");
        assert_eq!(cursor_options.grab_mode, CursorGrabMode::None);
        assert!(cursor_options.visible, "Released cursor should be visible");
    }
}
