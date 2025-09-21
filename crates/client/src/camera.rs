use avian3d::prelude::{Position, Rotation};
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, PrimaryWindow};
use lightyear::prelude::*;

use shared::input::PLAYER_CAPSULE_HEIGHT;
use shared::protocol::PlayerId;

#[derive(Component)]
pub struct PlayerCamera;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(grab_cursor);
        app.add_observer(spawn_camera_when_player_spawn);
        app.add_systems(
            Update,
            (update_camera_transform_from_player, toggle_cursor_grab),
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
    trigger: Trigger<OnAdd, (PlayerId, Predicted)>,
    player_query: Query<(&PlayerId, &Position), (With<Predicted>, With<Controlled>)>,
    mut commands: Commands,
    local_player_id: Res<crate::app::LocalPlayerId>,
) {
    let entity = trigger.target();
    if let Ok((player_id, position)) = player_query.single() {
        // Only spawn camera if this is the local player
        if player_id.0.to_bits() == local_player_id.0 {
            let camera_height = PLAYER_CAPSULE_HEIGHT / 2.0 + 0.6; // Player center + eye height offset
            let camera_position = position.0 + Vec3::new(0.0, camera_height, 0.0); // Eye height offset

            commands.spawn((
                PlayerCamera,
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
    mut camera_query: Query<&mut Transform, With<PlayerCamera>>,
) {
    let Ok(mut camera_transform) = camera_query.single_mut() else {
        info!("No player camera found to update");
        return;
    };

    // Find local player and update camera position and rotation
    let Ok((player_position, player_rotation)) = player_query.single() else {
        info!("No local player found to follow for camera");
        return;
    };

    let camera_height = PLAYER_CAPSULE_HEIGHT / 2.0 + 0.6; // Player center + eye height offset
    let new_camera_position = player_position.0 + Vec3::new(0.0, camera_height, 0.0);
    camera_transform.translation = new_camera_position;

    let (player_yaw, player_pitch, _) = player_rotation.0.to_euler(EulerRot::YXZ);
    let camera_quat = Quat::from_euler(EulerRot::YXZ, player_yaw, player_pitch, 0.0);
    camera_transform.rotation = camera_quat;
}

fn toggle_cursor_grab(
    keys: Res<ButtonInput<KeyCode>>,
    mut window_query: Query<&mut Window, With<PrimaryWindow>>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        if let Ok(mut window) = window_query.single_mut() {
            match window.cursor_options.grab_mode {
                CursorGrabMode::None => {
                    window.cursor_options.grab_mode = CursorGrabMode::Locked;
                    window.cursor_options.visible = false;
                    info!("🔒 Cursor locked");
                }
                _ => {
                    window.cursor_options.grab_mode = CursorGrabMode::None;
                    window.cursor_options.visible = true;
                    info!("🔓 Cursor released");
                }
            }
        }
    }
}
