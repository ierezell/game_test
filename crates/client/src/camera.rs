use bevy::input::common_conditions::input_toggle_active;
use bevy::prelude::{
    Add, App, Camera, Camera2d, Camera3d, ClearColorConfig, Commands, Component, Entity,
    IsDefaultUiCamera, KeyCode, Name, On, OnExit, Plugin, Query, Res, Startup, Transform, With,
    default,
};

use bevy_inspector_egui::{
    bevy_egui::{EguiGlobalSettings, EguiPlugin, PrimaryEguiContext},
    quick::WorldInspectorPlugin,
};

use lightyear::prelude::{Controlled, Predicted};
use shared::NetworkMode;
use shared::inputs::input::PLAYER_CAPSULE_HEIGHT;
use shared::protocol::PlayerId;

use crate::ClientGameState;

#[derive(Component, Default)]
pub struct PlayerCamera;

#[derive(Component)]
struct DebugCamera;

pub struct ClientCameraPlugin;
impl Plugin for ClientCameraPlugin {
    fn build(&self, app: &mut App) {
        let is_headless = app
            .world()
            .get_resource::<crate::Headless>()
            .is_some_and(|headless| headless.0);

        // Camera spawn/despawn logic should run in both normal and headless apps
        // so tests exercise the same gameplay wiring as runtime.
        app.add_systems(OnExit(ClientGameState::Playing), despawn_player_cameras);
        app.add_observer(spawn_camera_when_local_player_id_added);

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
    }
}

fn spawn_menu_and_debug_camera(mut commands: Commands) {
    commands.spawn((
        Camera {
            order: 100,
            clear_color: ClearColorConfig::None,
            ..default()
        },
        Camera2d::default(),
        IsDefaultUiCamera,
        DebugCamera,
        PrimaryEguiContext,
    ));
}

fn despawn_player_cameras(mut commands: Commands, camera_query: Query<Entity, With<PlayerCamera>>) {
    for camera in &camera_query {
        commands.entity(camera).despawn();
    }
}

fn spawn_local_player_camera(commands: &mut Commands, player_entity: Entity, local_player_id: u64) {
    let camera_entity = commands
        .spawn((
            PlayerCamera,
            Camera {
                // Keep player camera above lobby/menu cameras to avoid order ambiguity
                // during fast auto-start transitions.
                order: 10,
                ..default()
            },
            Camera3d::default(),
            Transform::from_xyz(0.0, PLAYER_CAPSULE_HEIGHT + 0.6, 0.0),
            Name::new(format!("Client_{}_Camera", local_player_id)),
        ))
        .id();

    commands.entity(player_entity).add_child(camera_entity);
    bevy::log::info!(
        "🎥 Spawned and parented camera for local player {}",
        local_player_id
    );
}

fn spawn_camera_when_local_player_id_added(
    trigger: On<Add, PlayerId>,
    mut commands: Commands,
    local_player_id: Res<crate::LocalPlayerId>,
    network_mode: Res<NetworkMode>,
    camera_query: Query<Entity, With<PlayerCamera>>,
    player_query: Query<(&PlayerId, Option<&Predicted>, Option<&Controlled>), With<PlayerId>>,
) {
    if !camera_query.is_empty() {
        return;
    }

    let Ok((player_id, predicted, controlled)) = player_query.get(trigger.entity) else {
        return;
    };

    let is_local_player = player_id.0.to_bits() == local_player_id.0;
    let is_camera_eligible =
        *network_mode == NetworkMode::Local || (predicted.is_some() && controlled.is_some());
    if !is_local_player || !is_camera_eligible {
        return;
    }

    spawn_local_player_camera(&mut commands, trigger.entity, local_player_id.0);
}

#[cfg(test)]
mod tests {
    use super::ClientCameraPlugin;
    use crate::ClientGameState;
    use crate::Headless;

    use crate::LocalPlayerId;
    use bevy::prelude::{App, MinimalPlugins, With};
    use bevy::state::app::AppExtStates;
    use lightyear::prelude::{Controlled, PeerId, Predicted};
    use shared::NetworkMode;
    use shared::protocol::PlayerId;

    fn run_frames(app: &mut App, frames: usize) {
        for _ in 0..frames {
            app.update();
        }
    }

    fn setup_camera_test_app(network_mode: NetworkMode) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(bevy::state::app::StatesPlugin);
        app.init_state::<ClientGameState>();
        app.insert_state(ClientGameState::Playing);
        app.insert_resource(Headless(true));
        app.insert_resource(LocalPlayerId(1));
        app.insert_resource(network_mode);
        app.add_plugins(ClientCameraPlugin);
        app
    }

    #[test]
    fn local_mode_spawns_camera_for_local_player_without_prediction_markers() {
        let mut app = setup_camera_test_app(NetworkMode::Local);

        app.world_mut().spawn(PlayerId(PeerId::Netcode(1)));
        run_frames(&mut app, 3);

        let camera_count = app
            .world_mut()
            .query_filtered::<bevy::prelude::Entity, With<super::PlayerCamera>>()
            .iter(app.world())
            .count();
        assert_eq!(camera_count, 1);
    }

    #[test]
    fn udp_mode_requires_predicted_controlled_for_camera_spawn() {
        let mut app = setup_camera_test_app(NetworkMode::Udp);

        app.world_mut().spawn(PlayerId(PeerId::Netcode(1)));
        run_frames(&mut app, 3);

        let camera_count = app
            .world_mut()
            .query_filtered::<bevy::prelude::Entity, With<super::PlayerCamera>>()
            .iter(app.world())
            .count();
        assert_eq!(camera_count, 0);
    }

    #[test]
    fn udp_mode_spawns_camera_for_predicted_controlled_local_player() {
        let mut app = setup_camera_test_app(NetworkMode::Udp);

        app.world_mut()
            .spawn((PlayerId(PeerId::Netcode(1)), Predicted, Controlled));
        run_frames(&mut app, 3);

        let camera_count = app
            .world_mut()
            .query_filtered::<bevy::prelude::Entity, With<super::PlayerCamera>>()
            .iter(app.world())
            .count();
        assert_eq!(camera_count, 1);
    }
}
