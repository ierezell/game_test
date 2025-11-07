use crate::LocalPlayerId;
use avian3d::prelude::Position;
use bevy::log::warn;
use bevy::prelude::{
    App, AppExtStates, Assets, Commands, CommandsStatesExt, Entity, Mesh, OnEnter, OnExit, Or,
    Plugin, Query, Res, ResMut, Resource, StandardMaterial, State, Time, Timer, TimerMode, Update,
    With, info,
};
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};
use leafwing_input_manager::prelude::ActionState;
use lightyear::prelude::{
    Confirmed, Controlled, MessageSender, Predicted, Replicated, UpdatesChannel,
};
use shared::game_state::GameState;
use shared::input::PlayerAction;
use shared::level::create_static::LevelDoneMarker;
use shared::protocol::{GameStateMarker, PlayerId, ReplicatedGameSeed, WorldCreatedEvent};

#[derive(Resource)]
struct LoadingTimer {
    timer: Timer,
}

impl Default for LoadingTimer {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(5.0, TimerMode::Once),
        }
    }
}

pub struct GameLifecyclePlugin;

impl Plugin for GameLifecyclePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LoadingTimer>();
        app.add_systems(OnExit(GameState::Playing), cleanup_on_exit_to_menu);
        app.add_systems(OnEnter(GameState::Loading), setup_client_loading);
        app.add_systems(OnEnter(GameState::Spawning), spawn_client_world);
        app.add_systems(OnEnter(GameState::Playing), setup_gameplay);
        app.add_systems(Update, check_players_spawned);

        // Monitor for ReplicatedGameSeed to know when to transition to Loading
        app.add_systems(Update, monitor_game_seed);

        app.init_state::<GameState>();
    }
}

/// Monitor for ReplicatedGameSeed from server and transition to Loading
fn monitor_game_seed(
    seed_query: Query<&ReplicatedGameSeed, With<GameStateMarker>>,
    current_state: Res<State<GameState>>,
    mut commands: Commands,
) {
    // Only check when in lobby
    if *current_state.get() != GameState::InLobby {
        return;
    }

    // If we see the seed, it means the server has started the game
    if let Some(_seed) = seed_query.iter().next() {
        info!("📨 CLIENT: Received ReplicatedGameSeed from server - transitioning to Loading");
        commands.set_state(GameState::Loading);
    }
}

/// System that sets up client loading state
fn setup_client_loading(mut commands: Commands) {
    info!("🔄 CLIENT: Entered Loading state");
    // Immediately transition to Spawning
    commands.set_state(GameState::Spawning);
}

/// System that spawns client-side world when entering the Spawning state
fn spawn_client_world(
    mut commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<StandardMaterial>>,
    level_query: Query<(), With<LevelDoneMarker>>,
    seed_query: Query<&ReplicatedGameSeed, With<GameStateMarker>>,
    mut message_sender: Query<&mut MessageSender<WorldCreatedEvent>>,
    local_player_id: Res<LocalPlayerId>,
) {
    // Check if world already exists to prevent re-spawning
    if !level_query.is_empty() {
        info!("⚠️ CLIENT: Static world already exists, skipping");
        return;
    }

    info!("🌍 CLIENT: Spawning static world");

    // Get seed from replicated component
    let seed = seed_query.iter().next().map(|s| s.seed).unwrap_or(42);

    info!("🎲 CLIENT: Using seed {} for world generation", seed);

    // Create the static level using the same seed as server
    shared::level::create_static::setup_static_level(
        commands.reborrow(),
        meshes,
        materials,
        Some(seed),
    );

    info!("✅ CLIENT: Static world created, sending confirmation to server");

    // Send WorldCreatedEvent to server
    if let Ok(mut sender) = message_sender.single_mut() {
        sender.send::<UpdatesChannel>(WorldCreatedEvent {
            client_id: local_player_id.0,
        });
        info!("📡 CLIENT: Sent WorldCreatedEvent to server");
    }

    // Now we wait for players to be replicated from server
    // The check_players_spawned system will transition to Playing when ready
}

/// Check if players have been spawned and transition to Playing state
fn check_players_spawned(
    mut commands: Commands,
    current_state: Res<State<GameState>>,
    player_query: Query<(), (With<PlayerId>, With<Predicted>, With<Controlled>)>,
    mut loading_timer: ResMut<LoadingTimer>,
    time: Res<Time>,
) {
    // Only check when in Spawning state
    if *current_state.get() != GameState::Spawning {
        return;
    }

    loading_timer.timer.tick(time.delta());

    let has_player = !player_query.is_empty();
    let timeout = loading_timer.timer.just_finished();

    if has_player {
        info!("✅ CLIENT: Player entity received, transitioning to Playing");
        commands.set_state(GameState::Playing);
        loading_timer.timer.reset();
    } else if timeout {
        warn!("⏰ CLIENT: Loading timeout - transitioning to Playing anyway");
        commands.set_state(GameState::Playing);
        loading_timer.timer.reset();
    }
}

/// System that sets up gameplay when entering Playing state
fn setup_gameplay(
    mut cursor_options_query: Query<&mut CursorOptions, With<PrimaryWindow>>,
    mut action_query: Query<
        &mut ActionState<PlayerAction>,
        (With<PlayerId>, With<Predicted>, With<Controlled>),
    >,
) {
    info!("🎮 CLIENT: Entering Playing state - setting up controls");

    // Lock cursor for FPS gameplay
    if let Ok(mut cursor_options) = cursor_options_query.single_mut() {
        cursor_options.grab_mode = CursorGrabMode::Locked;
        cursor_options.visible = false;
        info!("🔒 CLIENT: Cursor locked for gameplay");
    }

    // Enable player input
    if let Ok(mut action_state) = action_query.single_mut() {
        action_state.enable();
        info!("🎮 CLIENT: Player controls enabled");
    }
}

fn cleanup_on_exit_to_menu(
    mut commands: Commands,
    q_everything: Query<Entity, Or<(With<Predicted>, With<Confirmed<Position>>, With<Replicated>)>>,
) {
    info!("🧹 CLIENT: Cleaning up on exit to menu");

    for entity in &q_everything {
        commands.entity(entity).despawn();
    }
}
