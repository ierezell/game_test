use bevy::prelude::{
    App, Assets, Commands, Mesh, Plugin, Query, Res, ResMut, Single, StandardMaterial, Update,
};
use bevy::state::commands::CommandsStatesExt;
use shared::GymMode;
use shared::gym::setup_gym_level;
use shared::level::generation::{LevelConfig, build_level_physics, generate_level};
use shared::level::visuals::build_level_visuals;

use crate::ClientGameState;
use lightyear::prelude::MessageReceiver;

use shared::protocol::{LevelSeed, StartLoadingGameEvent};

pub struct ClientGameCyclePlugin;

impl Plugin for ClientGameCyclePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, handle_world_creation);
    }
}

fn handle_world_creation(
    mut receiver: Single<&mut MessageReceiver<StartLoadingGameEvent>>,
    mut commands: Commands,
    gym_mode: Option<Res<GymMode>>,
    level_seed_query: Query<&LevelSeed>,
    meshes: ResMut<Assets<Mesh>>,
    materials: Option<ResMut<Assets<StandardMaterial>>>,
    state: Res<bevy::prelude::State<ClientGameState>>,
) {
    if receiver.has_messages() {
        receiver.receive().for_each(drop);

        // First transition to Loading state
        bevy::log::info!("📨 Client received StartLoadingGameEvent, transitioning to Loading");
        commands.set_state(ClientGameState::Loading);
    }

    // When in Loading state, spawn the level then transition to Playing
    if state.get() == &ClientGameState::Loading {
        if let Some(gym) = gym_mode
            && gym.0
        {
            bevy::log::info!("🏋️  Gym mode active - using simple static level");
            setup_gym_level(commands.reborrow(), meshes, materials);
        } else if let Some(level_seed) = level_seed_query.iter().next() {
            bevy::log::info!("🌱 Client generating level with seed: {}", level_seed.seed);

            let config = LevelConfig {
                seed: level_seed.seed,
                target_zone_count: 12,
                min_zone_spacing: 35.0,
                max_depth: 8,
            };

            let level_graph = generate_level(config);
            build_level_physics(commands.reborrow(), &level_graph);
            build_level_visuals(commands.reborrow(), meshes, materials, level_graph);
        }

        bevy::log::info!("✅ Client level loaded, transitioning to Playing state");
        commands.set_state(ClientGameState::Playing);
    }
}
