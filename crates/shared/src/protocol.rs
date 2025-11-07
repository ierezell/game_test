use crate::input::PlayerAction;

use avian3d::prelude::{LinearVelocity, Position, Rotation};
use bevy::{
    log::debug,
    prelude::{App, Color, Component, Name, Plugin, Resource, default},
};
use leafwing_input_manager::action_state::ActionState;

use lightyear::input::config::InputConfig;
use lightyear::prelude::input::leafwing;
use lightyear::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PlayerId(pub u64);

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PlayerColor(pub Color);

// Server-only resource (not replicated)
#[derive(Resource, Clone, Debug, PartialEq)]
pub struct GameSeed {
    pub seed: u64,
}

// Server-only resource (not replicated)
#[derive(Resource, Clone, Debug, PartialEq)]
pub struct LobbyState {
    pub players: Vec<u64>,
    pub host_id: u64,
}

// Marker component for the game state entity
#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GameStateMarker;

// Replicated component that holds the seed - attached to GameStateMarker entity
#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ReplicatedGameSeed {
    pub seed: u64,
}

// Replicated component for lobby info - attached to GameStateMarker entity
#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ReplicatedLobbyInfo {
    pub player_count: u32,
    pub host_id: u64, // Using u64 instead of PeerId for simpler serialization
}

// Lightyear event: Client tells server to start the game
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct StartGameEvent;

// Lightyear event: Client confirms static world is created
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct WorldCreatedEvent {
    pub client_id: u64,
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
        app.register_component::<Name>().add_prediction();

        app.register_component::<Rotation>()
            .add_prediction()
            .add_linear_interpolation();

        app.register_component::<Position>()
            .add_prediction()
            .add_linear_interpolation();

        app.register_component::<LinearVelocity>().add_prediction();

        app.register_component::<ActionState<PlayerAction>>()
            .add_prediction();

        app.register_component::<GameStateMarker>();
        app.register_component::<ReplicatedGameSeed>();
        app.register_component::<ReplicatedLobbyInfo>();

        app.register_message::<StartGameEvent>();
        app.register_message::<WorldCreatedEvent>();

        debug!("✅ Protocol plugin initialized with components, messages, inputs, and events");
    }
}
