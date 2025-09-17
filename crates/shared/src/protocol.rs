use crate::input::{PlayerAction, PlayerVelocity};
use crate::scene::*;

use avian3d::prelude::{Position, Rotation};
use bevy::prelude::{App, Entity, Event, Name, Plugin, Resource, Vec3, default, info};
use leafwing_input_manager::prelude::ActionState;

use lightyear::input::prelude::InputConfig;
use lightyear::prelude::input::leafwing;
use lightyear::prelude::*;
use lightyear::prelude::{
    AppChannelExt, AppMessageExt, ChannelMode, ChannelSettings, NetworkDirection, PeerId,
    ReliableSettings,
};
use serde::{Deserialize, Serialize};

#[derive(Resource, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PlayerId(pub PeerId);

#[derive(Event, Debug, Clone)]
pub struct PlayerMovedEvent {
    pub entity: Entity,
}

#[derive(Event, Debug, Clone)]
pub struct SoundTriggeredEvent {
    pub position: Vec3,
    pub sound_type: String,
    pub intensity: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash, Copy, Default)]
pub enum Level {
    #[default]
    Void, // No level is loaded
    Example,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ServerWelcome {
    pub current_level: Level,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ClientLevelLoadComplete;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ClientHostRequestShutdown;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PlayerJoined {
    pub player_id: PeerId,
    pub player_name: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PlayerLeft {
    pub player_id: PeerId,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ChatMessage {
    pub player_id: PeerId,
    pub message: String,
}

pub struct UnorderedReliableChannel;
pub struct ReliableChannel;
pub struct UnreliableChannel;

#[derive(Clone)]
pub struct ProtocolPlugin;

impl Plugin for ProtocolPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(leafwing::InputPlugin::<PlayerAction> {
            config: InputConfig::<PlayerAction> {
                rebroadcast_inputs: true,
                ..default()
            },
        });

        app.register_component::<PlayerMarker>()
            .add_prediction(PredictionMode::Once)
            .add_interpolation(InterpolationMode::Once);

        app.register_component::<FloorMarker>()
            .add_prediction(PredictionMode::Once);

        app.register_component::<WallMarker>()
            .add_prediction(PredictionMode::Once);

        app.register_component::<PlayerColor>()
            .add_prediction(PredictionMode::Once)
            .add_interpolation(InterpolationMode::Once);

        app.register_component::<Name>()
            .add_prediction(PredictionMode::Once);

        app.register_component::<Position>()
            .add_delta_compression()
            .add_prediction(PredictionMode::Full)
            .add_interpolation(InterpolationMode::Full)
            .add_linear_interpolation_fn();

        app.register_component::<Rotation>()
            .add_delta_compression()
            .add_prediction(PredictionMode::Full)
            .add_interpolation(InterpolationMode::Full)
            .add_linear_interpolation_fn();

        app.register_component::<PlayerVelocity>()
            .add_prediction(PredictionMode::Full);

        app.register_component::<ActionState<PlayerAction>>()
            .add_prediction(PredictionMode::Full);

        app.add_event::<PlayerMovedEvent>()
            .add_event::<SoundTriggeredEvent>();

        app.add_message::<ServerWelcome>()
            .add_direction(NetworkDirection::ServerToClient);
        app.add_message::<ClientLevelLoadComplete>()
            .add_direction(NetworkDirection::ClientToServer);
        app.add_message::<ClientHostRequestShutdown>()
            .add_direction(NetworkDirection::ClientToServer);

        app.add_message::<PlayerJoined>()
            .add_direction(NetworkDirection::ServerToClient);
        app.add_message::<PlayerLeft>()
            .add_direction(NetworkDirection::ServerToClient);
        app.add_message::<ChatMessage>()
            .add_direction(NetworkDirection::Bidirectional);

        app.add_channel::<UnorderedReliableChannel>(ChannelSettings {
            mode: ChannelMode::UnorderedReliable(ReliableSettings::default()),
            ..default()
        });

        app.add_channel::<ReliableChannel>(ChannelSettings {
            mode: ChannelMode::OrderedReliable(ReliableSettings::default()),
            ..default()
        });

        app.add_channel::<UnreliableChannel>(ChannelSettings {
            mode: ChannelMode::UnorderedUnreliable,
            ..default()
        });
        info!("✅ Protocol plugin initialized with components, messages, inputs, and events");
    }
}
