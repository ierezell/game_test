use avian3d::prelude::{LinearVelocity, Rotation};

use bevy::prelude::{
    Add, App, Entity, FixedUpdate, KeyCode, MouseButton, On, Plugin, Query, Res, Vec2, With, debug,
    info,
};
use bevy::prelude::{ButtonInput, MessageReader, Update};
use bevy::window::WindowFocused;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};
use leafwing_input_manager::prelude::{ActionState, InputMap, MouseMove, VirtualDPad};
use shared::input::shared_player_movement;

use lightyear::prelude::{Controlled, Predicted};

use shared::input::PlayerAction;
use shared::protocol::PlayerId;

pub struct ClientInputPlugin;

impl Plugin for ClientInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, client_player_movement);
        app.add_systems(Update, (toggle_cursor_grab, handle_focus_change));
        app.add_observer(grab_cursor);
    }
}

fn client_player_movement(
    mut player_query: Query<
        (
            Entity,
            &mut Rotation,
            &mut LinearVelocity,
            &ActionState<PlayerAction>,
        ),
        (With<PlayerId>, With<Predicted>, With<Controlled>),
    >,
) {
    for (entity, mut rotation, mut velocity, action_state) in player_query.iter_mut() {
        let move_axis_pair = action_state.axis_pair(&PlayerAction::Move);
        let look_axis_pair = action_state.axis_pair(&PlayerAction::Look);

        if move_axis_pair != Vec2::ZERO
            || !action_state.get_pressed().is_empty()
            || look_axis_pair != Vec2::ZERO
        {
            debug!(
                "🎮 Player input detected: Entity {:?}, Move: {:?}, Look: {:?}",
                entity, move_axis_pair, look_axis_pair
            );
        }

        shared_player_movement(action_state, &mut rotation, &mut velocity);
    }
}

pub fn get_player_input_map() -> InputMap<PlayerAction> {
    let input_map = InputMap::<PlayerAction>::default()
        .with(PlayerAction::Jump, KeyCode::Space)
        .with(PlayerAction::Shoot, MouseButton::Left)
        .with(PlayerAction::Aim, MouseButton::Right)
        .with_dual_axis(PlayerAction::Move, VirtualDPad::wasd())
        .with_dual_axis(PlayerAction::Move, VirtualDPad::arrow_keys())
        .with_dual_axis(PlayerAction::Look, MouseMove::default());

    input_map
}

pub fn is_cursor_locked(cursor_options_query: &Query<&CursorOptions, With<PrimaryWindow>>) -> bool {
    cursor_options_query
        .single()
        .map_or(false, |cursor_options| {
            cursor_options.grab_mode == CursorGrabMode::Locked
        })
}

fn toggle_cursor_grab(
    keys: Res<ButtonInput<KeyCode>>,
    mut action_query: Query<
        &mut ActionState<PlayerAction>,
        (With<PlayerId>, With<Predicted>, With<Controlled>),
    >,
    mut cursor_options_query: Query<&mut CursorOptions, With<PrimaryWindow>>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        if let Ok(mut cursor_options) = cursor_options_query.single_mut() {
            match cursor_options.grab_mode {
                CursorGrabMode::None => {
                    cursor_options.grab_mode = CursorGrabMode::Locked;
                    cursor_options.visible = false;
                    info!("🔒 Cursor locked");
                    if let Ok(mut action_state) = action_query.single_mut() {
                        action_state.reset_all();
                        action_state.enable();
                        info!("🎮 Inputs enabled with cursor lock");
                    }
                }
                _ => {
                    cursor_options.grab_mode = CursorGrabMode::None;
                    cursor_options.visible = true;
                    info!("🔓 Cursor released");
                    if let Ok(mut action_state) = action_query.single_mut() {
                        action_state.reset_all();
                        action_state.disable();
                        info!("🎮 Inputs disabled with cursor release");
                    }
                }
            }
        }
    }
}

fn handle_focus_change(
    mut focus_events: MessageReader<WindowFocused>,
    mut action_query: Query<
        &mut ActionState<PlayerAction>,
        (With<PlayerId>, With<Predicted>, With<Controlled>),
    >,
    mut cursor_options_query: Query<&mut CursorOptions, With<PrimaryWindow>>,
) {
    for event in focus_events.read() {
        let Ok(mut cursor_options) = cursor_options_query.single_mut() else {
            continue;
        };
        let Ok(mut action_state) = action_query.single_mut() else {
            continue;
        };

        if event.focused {
            if cursor_options.grab_mode != CursorGrabMode::Locked {
                cursor_options.grab_mode = CursorGrabMode::Locked;
                cursor_options.visible = false;
                info!("🔒 Cursor locked on focus gain");
            }
            if action_state.disabled() {
                action_state.enable();
                info!("🎮 Inputs enabled on focus gain");
            }
        } else {
            // Don't disable inputs or release cursor on focus loss - keep them active
            info!("🎮 Focus lost but keeping input active for FPS gameplay");
        }
    }
}

fn grab_cursor(
    _trigger: On<Add, Controlled>,
    mut cursor_options_query: Query<&mut CursorOptions, With<PrimaryWindow>>,
    mut action_query: Query<
        &mut ActionState<PlayerAction>,
        (With<PlayerId>, With<Predicted>, With<Controlled>),
    >,
) {
    if let Ok(mut cursor_options) = cursor_options_query.single_mut() {
        cursor_options.grab_mode = CursorGrabMode::Locked;
        cursor_options.visible = false;
        info!("🔒 Initial cursor lock enabled for FPS gameplay");
    }

    if let Ok(mut action_state) = action_query.single_mut() {
        if action_state.disabled() {
            action_state.enable();
            info!("🎮 Initial input enabled for FPS gameplay");
        }
    }
}
