use bevy::input::common_conditions::input_toggle_active;
use bevy::prelude::{
    Add, App, Camera, Camera2d, Camera3d, Commands, Component, Entity, KeyCode, Name, On, Plugin,
    Query, Res, Startup, Transform, With, default,
};

use bevy_inspector_egui::{
    bevy_egui::{EguiGlobalSettings, EguiPlugin, PrimaryEguiContext},
    quick::WorldInspectorPlugin,
};

use lightyear::prelude::{Controlled, Predicted};

use shared::inputs::input::PLAYER_CAPSULE_HEIGHT;
use shared::protocol::PlayerId;

#[derive(Component, Default)]
pub struct PlayerCamera;

#[derive(Component)]
struct DebugCamera;

pub struct ClientCameraPlugin;
impl Plugin for ClientCameraPlugin {
    fn build(&self, app: &mut App) {
        let is_headless = app.world().get_resource::<crate::Headless>().is_some();

        if !is_headless {
            app.insert_resource(EguiGlobalSettings {
                auto_create_primary_context: false,
                ..Default::default()
            });
            app.add_plugins((
                EguiPlugin::default(),
                WorldInspectorPlugin::default().run_if(input_toggle_active(false, KeyCode::F2)),
            ));
            app.add_systems(Startup, spawn_menu_and_debug_camera);
        }

        app.add_observer(spawn_camera_when_player_spawn);
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
    trigger: On<Add, Controlled>,
    player_query: Query<&PlayerId, (With<Predicted>, With<Controlled>, With<PlayerId>)>,
    camera_query: Query<Entity, With<PlayerCamera>>,
    mut commands: Commands,
    local_player_id: Res<crate::LocalPlayerId>,
) {
    if !camera_query.is_empty() {
        bevy::log::info!("🎥 Camera already exists, skipping player camera spawn");
        return;
    }

    bevy::log::info!(
        "🔍 Attempting to spawn camera for entity {:?}",
        trigger.entity
    );

    if let Ok(player_id) = player_query.get(trigger.entity)
        && player_id.0.to_bits() == local_player_id.0
    {
        let camera_entity = commands
            .spawn((
                PlayerCamera,
                Camera {
                    order: 0,
                    ..default()
                },
                Camera3d::default(),
                Transform::from_xyz(0.0, PLAYER_CAPSULE_HEIGHT + 0.6, 0.0),
                Name::new(format!("Client_{}_Camera", local_player_id.0)),
            ))
            .id();

        commands.entity(trigger.entity).add_child(camera_entity);
        bevy::log::info!(
            "🎥 Spawned and parented camera for local player {}",
            local_player_id.0
        );
    } else {
        bevy::log::warn!(
            "⚠️ Failed to get player data for camera spawn. Player ID mismatch or entity not found."
        );
    }
}
