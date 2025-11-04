use avian3d::prelude::Position;
use bevy::log::debug;
use bevy::prelude::{
    App, AppExtStates, Commands, CommandsStatesExt, Entity, OnExit, Or, Plugin, Query, Res, State,
    Update, With,
};
use lightyear::prelude::{Confirmed, Controlled, Predicted, Replicated};
use shared::game_state::GameState;
use shared::level::create_static::LevelDoneMarker;
use shared::protocol::PlayerId;

pub struct GameLifecyclePlugin;

impl Plugin for GameLifecyclePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnExit(GameState::Playing), cleanup_on_exit_to_menu);

        app.add_systems(Update, check_level_loaded);
        app.init_state::<GameState>();
    }
}

fn check_level_loaded(
    mut commands: Commands,
    current_state: Res<State<GameState>>,
    level_query: Query<Entity, With<LevelDoneMarker>>,
    controlled_player_query: Query<Entity, (With<PlayerId>, With<Controlled>, With<Predicted>)>,
    all_player_query: Query<Entity, With<PlayerId>>,
    predicted_query: Query<Entity, (With<PlayerId>, With<Predicted>)>,
    controlled_query: Query<Entity, (With<PlayerId>, With<Controlled>)>,
) {
    if *current_state.get() == GameState::Loading {
        let has_level = !level_query.is_empty();
        let has_controlled_player = !controlled_player_query.is_empty();
        let has_controlled_player_any = !controlled_query.is_empty();

        debug!(
            "🔍 Loading check - Level: {}, All Players: {}, Predicted Players: {}, Controlled Players: {}, Controlled+Predicted: {}",
            has_level,
            all_player_query.iter().count(),
            predicted_query.iter().count(),
            controlled_query.iter().count(),
            controlled_player_query.iter().count()
        );

        // Simplified condition - just need any controlled player (level check bypassed)
        if has_controlled_player || has_controlled_player_any {
            debug!(
                "🎮 Player loaded! Controlled Player: {} - Transitioning to Playing (bypassing level check temporarily)",
                if has_controlled_player {
                    "Predicted+Controlled"
                } else {
                    "Controlled"
                }
            );
            commands.set_state(GameState::Playing);
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
