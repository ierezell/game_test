use bevy::log::debug;
use bevy::prelude::{
    App, AppExtStates, Commands, CommandsStatesExt, Entity, OnExit, Or, Plugin, Query, Res, State,
    States, Update, With,
};
use lightyear::prelude::{Confirmed, Controlled, Predicted, Replicated};
use shared::protocol::PlayerId;
use shared::scene::{FloorMarker, WallMarker};

#[derive(States, Debug, Default, Clone, PartialEq, Eq, Hash)]
pub enum GameState {
    #[default]
    MainMenu,
    ConnectingRemote, // Connection request sent to the server,
    Loading,          // Connected and waiting for scene and player to be replicated
    Playing,          // Scene and player are loaded, ready to play
}

pub struct GameLifecyclePlugin;

impl Plugin for GameLifecyclePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnExit(GameState::Playing), cleanup_on_exit_to_menu);
        app.add_systems(Update, check_assets_loaded);
        app.init_state::<GameState>();
    }
}

fn check_assets_loaded(
    mut commands: Commands,
    current_state: Res<State<GameState>>,
    floor_query: Query<Entity, With<FloorMarker>>,
    wall_query: Query<Entity, With<WallMarker>>,
    controlled_player_query: Query<Entity, (With<PlayerId>, With<Controlled>, With<Predicted>)>,
    all_player_query: Query<Entity, With<PlayerId>>,
    predicted_query: Query<Entity, (With<PlayerId>, With<Predicted>)>,
    controlled_query: Query<Entity, (With<PlayerId>, With<Controlled>)>,
) {
    if *current_state.get() == GameState::Loading {
        let has_floor = !floor_query.is_empty();
        let has_walls = wall_query.iter().count() >= 4;
        let has_controlled_player = !controlled_player_query.is_empty();
        let has_controlled_player_any = !controlled_query.is_empty();

        debug!(
            "🔍 Loading check - Floor: {}, Walls: {}, All Players: {}, Predicted Players: {}, Controlled Players: {}, Controlled+Predicted: {}",
            has_floor,
            wall_query.iter().count(),
            all_player_query.iter().count(),
            predicted_query.iter().count(),
            controlled_query.iter().count(),
            controlled_player_query.iter().count()
        );

        if has_floor && has_walls && (has_controlled_player || has_controlled_player_any) {
            debug!(
                "🎮 Scene and player loaded! Floor: {}, Walls: {}, Controlled Player: {} - Transitioning to Playing",
                has_floor,
                wall_query.iter().count(),
                if has_controlled_player {
                    controlled_player_query.iter().count()
                } else {
                    controlled_query.iter().count()
                }
            );
            commands.set_state(GameState::Playing);
        }
    }
}

fn cleanup_on_exit_to_menu(
    mut commands: Commands,
    q_everything: Query<Entity, Or<(With<Predicted>, With<Confirmed>, With<Replicated>)>>,
) {
    println!("cleaning up on exit to menu");

    for thing in &q_everything {
        commands.entity(thing).despawn()
    }
}
