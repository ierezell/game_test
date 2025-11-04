use bevy::prelude::States;

#[derive(States, Debug, Default, Clone, PartialEq, Eq, Hash)]
pub enum GameState {
    #[default]
    MainMenu,
    HostingLobby,
    JoiningGame,
    InLobby,
    Loading,
    Spawning,
    Playing,
    Connecting,
}
