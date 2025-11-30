use bevy::prelude::{
    Add, App, Commands, Entity, FixedUpdate, KeyCode, MouseButton, On, Plugin, Query, Res, Time,
    Transform, With,
};

use avian3d::prelude::{Collider, LinearVelocity, SpatialQueryPipeline};
use bevy::prelude::{ButtonInput, MessageReader, Update};
use bevy::window::WindowFocused;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};
use leafwing_input_manager::prelude::{ActionState, InputMap, MouseMove, VirtualDPad};

use lightyear::prelude::{Controlled, Predicted};

use shared::input::{FpsController, PlayerAction};
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
    time: Res<Time>,
    spatial_query: Res<SpatialQueryPipeline>,
    mut player_query: Query<
        (
            Entity,
            &ActionState<PlayerAction>,
            &mut FpsController,
            &mut Transform,
            &mut LinearVelocity,
            &Collider,
        ),
        (With<PlayerId>, With<Predicted>, With<Controlled>),
    >,
) {
    if let Ok((entity, action_state, mut controller, mut transform, mut velocity, collider)) =
        player_query.single_mut()
    {
        let move_input = action_state.axis_pair(&PlayerAction::Move);
        if move_input.length_squared() > 0.0 {
            println!("Client sending move input: {:?}", move_input);
        }
        shared::input::shared_player_movement(
            *time,
            spatial_query.clone(),
            entity,
            action_state,
            &mut controller,
            &mut transform,
            &mut velocity,
            collider,
        );
    }
}

pub fn get_player_input_map() -> InputMap<PlayerAction> {
    let input_map = InputMap::<PlayerAction>::default()
        .with(PlayerAction::Jump, KeyCode::Space)
        .with(PlayerAction::Shoot, MouseButton::Left)
        .with(PlayerAction::Aim, MouseButton::Right)
        .with(PlayerAction::Sprint, KeyCode::ShiftLeft)
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
                    if let Ok(mut action_state) = action_query.single_mut() {
                        action_state.reset_all();
                        action_state.enable();
                    }
                }
                _ => {
                    cursor_options.grab_mode = CursorGrabMode::None;
                    cursor_options.visible = true;
                    if let Ok(mut action_state) = action_query.single_mut() {
                        action_state.reset_all();
                        action_state.disable();
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
            }
            if action_state.disabled() {
                action_state.enable();
            }
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
