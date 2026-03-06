use bevy::prelude::{
    Assets, Commands, CommandsStatesExt, Entity, Mesh, Query, Res, ResMut, StandardMaterial, With,
    info,
};

use lightyear::prelude::{RemoteId, server::ClientOf};
use shared::level::visuals::build_level_visuals;
use shared::{
    GymMode,
    gym::setup_gym_level,
    level::{
        building::build_procedural_runtime_content,
        generation::{LevelConfig, build_level_physics, generate_level},
    },
    protocol::{LevelSeed, LobbyState},
};

use crate::{ServerGameState, entities::player::spawn_player_entities};

#[allow(clippy::too_many_arguments)]
pub(super) fn generate_and_build_level(
    mut commands: Commands,
    meshes: Option<ResMut<Assets<Mesh>>>,
    mut materials: Option<ResMut<Assets<StandardMaterial>>>,
    gym_mode: Option<Res<GymMode>>,
    level_seed_query: Query<&LevelSeed>,
    lobby_state: Query<&LobbyState>,
    client_query: Query<(Entity, &RemoteId), With<ClientOf>>,
) {
    let is_gym_mode = gym_mode.map(|gm| gm.0).unwrap_or(false);

    if is_gym_mode {
        info!("🏋️  GYM MODE: Setting up simple test environment with one NPC and obstacles");
        if let Some(mesh_assets) = meshes {
            let material_assets = materials.take();
            setup_gym_level(commands.reborrow(), mesh_assets, material_assets);
        }
        // Spawn players in gym mode.
        spawn_player_entities(commands.reborrow(), &lobby_state, &client_query);
    } else if let Some(level_seed) = level_seed_query.iter().next() {
        bevy::log::info!(
            "🌱 Server generating level on state enter with seed: {}",
            level_seed.seed
        );

        info!("🎮 NORMAL MODE: Setting up procedural level generation");
        let config = LevelConfig {
            seed: level_seed.seed,
            target_zone_count: 12,
            min_zone_spacing: 35.0,
            max_depth: 8,
        };
        let level_graph = generate_level(config);
        build_level_physics(commands.reborrow(), &level_graph);

        if let (Some(mesh_assets), Some(mat_assets)) = (meshes, materials) {
            build_level_visuals(
                commands.reborrow(),
                mesh_assets,
                Some(mat_assets),
                &level_graph,
            );
        }

        build_procedural_runtime_content(&mut commands, &level_graph);

        // Spawn players in normal mode.
        spawn_player_entities(commands.reborrow(), &lobby_state, &client_query);
    }

    // After loading is complete, transition to Playing.
    info!("✅ Server level loaded, transitioning to Playing state");
    commands.set_state(ServerGameState::Playing);
}
