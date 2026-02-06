use crate::inputs::input::get_player_input_map;

use bevy::app::Update;
use bevy::prelude::{
    App, Assets, Capsule3d, Color, Commands, Entity, Mesh, Mesh3d, MeshMaterial3d, Plugin, Query,
    Res, ResMut, StandardMaterial, With, Without, default,
};
use leafwing_input_manager::prelude::ActionState;

use shared::entities::{NpcPhysicsBundle, PlayerPhysicsBundle};

use shared::inputs::input::PlayerAction;

use crate::LocalPlayerId;
use lightyear::prelude::{Controlled, Interpolated, Predicted};
use shared::inputs::input::{PLAYER_CAPSULE_HEIGHT, PLAYER_CAPSULE_RADIUS};

use shared::protocol::{CharacterMarker, PlayerColor, PlayerId};

pub struct ClientEntitiesPlugin;

impl Plugin for ClientEntitiesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, handle_interpolated_npcs_setup);
        app.add_systems(Update, handle_local_player_setup);
        app.add_systems(Update, handle_interpolated_players_setup);
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
    // TODO : may use single here ?
    for (entity, color, player_id) in player_query.iter() {
        if player_id.0.to_bits() == local_player_id.0 {
            let input_map = get_player_input_map();
            let mut action_state = ActionState::<PlayerAction>::default();
            action_state.enable();
            commands.entity(entity).insert((
                Mesh3d(meshes.add(Capsule3d::new(PLAYER_CAPSULE_RADIUS, PLAYER_CAPSULE_HEIGHT))),
                MeshMaterial3d(materials.add(color.0)),
                input_map,
                action_state,
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
            PlayerPhysicsBundle::default(),
        ));
    }
}

fn handle_interpolated_npcs_setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    npc_query: Query<Entity, (With<Interpolated>, Without<PlayerId>, Without<Mesh3d>)>,
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
            NpcPhysicsBundle::default(),
        ));
    }
}
