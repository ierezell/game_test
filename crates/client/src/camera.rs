use avian3d::prelude::{Position, Rotation};

use bevy::prelude::{
    Add, App, Camera, Camera2d, Camera3d, Changed, Commands, Component, Entity, IntoScheduleConfigs,
    Name, On, Or, Plugin, PostUpdate, Query, Res, Startup, Transform, Vec3, With, default,
    in_state,
};

use bevy_inspector_egui::{
    bevy_egui::{EguiGlobalSettings, EguiPlugin, PrimaryEguiContext},
    quick::WorldInspectorPlugin,
};

use lightyear::prelude::{Controlled, Predicted};

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
        let is_headless = app.world().get_resource::<crate::Headless>().is_some();

        if !is_headless {
            app.insert_resource(EguiGlobalSettings {
                auto_create_primary_context: false,
                ..Default::default()
            });
            app.add_plugins((EguiPlugin::default(), WorldInspectorPlugin::default()));
            app.add_systems(Startup, spawn_menu_and_debug_camera);
        }

        app.add_observer(spawn_camera_when_player_spawn);
        app.add_systems(
            PostUpdate,
            update_camera_from_player.run_if(in_state(ClientGameState::Playing)),
        );
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
    player_query: Query<
        (&PlayerId, &Position),
        (With<Predicted>, With<Controlled>, With<PlayerId>),
    >,
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

    if let Ok((player_id, position)) = player_query.get(trigger.entity)
        && player_id.0.to_bits() == local_player_id.0
    {
        let camera_height = PLAYER_CAPSULE_HEIGHT + 0.6;
        let camera_position = position.0 + Vec3::new(0.0, camera_height, 0.0);

        bevy::log::info!(
            "🎥 Spawning camera at {:?} for player {}",
            camera_position,
            local_player_id.0
        );

        commands.spawn((
            PlayerCamera,
            Camera {
                order: 0,
                ..default()
            },
            Camera3d::default(),
            Transform::from_translation(camera_position),
            Name::new(format!("Client_{}_Camera", local_player_id.0)),
        ));
    } else {
        bevy::log::warn!(
            "⚠️ Failed to get player data for camera spawn. Player ID mismatch or entity not found."
        );
    }
}

fn update_camera_from_player(
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
        return;
    };

    let Ok((player_position, player_rotation)) = player_query.single() else {
        return;
    };

    camera_transform.translation = Vec3::new(
        player_position.0.x,
        player_position.0.y + PLAYER_CAPSULE_HEIGHT + 0.6,
        player_position.0.z,
    );

    camera_transform.rotation = player_rotation.0;
}

#[cfg(test)]
mod tests {
    use super::{PlayerCamera, update_camera_from_player};
    use avian3d::prelude::{Position, Rotation};
    use bevy::prelude::{App, MinimalPlugins, PostUpdate, Quat, Transform, Vec3};
    use lightyear::prelude::{Controlled, Predicted};
    use shared::inputs::input::PLAYER_CAPSULE_HEIGHT;
    use shared::protocol::PlayerId;

    #[test]
    fn camera_tracks_player_position_and_rotation() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(PostUpdate, update_camera_from_player);

        let expected_rotation = Quat::from_rotation_y(0.7) * Quat::from_rotation_x(-0.2);
        let player_position = Vec3::new(3.0, 1.5, -4.0);

        app.world_mut().spawn((
            PlayerId(lightyear::prelude::PeerId::Netcode(1)),
            Predicted,
            Controlled,
            Position::new(player_position),
            Rotation::from(expected_rotation),
        ));

        let camera_entity = app
            .world_mut()
            .spawn((PlayerCamera, Transform::default()))
            .id();

        app.update();

        let transform = app
            .world()
            .get::<Transform>(camera_entity)
            .expect("camera should have transform");

        assert_eq!(
            transform.translation,
            Vec3::new(
                player_position.x,
                player_position.y + PLAYER_CAPSULE_HEIGHT + 0.6,
                player_position.z
            )
        );

        let dot = transform.rotation.dot(expected_rotation).abs();
        assert!(
            dot > 0.999,
            "camera rotation should match player rotation, dot={}",
            dot
        );
    }
}
