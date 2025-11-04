use avian3d::prelude::{LinearVelocity, Position, Rotation};
use bevy::prelude::{
    Add, App, AppExtStates, Commands, CommandsStatesExt, Entity, FixedUpdate, Name, On, OnEnter,
    Plugin, Query, Res, ResMut, Resource, Single, Update, Vec2, Vec3, With, Without, debug, info,
};
use leafwing_input_manager::prelude::ActionState;
use lightyear::connection::client::Connected;
use lightyear::prelude::PeerId;

use lightyear::prelude::server::{ClientOf, Server};
use lightyear::prelude::{
    Confirmed, ControlledBy, InterpolationTarget, LocalTimeline, NetworkTarget, NetworkTimeline,
    Predicted, PredictionTarget, RemoteId, Replicate,
};

use shared::entities::health::{Health, Respawnable};
use shared::entities::player::color_from_id;
use shared::entities::stamina::{StaminaEffects, add_stamina_to_player};
use shared::entities::weapons::add_weapon_holder;
use shared::game_state::GameState;
use shared::input::{PLAYER_CAPSULE_HEIGHT, PlayerAction, shared_player_movement_with_stamina};
use shared::protocol::{GameSeed, LobbyState, PlayerColor, PlayerId};

#[derive(Resource)]
pub struct GameSeedResource {
    pub seed: u64,
}

pub struct ServerGameplayPlugin;

impl Plugin for ServerGameplayPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>();

        app.insert_resource(LobbyState {
            players: Vec::new(),
            host_id: PeerId::Netcode(0),
            game_started: false,
        });

        app.add_observer(handle_connected);
        app.add_systems(FixedUpdate, server_player_movement);
        app.add_systems(FixedUpdate, debug_player_position);

        // Lobby management systems
        app.add_systems(Update, handle_game_progression);

        app.add_systems(OnEnter(GameState::Loading), start_game_with_seed);
        app.add_systems(OnEnter(GameState::Spawning), spawn_game_world);

        // Navigation systems
        app.add_systems(
            Update,
            (
                shared::navigation_pathfinding::update_target_seekers,
                shared::navigation_pathfinding::simple_pathfinding,
                shared::navigation_pathfinding::move_navigation_agents,
                shared::entities::enemy::enemy_navigation_behavior,
                shared::entities::enemy::enemy_attack_behavior,
            ),
        );
    }
}

/// Handle game progression - simplified for now
/// When lobby has players, auto-progress after a delay
fn handle_game_progression(mut lobby_state: ResMut<LobbyState>, mut commands: Commands) {
    // Auto-start game when host is present (simplified logic)
    if !lobby_state.players.is_empty() && !lobby_state.game_started {
        info!("Starting game with {} players", lobby_state.players.len());

        // Mark game as started to prevent re-triggering
        lobby_state.game_started = true;

        // Generate game seed using a simple method
        let seed = 42; // Simple seed for now
        let seed_message = GameSeed { seed };

        commands.insert_resource(seed_message);
        commands.set_state(GameState::Loading);
    }
}

/// System that runs when the server enters Loading state
/// Generates and distributes the game seed to all clients
fn start_game_with_seed(mut commands: Commands) {
    info!("Server entered Loading state - preparing game world");

    // Transition to spawning after a brief delay
    std::thread::sleep(std::time::Duration::from_millis(100));
    commands.set_state(GameState::Spawning);
}

/// System that runs when the server enters Spawning state
/// Creates the actual game world
fn spawn_game_world(game_seed: Option<Res<GameSeed>>, mut commands: Commands) {
    info!("Server spawning game world");

    let seed = game_seed.map(|s| s.seed).unwrap_or(42);
    info!("Using seed {} for world generation", seed);

    // Create the static level using the seed
    shared::level::create_static::setup_static_level(commands.reborrow(), Some(seed));

    // Player entities are spawned automatically by handle_connected observer
    // Dynamic enemies and other entities can be added here in the future

    info!("Server game world spawned, transitioning to Playing state");
    commands.set_state(shared::game_state::GameState::Playing);
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
    mut commands: Commands,
    mut lobby_state: ResMut<LobbyState>,
) {
    let Ok(client_id) = query.get(trigger.entity) else {
        info!(
            "❌ Failed to get RemoteId for connected entity {:?}",
            trigger.entity
        );
        return;
    };
    let peer_id = client_id.0;
    info!(
        "✅ Client connected with client-id {client_id:?} (peer_id: {}). Adding to lobby.",
        peer_id
    );

    // Add player to lobby state
    lobby_state.players.push(peer_id);
    info!(
        "🎪 Added player {} to lobby. Lobby now has {} players",
        peer_id,
        lobby_state.players.len()
    );

    let color = color_from_id(client_id.to_bits());
    let angle: f32 = client_id.to_bits() as f32 * 6.28 / 4.0; // Distribute around circle
    let x = 5.0 * angle.cos();
    let z = 5.0 * angle.sin();
    let y = PLAYER_CAPSULE_HEIGHT + 10.0;

    info!(
        "🎯 Setting up prediction target for client_id: {:?} (peer_id: {})",
        client_id, peer_id
    );

    let player = commands
        .spawn((
            Name::new(format!("Player_{}", client_id.to_bits())),
            PlayerId(peer_id),
        ))
        .insert(LinearVelocity::default())
        .insert(Position(Vec3::new(x, y, z)))
        .insert(Rotation::default())
        .insert(PlayerColor(color))
        .insert(Health::with_regeneration(100.0, 10.0, 5.0))
        .insert(Respawnable::new(5.0))
        .insert(ControlledBy {
            owner: trigger.entity,
            lifetime: Default::default(),
        })
        .insert(Replicate::to_clients(NetworkTarget::All))
        .insert(PredictionTarget::to_clients(NetworkTarget::Single(peer_id)))
        .insert(InterpolationTarget::to_clients(
            NetworkTarget::AllExceptSingle(peer_id),
        ))
        .id();

    info!(
        "🌐 Player entity {:?} created for client {:?}",
        player, client_id
    );

    // Add physics bundle separately (not replicated - physics components are local only)
    commands
        .entity(player)
        .insert(shared::entities::player::PlayerPhysicsBundle::default());

    // Add weapon holder to player
    add_weapon_holder(&mut commands, player);

    // Add stamina system to player
    add_stamina_to_player(&mut commands, player);

    info!(
        "✅ Player entity {:?} fully configured for client {:?}",
        player, client_id
    );
}

pub fn server_player_movement(
    mut player_query: Query<
        (
            Entity,
            &mut Rotation,
            &mut LinearVelocity,
            &ActionState<PlayerAction>,
            Option<&StaminaEffects>,
        ),
        // Based on lightyear examples - avoid applying movement to predicted/confirmed entities
        // to prevent conflicts in host-server mode
        (
            With<PlayerId>,
            Without<Predicted>,
            Without<Confirmed<Position>>,
        ),
    >,
) {
    for (entity, mut rotation, mut velocity, action_state, _stamina_effects) in
        player_query.iter_mut()
    {
        let axis_pair = action_state.axis_pair(&PlayerAction::Move);
        if axis_pair != Vec2::ZERO || !action_state.get_pressed().is_empty() {
            debug!(
                "🖥️ SERVER: Processing movement for entity {:?} with axis {:?} and actions {:?}",
                entity,
                axis_pair,
                action_state.get_pressed()
            );
        }

        shared_player_movement_with_stamina(action_state, &mut rotation, &mut velocity);
    }
}
