use bevy::input::common_conditions::input_toggle_active;
use bevy::prelude::{
    Add, App, Camera, Camera2d, Camera3d, ClearColorConfig, Commands, Component, Entity,
    IntoScheduleConfigs, IsDefaultUiCamera, KeyCode, Name, On, Plugin, Query, Res, Startup,
    Transform, Update, With, default, in_state,
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
            app.add_systems(
                Update,
                ensure_local_player_camera_exists.run_if(in_state(ClientGameState::Playing)),
            );
        }

        app.add_observer(spawn_camera_when_player_spawn);
    }
}

fn spawn_local_player_camera(commands: &mut Commands, player_entity: Entity, local_player_id: u64) {
    let camera_entity = commands
        .spawn((
            PlayerCamera,
            Camera {
                order: 0,
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

fn spawn_camera_when_player_spawn(
    trigger: On<Add, PlayerId>,
    player_query: Query<&PlayerId, With<PlayerId>>,
    predicted_controlled_query: Query<(), (With<Predicted>, With<Controlled>)>,
    camera_query: Query<Entity, With<PlayerCamera>>,
    mut commands: Commands,
    local_player_id: Res<crate::LocalPlayerId>,
    network_mode: Res<NetworkMode>,
) {
    if !camera_query.is_empty() {
        bevy::log::info!("🎥 Camera already exists, skipping player camera spawn");
        return;
    }

    bevy::log::info!(
        "🔍 Attempting to spawn camera for entity {:?}",
        trigger.entity
    );

    if let Ok(player_id) = player_query.get(trigger.entity) {
        if player_id.0.to_bits() == local_player_id.0 {
            let is_predicted_controlled = predicted_controlled_query.get(trigger.entity).is_ok();
            let is_local_host_mode = *network_mode == NetworkMode::Local;

            if !is_predicted_controlled && !is_local_host_mode {
                return;
            }

            spawn_local_player_camera(&mut commands, trigger.entity, local_player_id.0);
        }
    } else {
        bevy::log::warn!(
            "⚠️ Failed to get player data for camera spawn. Player ID mismatch or entity not found."
        );
    }
}

fn ensure_local_player_camera_exists(
    mut commands: Commands,
    local_player_id: Res<crate::LocalPlayerId>,
    network_mode: Res<NetworkMode>,
    camera_query: Query<Entity, With<PlayerCamera>>,
    predicted_player_query: Query<
        (Entity, &PlayerId),
        (With<PlayerId>, With<Predicted>, With<Controlled>),
    >,
    player_query: Query<(Entity, &PlayerId), With<PlayerId>>,
) {
    if !camera_query.is_empty() {
        return;
    }

    if let Some((entity, _)) = predicted_player_query
        .iter()
        .find(|(_, player_id)| player_id.0.to_bits() == local_player_id.0)
    {
        spawn_local_player_camera(&mut commands, entity, local_player_id.0);
        return;
    }

    if *network_mode != NetworkMode::Local {
        return;
    }

    if let Some((entity, _)) = player_query
        .iter()
        .find(|(_, player_id)| player_id.0.to_bits() == local_player_id.0)
    {
        spawn_local_player_camera(&mut commands, entity, local_player_id.0);
    }
}

#[cfg(test)]
mod tests {
    use super::ensure_local_player_camera_exists;
    use crate::LocalPlayerId;
    use bevy::prelude::{App, MinimalPlugins, Update, With};
    use lightyear::prelude::{Controlled, PeerId, Predicted};
    use shared::NetworkMode;
    use shared::protocol::PlayerId;

    fn run_once(app: &mut App) {
        app.update();
    }

    #[test]
    fn local_mode_spawns_camera_for_local_player_without_prediction_markers() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(LocalPlayerId(1));
        app.insert_resource(NetworkMode::Local);
        app.add_systems(Update, ensure_local_player_camera_exists);

        app.world_mut().spawn(PlayerId(PeerId::Netcode(1)));
        run_once(&mut app);

        let camera_count = app
            .world_mut()
            .query_filtered::<bevy::prelude::Entity, With<super::PlayerCamera>>()
            .iter(app.world())
            .count();
        assert_eq!(camera_count, 1);
    }

    #[test]
    fn udp_mode_requires_predicted_controlled_for_camera_spawn() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(LocalPlayerId(1));
        app.insert_resource(NetworkMode::Udp);
        app.add_systems(Update, ensure_local_player_camera_exists);

        app.world_mut().spawn(PlayerId(PeerId::Netcode(1)));
        run_once(&mut app);

        let camera_count = app
            .world_mut()
            .query_filtered::<bevy::prelude::Entity, With<super::PlayerCamera>>()
            .iter(app.world())
            .count();
        assert_eq!(camera_count, 0);
    }

    #[test]
    fn udp_mode_spawns_camera_for_predicted_controlled_local_player() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(LocalPlayerId(1));
        app.insert_resource(NetworkMode::Udp);
        app.add_systems(Update, ensure_local_player_camera_exists);

        app.world_mut()
            .spawn((PlayerId(PeerId::Netcode(1)), Predicted, Controlled));
        run_once(&mut app);

        let camera_count = app
            .world_mut()
            .query_filtered::<bevy::prelude::Entity, With<super::PlayerCamera>>()
            .iter(app.world())
            .count();
        assert_eq!(camera_count, 1);
    }
}
