use crate::input::PlayerAction;
use crate::level::create_static::LevelDoneMarker;

use avian3d::prelude::{LinearVelocity, Position, Rotation};
use bevy::{
    log::debug,
    prelude::{App, Color, Component, Message, Name, Plugin, Resource, default},
};

use lightyear::input::config::InputConfig;
use lightyear::prelude::PeerId;
use lightyear::prelude::input::leafwing;
use lightyear::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PlayerId(pub PeerId);

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PlayerColor(pub Color);

#[derive(Message, Resource, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GameSeed {
    pub seed: u64,
}

#[derive(Message, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct StartGame;

#[derive(Message, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PlayerJoinedLobby {
    pub player_id: PeerId,
    pub player_name: String,
}

#[derive(Resource, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct LobbyState {
    pub players: Vec<PeerId>,
    pub host_id: PeerId,
    pub game_started: bool,
}

#[derive(Clone)]
pub struct ProtocolPlugin;

impl Plugin for ProtocolPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(leafwing::InputPlugin::<PlayerAction> {
            config: InputConfig::<PlayerAction> {
                rebroadcast_inputs: false,
                lag_compensation: true,
                ..default()
            },
        });

        app.register_component::<PlayerId>().add_prediction();
        app.register_component::<PlayerColor>().add_prediction();
        app.register_component::<Name>().add_prediction();
        app.register_component::<LevelDoneMarker>().add_prediction();

        app.register_component::<Rotation>().add_prediction();
        app.register_component::<Position>().add_prediction();
        app.register_component::<LinearVelocity>().add_prediction();

        // Register messages
        app.register_message::<GameSeed>();
        app.register_message::<StartGame>();
        app.register_message::<PlayerJoinedLobby>();

        debug!("✅ Protocol plugin initialized with components, messages, inputs, and events");
    }
}
