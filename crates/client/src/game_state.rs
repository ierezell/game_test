use bevy::prelude::{
    App, AppExtStates, Commands, Entity, EventWriter, OnExit, Or, Plugin, Query, Res, ResMut,
    States, With,
};
use common::protocol::{ClientHostRequestShutdown, ReliableChannel};
use lightyear::prelude::{Confirmed, Predicted, Replicated};

#[derive(States, Debug, Default, Clone, PartialEq, Eq, Hash)]
pub enum GameState {
    #[default]
    MainMenu,
    ConnectingRemote, // Connection request sent to the server,
    Loading,          // Connected and server told us to load something
    Spawning,         // Loaded the assets, now wait for the Player to be replicated
    Playing,          // Player exists and we can give control to the client
}

pub struct GameLifecyclePlugin;

impl Plugin for GameLifecyclePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnExit(GameState::Playing), cleanup_on_exit_to_menu);
        app.init_state::<GameState>();
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
