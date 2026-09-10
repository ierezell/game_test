use bevy::prelude::{
    App, Assets, Commands, Mesh, Plugin, Query, Res, ResMut, StandardMaterial, Update, With,
};
use bevy::state::commands::CommandsStatesExt;
use lightyear::prelude::{Client, MessageReceiver};
use shared::{GymMode, NetworkMode};
use shared::gym::setup_gym_level;
use shared::level::generation::{LevelConfig, build_level_physics, generate_level};
use shared::level::visuals::build_level_visuals;

use crate::ClientGameState;
use shared::protocol::{LevelSeed, StartLoadingGameEvent};

pub struct ClientGameCyclePlugin;

impl Plugin for ClientGameCyclePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, handle_world_creation);
    }
}

#[allow(clippy::too_many_arguments)]
fn handle_world_creation(
    mut receiver_q: Query<&mut MessageReceiver<StartLoadingGameEvent>, With<Client>>,
    mut commands: Commands,
    gym_mode: Option<Res<GymMode>>,
    network_mode: Res<NetworkMode>,
    level_seed_query: Query<&LevelSeed>,
    meshes: ResMut<Assets<Mesh>>,
    materials: Option<ResMut<Assets<StandardMaterial>>>,
    state: Res<bevy::prelude::State<ClientGameState>>,
) {
    let has_level_seed = level_seed_query.iter().next().is_some();
    let has_msg = receiver_q.iter().any(|r| r.has_messages());
    for mut receiver in receiver_q.iter_mut() {
        receiver.receive().for_each(drop);
    }

    if has_msg {
        // First transition to Loading state
        bevy::log::info!("📨 Client received StartLoadingGameEvent, transitioning to Loading");
        commands.set_state(ClientGameState::Loading);
    }

    if state.get() == &ClientGameState::Lobby && has_level_seed {
        bevy::log::info!(
            "📦 Client detected replicated LevelSeed while in Lobby, transitioning to Loading"
        );
        commands.set_state(ClientGameState::Loading);
    }

    // When in Loading state, spawn the level then transition to Playing
    if state.get() == &ClientGameState::Loading {
        if *network_mode == NetworkMode::Local {
            bevy::log::info!(
                "🏠 Local host mode detected: skipping client-side level generation (server world is shared)"
            );
            commands.set_state(ClientGameState::Playing);
            return;
        }

        if let Some(gym) = gym_mode
            && gym.0
        {
            bevy::log::info!("🏋️  Gym mode active - using simple static level");
            setup_gym_level(commands.reborrow(), meshes, materials);
        } else if let Some(seed) = level_seed_query
            .iter()
            .next()
            .map(|seed| seed.seed)
        {
            bevy::log::info!("🌱 Client generating level with seed: {}", seed);

            let config = LevelConfig {
                seed,
                target_zone_count: 12,
                min_zone_spacing: 35.0,
                max_depth: 8,
            };

            let level_graph = generate_level(config);
            build_level_physics(commands.reborrow(), &level_graph);
            build_level_visuals(commands.reborrow(), meshes, materials, &level_graph);
        } else {
            bevy::log::info!(
                "⏳ Client waiting for LevelSeed replication before generating procedural level"
            );
            return;
        }

        bevy::log::info!("✅ Client level loaded, transitioning to Playing state");
        commands.set_state(ClientGameState::Playing);
    }
}
