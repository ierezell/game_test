use avian3d::prelude::Position;
use avian3d::prelude::Rotation;
use bevy::prelude::{
    Add, App, Camera, Camera2d, Camera3d, Changed, Commands, Component, Entity, EulerRot, Name, On,
    Or, Plugin, PostUpdate, Quat, Query, Res, Startup, Transform, Update, Vec3, With, debug,
    default, info,
};
// Removed unused window imports
use bevy_inspector_egui::{
    bevy_egui::{EguiGlobalSettings, EguiPlugin, PrimaryEguiContext},
    quick::WorldInspectorPlugin,
};
use leafwing_input_manager::prelude::ActionState;
use lightyear::prelude::{Controlled, Predicted};

use shared::input::{MOUSE_SENSITIVITY, PITCH_LIMIT_RADIANS, PLAYER_CAPSULE_HEIGHT, PlayerAction};
use shared::level::create_static::setup_static_level_default;
use shared::protocol::PlayerId;
use shared::render::add_enemy_visuals;
pub struct RenderPlugin;

#[derive(Component, Default)]
pub struct CameraPitch(pub f32);

#[derive(Component, Default)]
pub struct PlayerCamera;

#[derive(Component)]
struct DebugCamera;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (setup_static_level_default, spawn_menu_and_debug_camera));
        app.add_observer(add_enemy_visuals);
        app.insert_resource(EguiGlobalSettings {
            auto_create_primary_context: false,
            ..Default::default()
        });
        app.add_plugins((EguiPlugin::default(), WorldInspectorPlugin::default()));
        app.add_observer(spawn_camera_when_player_spawn);
        app.add_systems(PostUpdate, update_camera_transform_from_player);
        app.add_systems(Update, update_camera_pitch);
    }
}

fn spawn_menu_and_debug_camera(mut commands: Commands) {
    commands.spawn((
        Camera {
            order: 100,
            ..default()
        },
        Camera2d::default(),
        DebugCamera,
        PrimaryEguiContext,
    ));
}

fn spawn_camera_when_player_spawn(
    // Trigger 3 times:
    // Once for (PlayerId, ShouldBePredicted) (When replicated from server)
    // Once when (Predicted) is added alone
    // Once when (PlayerId with Predicted) is added (The one we want)
    trigger: On<Add, (Predicted, Controlled, PlayerId)>,
    player_query: Query<
        (&PlayerId, &Position),
        (With<Predicted>, With<Controlled>, With<PlayerId>),
    >,
    camera_query: Query<Entity, With<PlayerCamera>>,
    mut commands: Commands,
    local_player_id: Res<crate::LocalPlayerId>,
) {
    if !camera_query.is_empty() {
        return;
    }

    let entity = trigger.entity;
    if let Ok((player_id, position)) = player_query.get(entity) {
        // Only spawn camera if this is the local player, is this a hack ??
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
