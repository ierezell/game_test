use avian3d::prelude::Position;
use bevy::log::{debug, warn};
use bevy::prelude::{
    App, AppExtStates, Commands, CommandsStatesExt, Entity, OnEnter, OnExit, Or, Plugin, Query,
    Res, ResMut, Resource, State, Time, Timer, TimerMode, Update, With, info,
};
use lightyear::prelude::{Confirmed, Controlled, Predicted, Replicated};
use shared::game_state::GameState;
use shared::level::create_static::LevelDoneMarker;
use shared::protocol::{GameSeed, PlayerId};

#[derive(Resource)]
struct LoadingTimer {
    timer: Timer,
}

impl Default for LoadingTimer {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(5.0, TimerMode::Once), // 5 second timeout
        }
    }
}

pub struct GameLifecyclePlugin;

impl Plugin for GameLifecyclePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LoadingTimer>();
        app.add_systems(OnExit(GameState::Playing), cleanup_on_exit_to_menu);
        app.add_systems(OnEnter(GameState::Spawning), spawn_client_world);
        app.add_systems(Update, check_level_loaded);
        app.init_state::<GameState>();
    }
}

/// System that spawns client-side world when entering the Spawning state
fn spawn_client_world(game_seed: Option<Res<GameSeed>>, mut commands: Commands) {
    info!("Client spawning world");

    let seed = game_seed.map(|s| s.seed).unwrap_or(42);
    info!("Using seed {} for client world generation", seed);

    // Create the static level using the same seed as server
    shared::level::create_static::setup_static_level(commands.reborrow(), Some(seed));

    info!("Client world spawned, transitioning to Playing state");
    commands.set_state(GameState::Playing);
}

fn check_level_loaded(
    mut commands: Commands,
    current_state: Res<State<GameState>>,
    level_query: Query<Entity, With<LevelDoneMarker>>,
    controlled_player_query: Query<Entity, (With<PlayerId>, With<Controlled>, With<Predicted>)>,
    all_player_query: Query<Entity, With<PlayerId>>,
    predicted_query: Query<Entity, (With<PlayerId>, With<Predicted>)>,
    controlled_query: Query<Entity, (With<PlayerId>, With<Controlled>)>,
    mut loading_timer: ResMut<LoadingTimer>,
    time: Res<Time>,
) {
    if *current_state.get() == GameState::Loading {
        // Tick the loading timer
        loading_timer.timer.tick(time.delta());
        
        let has_level = !level_query.is_empty();
        let has_controlled_player = !controlled_player_query.is_empty();
        let has_controlled_player_any = !controlled_query.is_empty();
        let timeout_reached = loading_timer.timer.just_finished();

        debug!(
            "🔍 Loading check - Level: {}, All Players: {}, Predicted Players: {}, Controlled Players: {}, Controlled+Predicted: {}, Timeout: {:.1}s",
            has_level,
            all_player_query.iter().count(),
            predicted_query.iter().count(),
            controlled_query.iter().count(),
            controlled_player_query.iter().count(),
            loading_timer.timer.elapsed_secs()
        );

        // Transition if we have a controlled player OR timeout is reached
        if has_controlled_player || has_controlled_player_any || timeout_reached {
            if timeout_reached && !has_controlled_player_any {
                warn!("⏰ Loading timeout reached - proceeding without controlled player");
                warn!("🌍 Will show static world only");
            } else {
                debug!(
                    "🎮 Player loaded! Controlled Player: {} - Transitioning to Spawning",
                    if has_controlled_player {
                        "Predicted+Controlled"
                    } else {
                        "Controlled"
                    }
                );
            }
            commands.set_state(GameState::Spawning);
            // Reset the timer for next time
            loading_timer.timer.reset();
        }
    }
}

fn cleanup_on_exit_to_menu(
    mut commands: Commands,
    q_everything: Query<Entity, Or<(With<Predicted>, With<Confirmed<Position>>, With<Replicated>)>>,
) {
    println!("cleaning up on exit to menu");

    for thing in &q_everything {
        commands.entity(thing).despawn()
    }
}
