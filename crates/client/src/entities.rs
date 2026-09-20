use bevy::app::Update;
use bevy::prelude::{
    App, Assets, Capsule3d, Color, Commands, Entity, Mesh, Mesh3d, MeshMaterial3d, Plugin, Query,
    Res, ResMut, StandardMaterial, Transform, With, Without, default,
};

use shared::entities::PlayerPhysicsBundle;

use crate::LocalPlayerId;
use crate::inputs::spawn_local_player_input_actions;
use lightyear::prelude::{Controlled, Interpolated, Predicted};
use shared::components::stamina::Stamina;
use shared::inputs::{PLAYER_CAPSULE_HEIGHT, PLAYER_CAPSULE_RADIUS};

use shared::protocol::{CharacterMarker, PlayerColor, PlayerId};

pub struct ClientEntitiesPlugin;

impl Plugin for ClientEntitiesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, handle_interpolated_npcs_setup);
        app.add_systems(Update, handle_local_player_setup);
        app.add_systems(Update, handle_interpolated_players_setup);
        app.add_systems(Update, spawn_local_player_input_actions);
    }
}

fn handle_local_player_setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    player_query: Query<
        (Entity, &PlayerColor, &PlayerId),
        (
            With<Predicted>,
            With<Controlled>,
            With<PlayerId>,
            Without<Mesh3d>,
        ),
    >,
    local_player_id: Res<LocalPlayerId>,
) {
    for (entity, color, player_id) in player_query.iter() {
        if player_id.0.to_bits() == local_player_id.0 {
            commands.entity(entity).insert((
                Mesh3d(meshes.add(Capsule3d::new(PLAYER_CAPSULE_RADIUS, PLAYER_CAPSULE_HEIGHT))),
                MeshMaterial3d(materials.add(color.0)),
                Transform::default(),
                Stamina::default(),
                PlayerPhysicsBundle::default(),
            ));
        }
    }
}

fn handle_interpolated_players_setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    player_query: Query<
        (Entity, &PlayerColor),
        (With<Interpolated>, With<CharacterMarker>, Without<Mesh3d>),
    >,
) {
    for (entity, color) in player_query.iter() {
        commands.entity(entity).insert((
            Mesh3d(meshes.add(Capsule3d::new(PLAYER_CAPSULE_RADIUS, PLAYER_CAPSULE_HEIGHT))),
            MeshMaterial3d(materials.add(color.0)),
            Transform::default(),
        ));
    }
}

fn handle_interpolated_npcs_setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    npc_query: Query<Entity, (With<CharacterMarker>, Without<PlayerId>, Without<Mesh3d>)>,
) {
    for entity in npc_query.iter() {
        let color = Color::srgb(0.5, 0.5, 0.5);
        commands.entity(entity).insert((
            Mesh3d(meshes.add(Capsule3d::new(PLAYER_CAPSULE_RADIUS, PLAYER_CAPSULE_HEIGHT))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: color,
                unlit: false,
                ..default()
            })),
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::handle_interpolated_players_setup;
    use avian3d::prelude::RigidBody;
    use bevy::prelude::{
        App, AssetApp, AssetPlugin, Color, Mesh, Mesh3d, MinimalPlugins, StandardMaterial, Update,
    };
    use lightyear::prelude::Interpolated;
    use shared::protocol::{CharacterMarker, PlayerColor};

    #[test]
    fn interpolated_players_are_visual_only_on_the_client() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()));
        app.init_asset::<Mesh>();
        app.init_asset::<StandardMaterial>();
        app.add_systems(Update, handle_interpolated_players_setup);

        let entity = app
            .world_mut()
            .spawn((
                Interpolated,
                CharacterMarker,
                PlayerColor(Color::srgb(0.2, 0.4, 0.8)),
            ))
            .id();

        app.update();

        assert!(app.world().get::<Mesh3d>(entity).is_some());
        assert!(app.world().get::<RigidBody>(entity).is_none());
    }
}
