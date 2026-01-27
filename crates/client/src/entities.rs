use crate::input::get_player_input_map;

use bevy::app::Update;
use bevy::prelude::{
    App, Assets, Capsule3d, Color, Commands, Entity, Mesh, Mesh3d, MeshMaterial3d, Name, OnEnter,
    Plugin, Query, Res, ResMut, StandardMaterial, With, Without, default,
};
use leafwing_input_manager::prelude::ActionState;
use shared::entities::{KinematicDisplayBundle, PlayerPhysicsBundle};
use shared::input::PlayerAction;
use shared::level_generation::{LevelConfig, generate_level};
use shared::level_visuals::build_level_visuals;
use shared::protocol::LevelSeed;

use crate::{ClientGameState, LocalPlayerId};
use lightyear::prelude::{Controlled, Interpolated, Predicted};
use shared::input::{PLAYER_CAPSULE_HEIGHT, PLAYER_CAPSULE_RADIUS};

use shared::protocol::{CharacterMarker, PlayerColor, PlayerId};

pub struct ClientEntitiesPlugin;

impl Plugin for ClientEntitiesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, handle_interpolated_npcs_setup);
        app.add_systems(Update, handle_local_player_setup);
        app.add_systems(Update, handle_interpolated_players_setup);
        app.add_systems(
            OnEnter(ClientGameState::Playing),
            client_generate_level_on_enter,
        );
    }
}

/// System that runs on state transition to generate the level
///
/// This runs when entering the Playing state and generates the level from the replicated seed
fn client_generate_level_on_enter(
    commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<StandardMaterial>>,
    level_seed_query: Query<&LevelSeed>,
) {
    // Check if we have a level seed already
    if let Some(level_seed) = level_seed_query.iter().next() {
        bevy::log::info!(
            "🌱 Client generating level on state enter with seed: {}",
            level_seed.seed
        );

        let config = LevelConfig {
            seed: level_seed.seed,
            target_zone_count: 12,
            min_zone_spacing: 35.0,
            max_depth: 8,
        };

        let level_graph = generate_level(config);
        build_level_visuals(commands, meshes, materials, level_graph);

        bevy::log::info!("✅ Client level built on Playing state enter");
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
        // Interpolated players use Kinematic rigidbody (no physics simulation)
        // This allows Position → Transform sync for rendering without physics conflicts
        commands.entity(entity).insert((
            Mesh3d(meshes.add(Capsule3d::new(PLAYER_CAPSULE_RADIUS, PLAYER_CAPSULE_HEIGHT))),
            MeshMaterial3d(materials.add(color.0)),
            KinematicDisplayBundle::default(),
        ));
    }
}

fn handle_interpolated_npcs_setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    npc_query: Query<(Entity, &Name), (With<Interpolated>, Without<PlayerId>, Without<Mesh3d>)>,
) {
    for (entity, name) in npc_query.iter() {
        let color = if name.as_str().contains("Enemy") {
            Color::srgb(0.8, 0.2, 0.2)
        } else if name.as_str().contains("Guard") {
            Color::srgb(0.9, 0.4, 0.1)
        } else if name.as_str().contains("Bot") {
            Color::srgb(0.2, 0.2, 0.8)
        } else if name.as_str().contains("Scout") {
            Color::srgb(0.1, 0.7, 0.3)
        } else {
            Color::srgb(0.5, 0.5, 0.5)
        };

        // Interpolated NPCs use Kinematic rigidbody (no physics simulation)
        // This allows Position → Transform sync for rendering without physics conflicts
        commands.entity(entity).insert((
            Mesh3d(meshes.add(Capsule3d::new(PLAYER_CAPSULE_RADIUS, PLAYER_CAPSULE_HEIGHT))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: color,
                emissive: color.to_linear() * 0.5,
                ..default()
            })),
            KinematicDisplayBundle::default(),
        ));
    }
}
