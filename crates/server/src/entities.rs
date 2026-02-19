use avian3d::prelude::{LinearVelocity, Position, Rotation};
use bevy::{
    ecs::schedule::IntoScheduleConfigs,
    prelude::{
        App, Assets, Commands, CommandsStatesExt, Entity, FixedUpdate, Mesh, Name, Plugin, Query,
        Res, ResMut, StandardMaterial, Vec3, With, info,
    },
    state::{condition::in_state, state::OnEnter},
};
use leafwing_input_manager::prelude::ActionState;

use shared::inputs::movement::GroundState;
use shared::{GymMode, level::visuals::build_level_visuals};
use shared::{gym::{setup_gym_level, spawn_gym_patrolling_npc_entities}, protocol::LevelSeed};
use shared::{
    inputs::input::PlayerAction,
    level::generation::{LevelConfig, build_level_physics, generate_level},
};

use lightyear::prelude::{
    Connected, ControlledBy, InterpolationTarget, NetworkTarget, PeerId, PredictionTarget,
    RemoteId, Replicate, server::ClientOf,
};
use shared::{
    components::flashlight::PlayerFlashlight,
    components::{
        health::{Health, Respawnable},
        weapons::Gun,
    },
    entities::{PlayerPhysicsBundle, color_from_id},
    protocol::{CharacterMarker, LobbyState, PlayerColor, PlayerId},
};

use crate::ServerGameState;

pub struct ServerEntitiesPlugin;
impl Plugin for ServerEntitiesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (
                spawn_late_joining_players,
                handle_player_death,
                handle_player_respawn,
            )
                .run_if(in_state(ServerGameState::Playing)),
        );
        app.add_systems(OnEnter(ServerGameState::Loading), generate_and_build_level);
        app.add_systems(
            OnEnter(ServerGameState::Playing),
            spawn_gym_patrolling_npc_entities,
        );
    }
}

fn generate_and_build_level(
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
        // Spawn players in gym mode
        spawn_player_entities(commands.reborrow(), &lobby_state, &client_query);
        // NPC spawning is handled by a separate system (spawn_gym_patrolling_npc_entities is a system)
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
                level_graph,
            );
        }

        // Spawn players in normal mode
        spawn_player_entities(commands.reborrow(), &lobby_state, &client_query);
    }

    // After loading is complete, transition to Playing
    info!("✅ Server level loaded, transitioning to Playing state");
    commands.set_state(ServerGameState::Playing);
}

fn spawn_player_entities(
    mut commands: Commands,
    lobby_state: &Query<&LobbyState>,
    client_query: &Query<(Entity, &RemoteId), With<ClientOf>>,
) {
    let Ok(lobby_data) = lobby_state.single() else {
        return;
    };
    let player_count = lobby_data.players.len() as f32;

    // Spawn players in a circle
    let spawn_radius = 3.0;

    for (index, player_id) in lobby_data.players.iter().enumerate() {
        if let Some((client_entity, remote_id)) =
            client_query
                .iter()
                .find(|(_, remote_id)| match remote_id.0 {
                    PeerId::Netcode(id) => id == *player_id,
                    _ => false,
                })
        {
            let angle = (index as f32) * 2.0 * std::f32::consts::PI / player_count;
            let spawn_position =
                Vec3::new(spawn_radius * angle.cos(), 3.5, spawn_radius * angle.sin());

            println!(
                "DEBUG: Spawning player entity for ID: {} at {:?}",
                player_id, spawn_position
            );

            let _player = commands
                .spawn((
                    Name::new(format!("Player_{}", player_id)),
                    PlayerId(PeerId::Netcode(*player_id)),
                    PlayerColor(color_from_id(*player_id)),
                    Rotation::default(),
                    Position::new(spawn_position),
                    LinearVelocity::default(),
                    Health::basic(),
                    Respawnable::new(3.0), // 3 second respawn delay
                    Gun::default(),
                    PlayerFlashlight::new(), // Add flashlight to player
                    ControlledBy {
                        owner: client_entity,
                        lifetime: Default::default(),
                    },
                    Replicate::to_clients(NetworkTarget::All),
                    PredictionTarget::to_clients(NetworkTarget::Single(remote_id.0)),
                    InterpolationTarget::to_clients(NetworkTarget::AllExceptSingle(remote_id.0)),
                ))
                .insert(GroundState::default())
                .insert((
                    CharacterMarker,
                    PlayerPhysicsBundle::default(),
                    ActionState::<PlayerAction>::default(),
                    leafwing_input_manager::prelude::InputMap::<PlayerAction>::default(),
                ))
                .id();
        } else {
            println!(
                "DEBUG: Could not find client entity for player ID: {}",
                player_id
            );
            // Dump available clients
            for (e, r) in client_query.iter() {
                println!("DEBUG: Available Client: {:?} with RemoteId: {:?}", e, r);
            }
        }
    }
}

/// Spawn player entities for clients that join after the game has already started
fn spawn_late_joining_players(
    mut commands: Commands,
    lobby_state: Query<&LobbyState>,
    client_query: Query<(Entity, &RemoteId), (With<ClientOf>, With<Connected>)>,
    existing_players: Query<&PlayerId>,
) {
    let Ok(lobby_data) = lobby_state.single() else {
        return;
    };

    // Check each connected client to see if they need a player entity
    for (client_entity, remote_id) in client_query.iter() {
        let player_id_bits = match remote_id.0 {
            PeerId::Netcode(id) => id,
            _ => continue,
        };

        // Check if this client is in the lobby
        if !lobby_data.players.contains(&player_id_bits) {
            continue;
        }

        // Check if this player already has an entity
        let player_exists = existing_players.iter().any(|pid| match pid.0 {
            PeerId::Netcode(id) => id == player_id_bits,
            _ => false,
        });

        if !player_exists {
            // Find a spawn position - use index based on when they joined
            let index = lobby_data
                .players
                .iter()
                .position(|&id| id == player_id_bits)
                .unwrap_or(0);
            let player_count = lobby_data.players.len() as f32;
            let spawn_radius = 3.0;
            let angle = (index as f32) * 2.0 * std::f32::consts::PI / player_count;
            let spawn_position =
                Vec3::new(spawn_radius * angle.cos(), 3.5, spawn_radius * angle.sin());

            println!(
                "DEBUG: Spawning late-joining player entity for ID: {} at {:?}",
                player_id_bits, spawn_position
            );

            commands
                .spawn((
                    Name::new(format!("Player_{}", player_id_bits)),
                    PlayerId(PeerId::Netcode(player_id_bits)),
                    PlayerColor(color_from_id(player_id_bits)),
                    Rotation::default(),
                    Position::new(spawn_position),
                    LinearVelocity::default(),
                    Health::basic(),
                    Respawnable::new(3.0),
                    Gun::default(),
                    PlayerFlashlight::new(), // Add flashlight to late-joining player
                    ControlledBy {
                        owner: client_entity,
                        lifetime: Default::default(),
                    },
                    Replicate::to_clients(NetworkTarget::All),
                    PredictionTarget::to_clients(NetworkTarget::Single(remote_id.0)),
                    InterpolationTarget::to_clients(NetworkTarget::AllExceptSingle(remote_id.0)),
                ))
                .insert(GroundState::default())
                .insert((
                    CharacterMarker,
                    PlayerPhysicsBundle::default(),
                    ActionState::<PlayerAction>::default(),
                    leafwing_input_manager::prelude::InputMap::<PlayerAction>::default(),
                ));
        }
    }
}

/// Handle player death - despawn the entity when health reaches 0
fn handle_player_death(
    mut commands: Commands,
    player_query: Query<(Entity, &Health, &PlayerId), With<CharacterMarker>>,
) {
    for (entity, health, player_id) in player_query.iter() {
        if health.is_dead {
            info!(
                "Player {:?} has died, despawning entity {:?}",
                player_id, entity
            );
            // Despawn the player entity - they'll respawn after the delay
            commands.entity(entity).despawn();
        }
    }
}

/// Handle player respawn - respawn players after their respawn delay
fn handle_player_respawn() {
    // This system will spawn players who are in the lobby but don't have entities
    // The spawn_late_joining_players system already handles this logic
    // So dead players will automatically respawn through that system
}
