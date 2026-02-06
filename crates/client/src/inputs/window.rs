use bevy::prelude::{
    Add, ButtonInput, Commands, KeyCode, MessageReader, MouseButton, On, Query, Res, With,
};

use bevy::window::WindowFocused;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};
use leafwing_input_manager::prelude::ActionState;

use crate::inputs::input::get_player_input_map;
use lightyear::prelude::{Controlled, Predicted};

use shared::inputs::input::PlayerAction;

use shared::protocol::PlayerId;

pub fn is_cursor_locked(cursor_options_query: &Query<&CursorOptions, With<PrimaryWindow>>) -> bool {
    cursor_options_query
        .single()
        .is_ok_and(|cursor_options| cursor_options.grab_mode == CursorGrabMode::Locked)
}

pub fn toggle_cursor_grab(
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut action_query: Query<
        &mut ActionState<PlayerAction>,
        (With<PlayerId>, With<Predicted>, With<Controlled>),
    >,
    mut cursor_options_query: Query<&mut CursorOptions, With<PrimaryWindow>>,
) {
    let Ok(mut cursor_options) = cursor_options_query.single_mut() else {
        return;
    };

    // Toggle with Escape key
    if keys.just_pressed(KeyCode::Escape) {
        match cursor_options.grab_mode {
            CursorGrabMode::None => {
                cursor_options.grab_mode = CursorGrabMode::Locked;
                cursor_options.visible = false;
                if let Ok(mut action_state) = action_query.single_mut() {
                    // Only reset movement inputs, preserve camera look to avoid camera jump
                    action_state.set_axis_pair(&PlayerAction::Move, bevy::math::Vec2::ZERO);
                    action_state.release(&PlayerAction::Jump);
                    action_state.release(&PlayerAction::Sprint);
                    action_state.release(&PlayerAction::Shoot);
                    action_state.release(&PlayerAction::Aim);
                    action_state.enable();
                }
            }
            _ => {
                cursor_options.grab_mode = CursorGrabMode::None;
                cursor_options.visible = true;
                if let Ok(mut action_state) = action_query.single_mut() {
                    // Only reset movement inputs, preserve camera look to avoid camera jump
                    action_state.set_axis_pair(&PlayerAction::Move, bevy::math::Vec2::ZERO);
                    action_state.release(&PlayerAction::Jump);
                    action_state.release(&PlayerAction::Sprint);
                    action_state.release(&PlayerAction::Shoot);
                    action_state.release(&PlayerAction::Aim);
                    action_state.disable();
                }
            }
        }
    }

    // Re-lock cursor on mouse click when unlocked
    if cursor_options.grab_mode == CursorGrabMode::None && mouse.just_pressed(MouseButton::Left) {
        cursor_options.grab_mode = CursorGrabMode::Locked;
        cursor_options.visible = false;
        if let Ok(mut action_state) = action_query.single_mut() {
            action_state.set_axis_pair(&PlayerAction::Move, bevy::math::Vec2::ZERO);
            action_state.release(&PlayerAction::Jump);
            action_state.release(&PlayerAction::Sprint);
            action_state.release(&PlayerAction::Shoot);
            action_state.release(&PlayerAction::Aim);
            action_state.enable();
        }
    }
}

pub fn handle_focus_change(
    mut focus_events: MessageReader<WindowFocused>,
    mut action_query: Query<
        &mut ActionState<PlayerAction>,
        (With<PlayerId>, With<Predicted>, With<Controlled>),
    >,
) {
    for event in focus_events.read() {
        let Ok(mut action_state) = action_query.single_mut() else {
            continue;
        };

        if event.focused {
            // Reset movement inputs before re-enabling to prevent stuck keys
            // Preserve Look action to avoid camera jump on focus regain
            action_state.set_axis_pair(&PlayerAction::Move, bevy::math::Vec2::ZERO);
            action_state.release(&PlayerAction::Jump);
            action_state.release(&PlayerAction::Sprint);
            action_state.release(&PlayerAction::Shoot);
            action_state.release(&PlayerAction::Aim);

            // DON'T auto-lock cursor on focus - let user press Escape to toggle manually
            // This prevents cursor from being stuck when alt-tabbing or clicking away

            if action_state.disabled() {
                action_state.enable();
            }
        } else {
            // Window lost focus - reset movement inputs to prevent stuck keys
            // Preserve Look action to avoid camera jump on focus loss
            action_state.set_axis_pair(&PlayerAction::Move, bevy::math::Vec2::ZERO);
            action_state.release(&PlayerAction::Jump);
            action_state.release(&PlayerAction::Sprint);
            action_state.release(&PlayerAction::Shoot);
            action_state.release(&PlayerAction::Aim);
        }
    }
}

pub fn grab_cursor(
    trigger: On<Add, Controlled>,
    mut commands: Commands,
    mut cursor_options_query: Query<&mut CursorOptions, With<PrimaryWindow>>,
    mut action_query: Query<
        &mut ActionState<PlayerAction>,
        (With<PlayerId>, With<Predicted>, With<Controlled>),
    >,
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
}
