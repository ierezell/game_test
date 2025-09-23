use avian3d::prelude::{Position, Rotation};
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, PrimaryWindow, WindowFocused};
use leafwing_input_manager::prelude::ActionState;
use lightyear::prelude::*;

use shared::input::{MOUSE_SENSITIVITY, PITCH_LIMIT_RADIANS, PLAYER_CAPSULE_HEIGHT, PlayerAction};
use shared::protocol::PlayerId;

#[derive(Component, Default)]
pub struct CameraPitch(pub f32);

#[derive(Component, Default)]
pub struct PlayerCamera;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(grab_cursor);
        app.add_observer(spawn_camera_when_player_spawn);
        app.add_systems(PostUpdate, update_camera_transform_from_player);
        app.add_systems(
            Update,
            (update_camera_pitch, toggle_cursor_grab, handle_focus_change),
        );
    }
}

fn grab_cursor(
    _trigger: Trigger<OnAdd, (PlayerId, Predicted)>,
    mut window_query: Query<&mut Window, With<PrimaryWindow>>,
) {
    if let Ok(mut window) = window_query.single_mut() {
        window.cursor_options.grab_mode = CursorGrabMode::Locked;
        window.cursor_options.visible = false;
        info!("🔒 Initial cursor lock enabled for FPS gameplay");
    }
}

fn spawn_camera_when_player_spawn(
    // Trigger 3 times:
    // Once for (PlayerId, ShouldBePredicted) (When replicated from server)
    // Once when (Predicted) is added alone
    // Once when (PlayerId with Predicted) is added (The one we want)
    trigger: Trigger<OnAdd, (Predicted, Controlled, PlayerId)>,
    player_query: Query<
        (&PlayerId, &Position),
        (With<Predicted>, With<Controlled>, With<PlayerId>),
    >,
    camera_query: Query<Entity, With<PlayerCamera>>,
    mut commands: Commands,
    local_player_id: Res<crate::app::LocalPlayerId>,
) {
    if !camera_query.is_empty() {
        return;
    }

    let entity = trigger.target();
    if let Ok((player_id, position)) = player_query.get(entity) {
        // Only spawn camera if this is the local player
        if player_id.0.to_bits() == local_player_id.0 {
            let camera_height = position.0.y + PLAYER_CAPSULE_HEIGHT + 0.6; // Player center + eye height offset
            let camera_position = position.0 + Vec3::new(0.0, camera_height, 0.0); // Eye height offset

            commands.spawn((
                PlayerCamera,
                CameraPitch::default(),
                Camera {
                    order: 0,
                    ..default()
                },
                Camera3d::default(),
                Transform::from_translation(camera_position),
                Name::new(format!("Client_{}_Camera", player_id.0.to_bits())),
            ));
            info!("🎥 ADDED Camera to LOCAL predicted player: {:?}", entity);
        } else {
            info!(
                "Skipping camera spawn for non-local player: {:?}",
                player_id
            );
        }
    }
}

fn handle_focus_change(
    mut focus_events: EventReader<WindowFocused>,
    mut action_query: Query<
        &mut ActionState<PlayerAction>,
        (With<PlayerId>, With<Predicted>, With<Controlled>),
    >,
    mut window_query: Query<&mut Window, With<PrimaryWindow>>,
) {
    for event in focus_events.read() {
        let Ok(mut window) = window_query.single_mut() else {
            continue;
        };
        let Ok(mut action_state) = action_query.single_mut() else {
            continue;
        };

        if event.focused {
            if window.cursor_options.grab_mode != CursorGrabMode::Locked {
                window.cursor_options.grab_mode = CursorGrabMode::Locked;
                window.cursor_options.visible = false;
                info!("🔒 Cursor locked on focus gain");
            }
            if action_state.disabled() {
                action_state.enable();
                info!("🎮 Inputs enabled on focus gain");
            }
        } else {
            if window.cursor_options.grab_mode != CursorGrabMode::None {
                window.cursor_options.grab_mode = CursorGrabMode::None;
                window.cursor_options.visible = true;
                info!("🔓 Cursor released on focus loss");
            }
            if !action_state.disabled() {
                action_state.disable();
                info!("🎮 Inputs disabled on focus loss");
            }
        }
    }
}

fn update_camera_pitch(
    mut camera_query: Query<&mut CameraPitch, With<PlayerCamera>>,
    action_query: Query<
        &ActionState<PlayerAction>,
        (With<PlayerId>, With<Predicted>, With<Controlled>),
    >,
) {
    let Ok(action_state) = action_query.single() else {
        return;
    };

    let mouse_delta = action_state.axis_pair(&PlayerAction::Look);
    if mouse_delta.y.abs() < 0.001 {
        return;
    }

    let pitch_delta = -mouse_delta.y * MOUSE_SENSITIVITY;

    if let Ok(mut camera_pitch) = camera_query.single_mut() {
        camera_pitch.0 =
            (camera_pitch.0 + pitch_delta).clamp(-PITCH_LIMIT_RADIANS, PITCH_LIMIT_RADIANS);
    }
}

fn update_camera_transform_from_player(
    player_query: Query<
        (&Position, &Rotation),
        (
            With<PlayerId>,
            With<Predicted>,
            With<Controlled>,
            Or<(Changed<Position>, Changed<Rotation>)>,
        ),
    >,
    mut camera_query: Query<(&mut Transform, &CameraPitch), With<PlayerCamera>>,
) {
    let Ok((mut camera_transform, camera_pitch)) = camera_query.single_mut() else {
        debug!("No player camera found to update");
        return;
    };

    // Find local player and update camera position and rotation
    let Ok((player_position, player_rotation)) = player_query.single() else {
        return; // If unlocking cursor, no more changes, Or<(Changed<Position>, Changed<Rotation>)> will not trigger and this query will fail
    };

    camera_transform.translation = Vec3::new(
        player_position.0.x,
        player_position.0.y + PLAYER_CAPSULE_HEIGHT + 0.6,
        player_position.0.z,
    );

    let (player_yaw, _, _) = player_rotation.0.to_euler(EulerRot::YXZ);
    let camera_quat = Quat::from_euler(EulerRot::YXZ, player_yaw, camera_pitch.0, 0.0);
    camera_transform.rotation = camera_quat;
}

fn toggle_cursor_grab(
    keys: Res<ButtonInput<KeyCode>>,
    mut action_query: Query<
        &mut ActionState<PlayerAction>,
        (With<PlayerId>, With<Predicted>, With<Controlled>),
    >,
    mut window_query: Query<&mut Window, With<PrimaryWindow>>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        if let Ok(mut window) = window_query.single_mut() {
            match window.cursor_options.grab_mode {
                CursorGrabMode::None => {
                    window.cursor_options.grab_mode = CursorGrabMode::Locked;
                    window.cursor_options.visible = false;
                    info!("🔒 Cursor locked");
                    if let Ok(mut action_state) = action_query.single_mut() {
                        action_state.reset_all();
                        action_state.enable();
                    }
                }
                _ => {
                    window.cursor_options.grab_mode = CursorGrabMode::None;
                    window.cursor_options.visible = true;
                    info!("🔓 Cursor released");
                    if let Ok(mut action_state) = action_query.single_mut() {
                        action_state.reset_all();
                        action_state.disable();
                    }
                }
            }
        }
    }
}
