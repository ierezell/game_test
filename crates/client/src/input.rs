use bevy::prelude::{
    Add, App, ButtonInput, Commands, FixedUpdate, IntoScheduleConfigs, KeyCode, MessageReader, MouseButton, On, Plugin, Query, Res,
    Update, With,
};

use avian3d::prelude::{Rotation};
use bevy::window::WindowFocused;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};
use leafwing_input_manager::prelude::{ActionState, InputMap, MouseMove, VirtualDPad};

use lightyear::prelude::{Controlled, Predicted};

use shared::input::PlayerAction;
use shared::movement::{PhysicsConfig, update_ground_detection, apply_movement};
use shared::camera::{FpsCamera, update_camera_from_input};
use shared::protocol::PlayerId;

pub struct ClientInputPlugin;

impl Plugin for ClientInputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PhysicsConfig>();
        
        // Movement systems (FixedUpdate for physics)
        app.add_systems(FixedUpdate, (
            update_ground_detection,
            apply_movement,
            update_camera_rotation_client,
        ).chain());
        
        // Camera and input management (Update for responsiveness)
        app.add_systems(Update, (
            update_camera_from_input,
            toggle_cursor_grab,
            handle_focus_change,
            detect_stuck_inputs,
        ));
        
        app.add_observer(grab_cursor);
    }
}

/// Client system: Update entity Rotation from FpsCamera
fn update_camera_rotation_client(
    mut query: Query<
        (&FpsCamera, &mut Rotation),
        (With<PlayerId>, With<Predicted>, With<Controlled>),
    >,
) {
    for (camera, mut rotation) in query.iter_mut() {
        rotation.0 = bevy::prelude::Quat::from_euler(
            bevy::prelude::EulerRot::YXZ,
            camera.yaw,
            0.0,
            0.0,
        );
    }
}

pub fn get_player_input_map() -> InputMap<PlayerAction> {
    InputMap::<PlayerAction>::default()
        .with(PlayerAction::Jump, KeyCode::Space)
        .with(PlayerAction::Shoot, MouseButton::Left)
        .with(PlayerAction::Aim, MouseButton::Right)
        .with(PlayerAction::Sprint, KeyCode::ShiftLeft)
        .with_dual_axis(PlayerAction::Move, VirtualDPad::wasd())
        .with_dual_axis(PlayerAction::Move, VirtualDPad::arrow_keys())
        .with_dual_axis(PlayerAction::Look, MouseMove::default())
}

pub fn is_cursor_locked(cursor_options_query: &Query<&CursorOptions, With<PrimaryWindow>>) -> bool {
    cursor_options_query
        .single()
        .is_ok_and(|cursor_options| cursor_options.grab_mode == CursorGrabMode::Locked)
}

fn toggle_cursor_grab(
    keys: Res<ButtonInput<KeyCode>>,
    mut action_query: Query<
        &mut ActionState<PlayerAction>,
        (With<PlayerId>, With<Predicted>, With<Controlled>),
    >,
    mut cursor_options_query: Query<&mut CursorOptions, With<PrimaryWindow>>,
) {
    if keys.just_pressed(KeyCode::Escape)
        && let Ok(mut cursor_options) = cursor_options_query.single_mut()
    {
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
            // Reset movement inputs before re-enabling to prevent stuck keys
            // Preserve Look action to avoid camera jump on focus regain
            action_state.set_axis_pair(&PlayerAction::Move, bevy::math::Vec2::ZERO);
            action_state.release(&PlayerAction::Jump);
            action_state.release(&PlayerAction::Sprint);
            action_state.release(&PlayerAction::Shoot);
            action_state.release(&PlayerAction::Aim);
            
            if cursor_options.grab_mode != CursorGrabMode::Locked {
                cursor_options.grab_mode = CursorGrabMode::Locked;
                cursor_options.visible = false;
            }
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

fn grab_cursor(
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

/// Detects and clears stuck inputs that might occur from network lag, 
/// focus changes, or input system edge cases
fn detect_stuck_inputs(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut action_query: Query<
        &mut ActionState<PlayerAction>,
        (With<PlayerId>, With<Predicted>, With<Controlled>),
    >,
) {
    let Ok(mut action_state) = action_query.single_mut() else {
        return;
    };

    // If ActionState shows movement but no movement keys are actually pressed, clear it
    let move_input = action_state.axis_pair(&PlayerAction::Move);
    if move_input.length_squared() > 0.0 {
        let wasd_pressed = keyboard.pressed(KeyCode::KeyW)
            || keyboard.pressed(KeyCode::KeyA)
            || keyboard.pressed(KeyCode::KeyS)
            || keyboard.pressed(KeyCode::KeyD);
        let arrows_pressed = keyboard.pressed(KeyCode::ArrowUp)
            || keyboard.pressed(KeyCode::ArrowDown)
            || keyboard.pressed(KeyCode::ArrowLeft)
            || keyboard.pressed(KeyCode::ArrowRight);

        if !wasd_pressed && !arrows_pressed {
            // Input is stuck - clear it
            action_state.set_axis_pair(&PlayerAction::Move, bevy::math::Vec2::ZERO);
        }
    }
}
