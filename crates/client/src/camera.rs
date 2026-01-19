use avian3d::prelude::Position;

use bevy::prelude::{
    Add, App, Camera, Camera2d, Camera3d, Changed, Commands, Component, Entity, EulerRot,
    IntoScheduleConfigs, Name, On, Or, Plugin, PostUpdate, Quat, Query, Res, Startup, Transform,
    Vec3, With, default, in_state,
};

use bevy_inspector_egui::{
    bevy_egui::{EguiGlobalSettings, EguiPlugin, PrimaryEguiContext},
    quick::WorldInspectorPlugin,
};

use lightyear::prelude::{Controlled, Predicted};

use shared::input::PLAYER_CAPSULE_HEIGHT;
use shared::protocol::PlayerId;

use crate::ClientGameState;
pub struct RenderPlugin;

#[derive(Component, Default)]
pub struct CameraPitch(pub f32);

#[derive(Component, Default)]
pub struct PlayerCamera;

#[derive(Component)]
struct DebugCamera;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_menu_and_debug_camera);
        app.insert_resource(EguiGlobalSettings {
            auto_create_primary_context: false,
            ..Default::default()
        });
        app.add_plugins((EguiPlugin::default(), WorldInspectorPlugin::default()));
        app.add_observer(spawn_camera_when_player_spawn);
        app.add_systems(
            PostUpdate,
            update_camera_transform_from_player_controller_rotation
                .run_if(in_state(ClientGameState::Playing)),
        );
    }
}

fn spawn_menu_and_debug_camera(mut commands: Commands) {
    commands.spawn((
        Camera {
            order: 100,
            ..default()
        },
        Camera2d,
        DebugCamera,
        PrimaryEguiContext,
    ));
}

fn spawn_camera_when_player_spawn(
    trigger: On<Add, Controlled>,
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
    let Ok((player_id, position)) = player_query.get(entity) else {
        return;
    };

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
            Name::new(format!("Client_{}_Camera", local_player_id.0)),
        ));
    }
}

fn update_camera_transform_from_player_controller_rotation(
    player_query: Query<
        (&Position, &shared::input::FpsController),
        (
            With<PlayerId>,
            With<Predicted>,
            With<Controlled>,
            Or<(Changed<Position>, Changed<shared::input::FpsController>)>,
        ),
    >,
    mut camera_query: Query<&mut Transform, With<PlayerCamera>>,
) {
    let Ok(mut camera_transform) = camera_query.single_mut() else {
        return;
    };

    let Ok((player_position, fps_controller)) = player_query.single() else {
        return;
    };

    camera_transform.translation = Vec3::new(
        player_position.0.x,
        player_position.0.y + PLAYER_CAPSULE_HEIGHT + 0.6,
        player_position.0.z,
    );

    camera_transform.rotation =
        Quat::from_euler(EulerRot::YXZ, fps_controller.yaw, fps_controller.pitch, 0.0);
}
