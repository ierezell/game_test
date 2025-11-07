use avian3d::prelude::{LinearVelocity, Position, Rotation};
use bevy::prelude::{
    Add, App, AppExtStates, Assets, Commands, CommandsStatesExt, DetectChanges, Entity,
    FixedUpdate, Mesh, Name, On, OnEnter, Plugin, Query, Res, ResMut, Resource, Single,
    StandardMaterial, State, Update, Vec2, With, Without, debug, info, warn,
};
use leafwing_input_manager::prelude::ActionState;
use lightyear::connection::client::Connected;
use lightyear::prelude::PeerId;

use lightyear::prelude::server::ClientOf;
use lightyear::prelude::{
    Confirmed, ControlledBy, InterpolationTarget, LocalTimeline, MessageReceiver, NetworkTarget,
    NetworkTimeline, Predicted, PredictionTarget, RemoteId, Replicate, Server,
};

use shared::entities::player::color_from_id;

use shared::game_state::GameState;
use shared::input::{PlayerAction, shared_player_movement};
use shared::protocol::{
    GameSeed, GameStateMarker, LobbyState, PlayerColor, PlayerId, ReplicatedGameSeed,
    ReplicatedLobbyInfo, StartGameEvent, WorldCreatedEvent,
};

#[derive(Resource, Default)]
pub struct WorldCreationTracker {
    pub static_world_created: bool,
}

pub struct ServerGameplayPlugin;

impl Plugin for ServerGameplayPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>();
        app.insert_resource(LobbyState {
            players: Vec::new(),
            host_id: 0,
        });

        app.insert_resource(WorldCreationTracker::default());

        app.add_observer(handle_connected);
        app.add_systems(FixedUpdate, server_player_movement);
        app.add_systems(FixedUpdate, debug_player_position);

        // System to create the game state marker entity on startup
        app.add_systems(bevy::prelude::Startup, setup_game_state_marker);

        // Systems to sync replicated components when lobby changes
        app.add_systems(Update, sync_lobby_info);

        // Handle client messages
        app.add_systems(Update, handle_start_game_event);
        app.add_systems(Update, handle_world_created_event);

        app.add_systems(OnEnter(GameState::Loading), setup_game_seed);
        app.add_systems(OnEnter(GameState::Spawning), spawn_game_world);
        app.add_systems(OnEnter(GameState::Playing), spawn_player_entities);
    }
}

/// Create a game state marker entity that will hold replicated components
fn setup_game_state_marker(mut commands: Commands) {
    commands.spawn((
        Name::new("GameStateMarker"),
        GameStateMarker,
        ReplicatedLobbyInfo {
            player_count: 0,
            host_id: 0,
        },
        Replicate::to_clients(NetworkTarget::All),
    ));
    info!("🎯 SERVER: Created GameStateMarker entity");
}

/// Sync replicated lobby info when lobby state changes
fn sync_lobby_info(
    lobby_state: Res<LobbyState>,
    mut game_state_query: Query<&mut ReplicatedLobbyInfo, With<GameStateMarker>>,
) {
    if !lobby_state.is_changed() {
        return;
    }

    if let Ok(mut lobby_info) = game_state_query.single_mut() {
        lobby_info.player_count = lobby_state.players.len() as u32;
        lobby_info.host_id = lobby_state.host_id;
        info!(
            "📡 SERVER: Synced lobby info - {} players",
            lobby_info.player_count
        );
    }
}

/// Handle StartGameEvent from client
fn handle_start_game_event(
    mut message_receivers: Query<&mut MessageReceiver<StartGameEvent>>,
    mut commands: Commands,
    current_state: Res<State<GameState>>,
) {
    for mut receiver in message_receivers.iter_mut() {
        for _event in receiver.receive() {
            info!("🎯 SERVER: Received StartGameEvent from client!");

            // Only process if we're in lobby
            if *current_state.get() == GameState::InLobby {
                info!("🎯 SERVER: Valid StartGameEvent - transitioning to Loading state");
                commands.set_state(GameState::Loading);
            } else {
                info!(
                    "🎯 SERVER: StartGameEvent received but already in state: {:?}",
                    current_state.get()
                );
            }
        }
    }
}

/// Handle WorldCreatedEvent from clients
fn handle_world_created_event(
    mut message_receivers: Query<&mut MessageReceiver<WorldCreatedEvent>>,
    _lobby_state: Res<LobbyState>,
    world_tracker: ResMut<WorldCreationTracker>,
    mut commands: Commands,
) {
    for mut receiver in message_receivers.iter_mut() {
        for event in receiver.receive() {
            info!(
                "📨 SERVER: Received WorldCreatedEvent from client {}",
                event.client_id
            );

            // Track which clients have confirmed
            // For now, we'll just check if we got confirmations from all clients
            // In a full implementation, you'd track individual clients

            // Check if all clients have confirmed and server world is created
            if world_tracker.static_world_created {
                info!("✅ SERVER: World created, transitioning to Playing to spawn players");
                commands.set_state(GameState::Playing);
            }
        }
    }
}

/// System that runs when the server enters Loading state
/// Creates the game seed resource and replicates it to clients
fn setup_game_seed(
    mut commands: Commands,
    mut game_state_query: Query<(Entity, &mut ReplicatedLobbyInfo), With<GameStateMarker>>,
) {
    info!("🎮 SERVER: Entered Loading state - creating game seed");

    let seed = 42; // Simple seed for now
    let seed_resource = GameSeed { seed };

    commands.insert_resource(seed_resource);
    info!("📦 SERVER: Created GameSeed {} resource", seed);

    // Add the replicated seed to the game state marker entity
    if let Ok((entity, _)) = game_state_query.single_mut() {
        commands.entity(entity).insert(ReplicatedGameSeed { seed });
        info!("📡 SERVER: Added ReplicatedGameSeed to GameStateMarker");
    }

    // Automatically transition to Spawning
    commands.set_state(GameState::Spawning);
}

/// System that runs when the server enters Spawning state
fn spawn_game_world(
    game_seed: Option<Res<GameSeed>>,
    mut commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<StandardMaterial>>,
    mut world_tracker: ResMut<WorldCreationTracker>,
    _current_state: Res<State<GameState>>,
) {
    // Prevent re-running if we're re-entering this state
    if world_tracker.static_world_created {
        info!("⚠️ SERVER: Static world already created, skipping");
        return;
    }

    info!("🌍 SERVER: Spawning static world");

    let seed = game_seed.map(|s| s.seed).unwrap_or(42);
    info!("🎲 SERVER: Using seed {} for world generation", seed);

    // Create the static level using the seed
    shared::level::create_static::setup_static_level(
        commands.reborrow(),
        meshes,
        materials,
        Some(seed),
    );

    // Mark server static world as created
    world_tracker.static_world_created = true;
    info!("✅ SERVER: Static world created");

    // Automatically transition to Playing state to spawn players
    // In a real implementation, you might want to wait for client confirmations here
    info!("🎮 SERVER: Transitioning to Playing state to spawn players");
    commands.set_state(GameState::Playing);
}

fn debug_player_position(
    query: Query<(&Name, &Position, &LinearVelocity), With<PlayerId>>,
    timeline: Single<&LocalTimeline, With<Server>>,
) {
    for (name, pos, vel) in query.iter() {
        debug!(
            "S:{:?} pos:{:?} vel:{:?} tick:{:?}",
            name,
            pos,
            vel,
            timeline.tick()
        );
    }
}

fn handle_connected(
    trigger: On<Add, Connected>,
    query: Query<&RemoteId, With<ClientOf>>,
    mut lobby_state: ResMut<LobbyState>,
) {
    let Ok(client_id) = query.get(trigger.entity) else {
        info!(
            "❌ Failed to get RemoteId for connected entity {:?}",
            trigger.entity
        );
        return;
    };
    let peer_id = client_id.0.to_bits();
    info!(
        "✅ Client connected with client-id {client_id:?} (peer_id: {}). Adding to lobby.",
        peer_id
    );

    // Add player to lobby state - do NOT spawn player entity yet
    if !lobby_state.players.contains(&peer_id) {
        lobby_state.players.push(peer_id);

        // Set first player as host
        if lobby_state.players.len() == 1 {
            lobby_state.host_id = peer_id;
            info!("👑 Player {} is now the host", peer_id);
        }
    }

    info!("🎪 Lobby now has {} players", lobby_state.players.len());

    // Players will be spawned when the game transitions to Playing state
    info!("👥 Player added to lobby, waiting for game start to spawn entities");
}

/// Spawn player entities when entering Playing state
fn spawn_player_entities(
    mut commands: Commands,
    lobby_state: Res<LobbyState>,
    client_query: Query<(Entity, &RemoteId), With<ClientOf>>,
    existing_players: Query<&PlayerId>,
) {
    // Check if players are already spawned to prevent re-spawning
    if !existing_players.is_empty() {
        info!("⚠️ SERVER: Players already spawned, skipping");
        return;
    }

    info!(
        "🚀 SERVER: Spawning player entities for {} players",
        lobby_state.players.len()
    );

    for player_id in &lobby_state.players {
        // Find the client entity for this player
        if let Some((client_entity, client_id)) = client_query
            .iter()
            .find(|(_, remote_id)| remote_id.0.to_bits() == *player_id)
        {
            let color = color_from_id(client_id.to_bits());

            info!(
                "🎯 SERVER: Spawning player for client_id: {:?} (peer_id: {})",
                client_id, player_id
            );

            let player = commands
                .spawn((
                    Name::new(format!("Player_{}", client_id.to_bits())),
                    PlayerId(*player_id),
                    PlayerColor(color),
                    Position::default(),
                    Rotation::default(),
                    LinearVelocity::default(),
                    ControlledBy {
                        owner: client_entity,
                        lifetime: Default::default(),
                    },
                    Replicate::to_clients(NetworkTarget::All),
                    PredictionTarget::to_clients(NetworkTarget::Single(PeerId::Netcode(
                        *player_id,
                    ))),
                    InterpolationTarget::to_clients(NetworkTarget::AllExceptSingle(
                        PeerId::Netcode(*player_id),
                    )),
                ))
                .id();

            info!(
                "🌐 SERVER: Player entity {:?} created for client {:?}",
                player, client_id
            );

            // Add physics bundle
            commands
                .entity(player)
                .insert(shared::entities::player::PlayerPhysicsBundle::default());

            info!(
                "✅ SERVER: Player entity {:?} fully configured for client {:?}",
                player, client_id
            );
        } else {
            warn!(
                "❌ SERVER: Could not find client entity for player_id: {}",
                player_id
            );
        }
    }

    info!("🎮 SERVER: All players spawned, game is ready!");
}

pub fn server_player_movement(
    mut player_query: Query<
        (
            Entity,
            &mut Rotation,
            &mut LinearVelocity,
            &ActionState<PlayerAction>,
        ),
        (
            With<PlayerId>,
            Without<Predicted>,
            Without<Confirmed<Position>>,
        ),
    >,
) {
    for (entity, mut rotation, mut velocity, action_state) in player_query.iter_mut() {
        let axis_pair = action_state.axis_pair(&PlayerAction::Move);
        if axis_pair != Vec2::ZERO || !action_state.get_pressed().is_empty() {
            debug!(
                "🖥️ SERVER: Processing movement for entity {:?} with axis {:?} and actions {:?}",
                entity,
                axis_pair,
                action_state.get_pressed()
            );
        }

        shared_player_movement(action_state, &mut rotation, &mut velocity);
    }
}
