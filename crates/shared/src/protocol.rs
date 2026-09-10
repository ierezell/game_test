use crate::{
    components::{
        flashlight::PlayerFlashlight,
        health::{Health, Respawnable},
        weapons::{Gun, Projectile, ProjectileGun},
    },
    inputs::movement::GroundState,
    navigation::{PatrolRoute, PatrolState, SimpleNavigationAgent},
};
use avian3d::prelude::{LinearVelocity, Position, Rotation};
use bevy::{
    log::debug,
    prelude::{App, Color, Component, Name, Plugin, default},
    reflect::TypePath,
};

use lightyear::prelude::{
    AppChannelExt, AppComponentExt, AppMessageExt, ChannelMode, ChannelSettings,
    InterpolationRegistrationExt, NetworkDirection, PeerId, ReliableSettings,
    PredictionBuilderExt,
};

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

/// LevelSeed component - replicated from server to clients
/// Used to synchronize procedural level generation across the network
#[derive(Component, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LevelSeed {
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
pub struct HostStartGameEvent {
    pub requested: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct StartLoadingGameEvent {
    pub start: bool,
}

#[derive(TypePath)]
pub struct LobbyControlChannel;

#[derive(Clone)]
pub struct ProtocolPlugin;
impl Plugin for ProtocolPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(avian3d::physics_transform::PhysicsTransformConfig {
            transform_to_position: false,
            position_to_transform: true,
            ..default()
        });

        app.component::<PlayerId>().replicate();
        app.component::<Name>().replicate();
        app.component::<PlayerColor>().replicate();
        app.component::<GameSeed>().replicate();
        app.component::<LevelSeed>().replicate();
        app.component::<CharacterMarker>().replicate();

        app.component::<Rotation>()
            .replicate()
            .predict()
            .add_linear_interpolation()
            .add_correction_fn(|start: Rotation, end: Rotation, t| {
                start.slerp(end, t)
            });

        app.component::<Position>()
            .replicate()
            .predict()
            .add_linear_interpolation()
            .add_correction_fn(|start: Position, end: Position, t| {
                Position(start.0.lerp(end.0, t))
            });

        app.component::<LinearVelocity>().replicate().predict();

        app.component::<GroundState>().replicate().predict();

        // Health and weapon components
        app.component::<Health>().replicate().predict();
        app.component::<Respawnable>().replicate();
        app.component::<Gun>().replicate().predict();
        app.component::<ProjectileGun>().replicate().predict();
        app.component::<Projectile>().replicate().predict();

        app.component::<PlayerFlashlight>()
            .replicate()
            .predict();

        app.component::<SimpleNavigationAgent>().replicate();
        app.component::<PatrolRoute>().replicate();
        app.component::<PatrolState>().replicate();

        app.component::<LobbyState>().replicate();

        app.add_channel::<LobbyControlChannel>(ChannelSettings {
            mode: ChannelMode::OrderedReliable(ReliableSettings::default()),
            ..default()
        })
        .add_direction(NetworkDirection::Bidirectional);

        // Events
        app.register_message::<ClientWorldCreatedEvent>()
            .add_direction(NetworkDirection::ClientToServer);

        app.register_message::<HostStartGameEvent>()
            .add_direction(NetworkDirection::ClientToServer);

        app.register_message::<StartLoadingGameEvent>()
            .add_direction(NetworkDirection::ServerToClient);

        debug!("Protocol plugin initialized with components, messages, inputs, and events");
    }
}