use avian3d::prelude::{Position, Rotation};
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, PrimaryWindow};
use lightyear::prelude::*;

use crate::app::LocalPlayerId;
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
    trigger: Trigger<OnAdd, (PlayerId, Predicted)>,
    mut player_query: Query<(&PlayerId, &Position)>,
    camera_query: Query<Entity, With<PlayerCamera>>,
    mut commands: Commands,
    local_player_id: Res<LocalPlayerId>,
) {
    let entity = trigger.target();

    info!("🚀 Player SPAWN OBSERVER FIRED! Entity: {:?}", entity);

    if let Ok((player_id, position)) = player_query.get_mut(entity) {
        let is_local_player = player_id.0.to_bits() == local_player_id.0;
        debug!(
            "Camera spawn: entity={:?}, is_local_player={}, player_id={:?}, local_player_id={:?}",
            entity, is_local_player, player_id.0, local_player_id.0
        );
        // Only add camera to the local player
        if is_local_player {
            if !camera_query.is_empty() {
                debug!("Camera already exists for local player");
                return;
            }
            let camera_height = PLAYER_CAPSULE_HEIGHT / 2.0 + 0.6; // Player center + eye height offset
            let camera_position = position.0 + Vec3::new(0.0, camera_height, 0.0); // Eye height offset

            commands.spawn((
                PlayerCamera,
                Camera3d::default(),
                Transform::from_translation(camera_position),
                Name::new(format!("Client_{}_Camera", local_player_id.0)),
            ));
            info!("🎥 ADDED Camera to LOCAL predicted player: {:?}", entity);
        }
    } else {
        error!("❌ FAILED TO GET PLAYER COMPONENTS for entity {:?}", entity);
    }
}

/// Update camera position to follow local player
fn update_camera_transform_from_player(
    player_query: Query<
        (&PlayerId, &Position, &Rotation),
        (With<PlayerId>, Or<(Changed<Position>, Changed<Rotation>)>),
    >,
    mut camera_query: Query<&mut Transform, With<PlayerCamera>>,
    local_player_id: Res<LocalPlayerId>,
) {
    let Ok(mut camera_transform) = camera_query.single_mut() else {
        debug!("No FirstPersonCamera entity found");
        return;
    };

    // Find local player and update camera position and rotation
    let mut found = false;
    for (player_id, player_position, player_rotation) in player_query.iter() {
        if player_id.0.to_bits() == local_player_id.0 {
            let camera_height = PLAYER_CAPSULE_HEIGHT / 2.0 + 0.6; // Player center + eye height offset
            let new_camera_position = player_position.0 + Vec3::new(0.0, camera_height, 0.0);
            camera_transform.translation = new_camera_position;
            camera_transform.rotation = player_rotation.0;
            info!(
                "🎥 Camera following player - Player pos: {:?}, Camera pos: {:?}, Rotation: {:?}",
                player_position.0, new_camera_position, player_rotation.0
            );
            found = true;
            break;
        }
    }
    if !found {
        debug!("No local player found for camera to follow");
    }
}

/// Toggle cursor grab with Escape key
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
