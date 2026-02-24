use avian3d::prelude::Rotation;
use bevy::math::Vec2;
use bevy::prelude::{
    Add, ButtonInput, Commands, KeyCode, MessageReader, MouseButton, On, Quat, Query, Res,
    ResMut, Resource, With,
};

use bevy::window::WindowFocused;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow, Window};
use leafwing_input_manager::prelude::ActionState;

use crate::inputs::input::get_player_input_map;
use lightyear::prelude::Controlled;

use shared::inputs::input::PlayerAction;

use shared::protocol::PlayerId;

const LOOK_SUPPRESS_FRAMES_AFTER_CAPTURE: u8 = 2;

#[derive(Resource, Debug)]
pub struct InputCaptureState {
    pub captured: bool,
    pub window_focused: bool,
    pub suppress_look_frames: u8,
    pub frozen_rotation: Option<Quat>,
}

impl Default for InputCaptureState {
    fn default() -> Self {
        Self {
            captured: true,
            window_focused: true,
            suppress_look_frames: 0,
            frozen_rotation: None,
        }
    }
}

pub fn is_cursor_locked(cursor_options_query: &Query<&CursorOptions, With<PrimaryWindow>>) -> bool {
    cursor_options_query
        .single()
        .is_ok_and(|cursor_options| cursor_options.grab_mode == CursorGrabMode::Locked)
}

pub fn enforce_playing_cursor_and_input(
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut capture_state: ResMut<InputCaptureState>,
    mut action_query: Query<&mut ActionState<PlayerAction>, (With<PlayerId>, With<Controlled>)>,
    rotation_query: Query<&Rotation, (With<PlayerId>, With<Controlled>)>,
    mut cursor_options_query: Query<(&Window, &mut CursorOptions), With<PrimaryWindow>>,
) {
    let Ok((window, mut cursor_options)) = cursor_options_query.single_mut() else {
        return;
    };

    capture_state.window_focused = window.focused;
    let was_captured = capture_state.captured;
    let should_capture = capture_state.window_focused;

    if should_capture != was_captured {
        for mut action_state in &mut action_query {
            hard_reset_action_state(&mut action_state, should_capture);
        }

        if should_capture {
            capture_state.suppress_look_frames = LOOK_SUPPRESS_FRAMES_AFTER_CAPTURE;
            capture_state.frozen_rotation = None;
        } else {
            capture_state.frozen_rotation = rotation_query.iter().next().map(|rotation| rotation.0);
        }
    }

    apply_cursor_capture(&mut cursor_options, should_capture);

    capture_state.captured = should_capture;

    if should_capture {
        for mut action_state in &mut action_query {
            if action_state.disabled() {
                action_state.enable();
            }

            reconcile_with_physical_inputs(&mut action_state, &keys, &mouse);
        }
    } else {
        for mut action_state in &mut action_query {
            hard_reset_action_state(&mut action_state, false);
        }
    }

    if capture_state.suppress_look_frames > 0 {
        for mut action_state in &mut action_query {
            action_state.set_axis_pair(&PlayerAction::Look, Vec2::ZERO);
        }
        capture_state.suppress_look_frames = capture_state.suppress_look_frames.saturating_sub(1);
    }
}

pub fn enforce_lobby_cursor_and_input(
    mut capture_state: ResMut<InputCaptureState>,
    mut action_query: Query<&mut ActionState<PlayerAction>, (With<PlayerId>, With<Controlled>)>,
    mut cursor_options_query: Query<&mut CursorOptions, With<PrimaryWindow>>,
) {
    let Ok(mut cursor_options) = cursor_options_query.single_mut() else {
        return;
    };

    apply_cursor_capture(&mut cursor_options, false);
    capture_state.captured = false;
    capture_state.frozen_rotation = None;
    for mut action_state in &mut action_query {
        hard_reset_action_state(&mut action_state, false);
    }
}

pub fn handle_focus_change(
    mut focus_events: MessageReader<WindowFocused>,
    mut capture_state: ResMut<InputCaptureState>,
    mut cursor_options_query: Query<&mut CursorOptions, With<PrimaryWindow>>,

    mut action_query: Query<&mut ActionState<PlayerAction>, (With<PlayerId>, With<Controlled>)>,
    rotation_query: Query<&Rotation, (With<PlayerId>, With<Controlled>)>,
) {
    let Ok(mut cursor_options) = cursor_options_query.single_mut() else {
        return;
    };

    for event in focus_events.read() {
        capture_state.window_focused = event.focused;

        if event.focused {
            let should_capture = true;
            apply_cursor_capture(&mut cursor_options, should_capture);
            for mut action_state in &mut action_query {
                hard_reset_action_state(&mut action_state, should_capture);
            }

            capture_state.captured = should_capture;
            if should_capture {
                capture_state.suppress_look_frames = LOOK_SUPPRESS_FRAMES_AFTER_CAPTURE;
                capture_state.frozen_rotation = None;
            }
        } else {
            apply_cursor_capture(&mut cursor_options, false);
            for mut action_state in &mut action_query {
                hard_reset_action_state(&mut action_state, false);
            }

            capture_state.captured = false;
            capture_state.suppress_look_frames = 0;
            capture_state.frozen_rotation = rotation_query.iter().next().map(|rotation| rotation.0);
        }
    }
}

pub fn sync_window_focus_state(
    mut capture_state: ResMut<InputCaptureState>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let Ok(window) = window_query.single() else {
        return;
    };

    capture_state.window_focused = window.focused;
    if !capture_state.window_focused {
        capture_state.captured = false;
    }
}

pub fn freeze_local_rotation_while_unlocked(
    mut capture_state: ResMut<InputCaptureState>,
    mut rotation_query: Query<&mut Rotation, (With<PlayerId>, With<Controlled>)>,
) {
    if capture_state.captured {
        capture_state.frozen_rotation = None;
        return;
    }

    for mut rotation in &mut rotation_query {
        let frozen_rotation = capture_state.frozen_rotation.get_or_insert(rotation.0);
        rotation.0 = *frozen_rotation;
    }
}

pub fn enforce_uncaptured_hard_lock(
    capture_state: Res<InputCaptureState>,
    mut action_query: Query<&mut ActionState<PlayerAction>, (With<PlayerId>, With<Controlled>)>,
    mut rotation_query: Query<&mut Rotation, (With<PlayerId>, With<Controlled>)>,
) {
    if capture_state.captured {
        return;
    }

    for mut action_state in &mut action_query {
        hard_reset_action_state(&mut action_state, false);
    }

    if let Some(frozen_rotation) = capture_state.frozen_rotation {
        for mut rotation in &mut rotation_query {
            rotation.0 = frozen_rotation;
        }
    }
}

pub fn grab_cursor(
    trigger: On<Add, Controlled>,
    mut commands: Commands,
    mut capture_state: ResMut<InputCaptureState>,
    mut cursor_options_query: Query<&mut CursorOptions, With<PrimaryWindow>>,
    mut action_query: Query<&mut ActionState<PlayerAction>, (With<PlayerId>, With<Controlled>)>,
) {
    if let Ok(mut cursor_options) = cursor_options_query.single_mut() {
        cursor_options.grab_mode = CursorGrabMode::Locked;
        cursor_options.visible = false;
    }

    let controlled_entity = trigger.entity;

    match action_query.get_mut(controlled_entity) {
        Ok(mut action_state) => {
            action_state.enable();
        }
        Err(_) => {
            let input_map = get_player_input_map();
            let mut action_state = ActionState::<PlayerAction>::default();
            action_state.enable();
            commands
                .entity(controlled_entity)
                .insert((input_map, action_state));
        }
    }

    capture_state.captured = true;
    capture_state.suppress_look_frames = LOOK_SUPPRESS_FRAMES_AFTER_CAPTURE;
    capture_state.frozen_rotation = None;
}

fn apply_cursor_capture(cursor_options: &mut CursorOptions, capture: bool) {
    cursor_options.grab_mode = if capture {
        CursorGrabMode::Locked
    } else {
        CursorGrabMode::None
    };
    cursor_options.visible = !capture;
}

fn hard_reset_action_state(action_state: &mut ActionState<PlayerAction>, enabled: bool) {
    action_state.set_axis_pair(&PlayerAction::Move, Vec2::ZERO);
    action_state.set_axis_pair(&PlayerAction::Look, Vec2::ZERO);
    action_state.release(&PlayerAction::Jump);
    action_state.release(&PlayerAction::Sprint);
    action_state.release(&PlayerAction::Shoot);
    action_state.release(&PlayerAction::Aim);
    action_state.release(&PlayerAction::Reload);
    action_state.release(&PlayerAction::ToggleFlashlight);

    if enabled {
        action_state.enable();
    } else {
        action_state.disable();
    }
}

fn reconcile_with_physical_inputs(
    action_state: &mut ActionState<PlayerAction>,
    keys: &ButtonInput<KeyCode>,
    mouse: &ButtonInput<MouseButton>,
) {
    let move_pressed = keys.pressed(KeyCode::KeyW)
        || keys.pressed(KeyCode::KeyA)
        || keys.pressed(KeyCode::KeyS)
        || keys.pressed(KeyCode::KeyD)
        || keys.pressed(KeyCode::ArrowUp)
        || keys.pressed(KeyCode::ArrowLeft)
        || keys.pressed(KeyCode::ArrowDown)
        || keys.pressed(KeyCode::ArrowRight);

    if !move_pressed {
        action_state.set_axis_pair(&PlayerAction::Move, Vec2::ZERO);
    }

    if !keys.pressed(KeyCode::Space) {
        action_state.release(&PlayerAction::Jump);
    }
    if !keys.pressed(KeyCode::ShiftLeft) {
        action_state.release(&PlayerAction::Sprint);
    }
    if !keys.pressed(KeyCode::KeyR) {
        action_state.release(&PlayerAction::Reload);
    }
    if !keys.pressed(KeyCode::KeyF) {
        action_state.release(&PlayerAction::ToggleFlashlight);
    }
    if !mouse.pressed(MouseButton::Left) {
        action_state.release(&PlayerAction::Shoot);
    }
    if !mouse.pressed(MouseButton::Right) {
        action_state.release(&PlayerAction::Aim);
    }
}
