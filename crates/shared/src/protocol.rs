use crate::{
    input::PlayerAction,
    navigation::{PatrolRoute, PatrolState, SimpleNavigationAgent},
};
use avian3d::prelude::{LinearVelocity, Position, Rotation};
use bevy::{
    log::debug,
    prelude::{App, Color, Component, Name, Plugin, default},
};

use lightyear::prelude::{
    AppComponentExt, AppMessageExt, InterpolationRegistrationExt, NetworkDirection, PeerId,
    PredictionRegistrationExt, input::leafwing::InputPlugin,
};

use lightyear::input::config::InputConfig;

use serde::{Deserialize, Serialize};

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PlayerId(pub PeerId);

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PlayerColor(pub Color);

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CharacterMarker;

#[derive(Component, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GameSeed {
    pub seed: u64,
}

#[derive(Component, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LobbyState {
    pub players: Vec<u64>,
    pub host_id: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ClientWorldCreatedEvent {
    pub client_id: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct HostStartGameEvent;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct StartLoadingGameEvent;

#[derive(Clone)]
pub struct ProtocolPlugin;
impl Plugin for ProtocolPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputPlugin::<PlayerAction> {
            config: InputConfig::<PlayerAction> {
                rebroadcast_inputs: true,
                lag_compensation: true,
                ..default()
            },
        });

        app.insert_resource(avian3d::physics_transform::PhysicsTransformConfig {
            transform_to_position: false,
            position_to_transform: true,
            ..default()
        });

        app.register_component::<PlayerId>();
        app.register_component::<Name>();
        app.register_component::<PlayerColor>();
        app.register_component::<GameSeed>();
        app.register_component::<CharacterMarker>();

        app.register_component::<Rotation>()
            .add_prediction()
            .add_linear_interpolation();

        app.register_component::<Position>()
            .add_prediction()
            .add_linear_interpolation();

        app.register_component::<LinearVelocity>().add_prediction();

        // Navigation components for debug visualization on client
        app.register_component::<SimpleNavigationAgent>();
        app.register_component::<PatrolRoute>();
        app.register_component::<PatrolState>();

        app.register_component::<LobbyState>();

        app.register_message::<ClientWorldCreatedEvent>()
            .add_direction(NetworkDirection::ClientToServer);

        app.register_message::<HostStartGameEvent>()
            .add_direction(NetworkDirection::ClientToServer);

        app.register_message::<StartLoadingGameEvent>()
            .add_direction(NetworkDirection::ServerToClient);

        debug!("✅ Protocol plugin initialized with components, messages, inputs, and events");
    }
}
