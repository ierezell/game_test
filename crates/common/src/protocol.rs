//! Network protocol for multiplayer functionality.
//! Based on Lightyear 0.23.0 API following examples from the official repository.

use crate::input;

use avian3d::prelude::{Position, Rotation};
use bevy::prelude::{
    App, Color, Component, Entity, Event, Plugin, Resource, Transform, Vec2, Vec3, default, info,
};

use lightyear::prelude::*;
use lightyear::prelude::{
    AppChannelExt, AppMessageExt, ChannelMode, ChannelSettings, NetworkDirection, PeerId,
    ReliableSettings,
};
use serde::{Deserialize, Serialize};

#[derive(Resource, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PlayerId(pub u64);

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PlayerColor(pub Color);

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

// Additional message types for better multiplayer experience
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

// Channel types
pub struct UnorderedReliableChannel;
pub struct ReliableChannel;
pub struct UnreliableChannel;

// Protocol plugin
#[derive(Clone)]
pub struct ProtocolPlugin;

impl Plugin for ProtocolPlugin {
    fn build(&self, app: &mut App) {
        // Register components, messages, and inputs using the dedicated modules
        // Register PlayerColor component
        app.register_component::<PlayerColor>()
            .add_prediction(PredictionMode::Once)
            .add_interpolation(InterpolationMode::Once);

        // Register physics components if available
        app.register_component::<Position>()
            .add_prediction(PredictionMode::Full)
            .add_interpolation(InterpolationMode::Full)
            .add_interpolation_fn(|start, end, t| Position(start.0.lerp(end.0, t)))
            .add_correction_fn(|start, end, t| Position(start.0.lerp(end.0, t)));

        app.register_component::<Rotation>()
            .add_prediction(PredictionMode::Full)
            .add_interpolation(InterpolationMode::Full)
            .add_interpolation_fn(|start, end, t| Rotation(*start.slerp(end, t)))
            .add_correction_fn(|start, end, t| Rotation(*start.slerp(end, t)));

        // Register events
        app.add_event::<PlayerMovedEvent>()
            .add_event::<SoundTriggeredEvent>();

        // Existing messages
        app.add_message::<ServerWelcome>()
            .add_direction(NetworkDirection::ServerToClient);
        app.add_message::<ClientLevelLoadComplete>()
            .add_direction(NetworkDirection::ClientToServer);
        app.add_message::<ClientHostRequestShutdown>()
            .add_direction(NetworkDirection::ClientToServer);

        // New messages
        app.add_message::<PlayerJoined>()
            .add_direction(NetworkDirection::ServerToClient);
        app.add_message::<PlayerLeft>()
            .add_direction(NetworkDirection::ServerToClient);
        app.add_message::<ChatMessage>()
            .add_direction(NetworkDirection::Bidirectional);

        // Channel configurations
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
