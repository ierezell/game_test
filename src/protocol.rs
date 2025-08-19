//! Network protocol for multiplayer functionality.
//! Based on Lightyear 0.23.0 API following examples from the official repository.

use crate::player::PlayerAction;
// use avian3d::prelude::*; // Commented out since Avian3D components don't have serde support yet
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;
use lightyear::prelude::*;
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::Duration;

// Network configuration constants
pub const FIXED_TIMESTEP_HZ: f64 = 64.0;
pub const SERVER_REPLICATION_INTERVAL: Duration = Duration::from_millis(100);
pub const SERVER_PORT: u16 = 5000;
pub const CLIENT_PORT: u16 = 0; // 0 means OS assigns available port
pub const SERVER_ADDR: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), SERVER_PORT);

// Shared settings for authentication
pub const PROTOCOL_ID: u64 = 0x12345678;
pub const PRIVATE_KEY: [u8; 32] = [0; 32]; // In production, use proper key generation

// Components that will be networked
#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PlayerId(pub u64);

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PlayerHealth {
    pub current: f32,
    pub max: f32,
}

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PlayerStamina {
    pub current: f32,
    pub max: f32,
}

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct NetworkedWeapon {
    pub ammo: u32,
    pub max_ammo: u32,
    pub is_reloading: bool,
}

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct FlashlightState {
    pub is_on: bool,
}

// Position and rotation components for networking
#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PlayerPosition(pub Vec3);

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PlayerRotation(pub Quat);

// Generic networked health component
#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct NetworkedHealth {
    pub current: f32,
    pub max: f32,
}

// Enemy components
#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct EnemyPosition(pub Vec3);

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum EnemyType {
    Striker,
    Charger,
    Shooter,
}

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct NetworkedEnemyType(pub EnemyType);

// Messages (Events for local handling)
#[derive(Event, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct WeaponFireMessage {
    pub player_id: u64,
    pub origin: Vec3,
    pub direction: Vec3,
    pub damage: f32,
}

#[derive(Event, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PlayerConnectedMessage {
    pub player_id: u64,
    pub name: String,
}

#[derive(Event, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PlayerDisconnectedMessage {
    pub player_id: u64,
}

// Channels
pub struct ReliableChannel;

pub struct UnreliableChannel;

// Protocol plugin
#[derive(Clone)]
pub struct ProtocolPlugin;

impl Plugin for ProtocolPlugin {
    fn build(&self, app: &mut App) {
        // Register input system
        app.add_plugins(InputManagerPlugin::<PlayerAction>::default());
        
        // Register components for replication
        app.register_component::<PlayerId>();
        app.register_component::<PlayerHealth>();
        app.register_component::<PlayerStamina>();
        app.register_component::<NetworkedWeapon>();
        app.register_component::<FlashlightState>();
        
        // Note: Avian3D components need to have serde support
        // For now we skip them until avian3d adds serde support
        // app.register_component::<Position>();
        // app.register_component::<Rotation>();
        // app.register_component::<LinearVelocity>();
        // app.register_component::<AngularVelocity>();
        
        // Register messages
        app.add_message::<WeaponFireMessage>()
           .add_direction(NetworkDirection::ServerToClient);
        app.add_message::<PlayerConnectedMessage>()
           .add_direction(NetworkDirection::ServerToClient);
        app.add_message::<PlayerDisconnectedMessage>()
           .add_direction(NetworkDirection::ServerToClient);
        
        // Register channels
        app.add_channel::<ReliableChannel>(ChannelSettings {
            mode: ChannelMode::OrderedReliable(ReliableSettings::default()),
            ..Default::default()
        })
        .add_direction(NetworkDirection::Bidirectional);
        
        app.add_channel::<UnreliableChannel>(ChannelSettings {
            mode: ChannelMode::UnorderedUnreliable,
            ..Default::default()
        })
        .add_direction(NetworkDirection::Bidirectional);
        
        info!("✅ Protocol plugin initialized");
    }
}



// Bundle for a networked player
#[derive(Bundle)]
pub struct NetworkedPlayerBundle {
    pub player_id: PlayerId,
    pub health: PlayerHealth,
    pub stamina: PlayerStamina,
    pub weapon: NetworkedWeapon,
    pub flashlight: FlashlightState,
    // Note: Removed Avian3D physics components since they don't have serde support
    // pub position: Position,
    // pub rotation: Rotation,
    // pub velocity: LinearVelocity,
    // pub angular_velocity: AngularVelocity,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub name: Name,
    pub replicate: Replicate,
}

impl NetworkedPlayerBundle {
    pub fn new(player_id: u64, position: Vec3) -> Self {
        Self {
            player_id: PlayerId(player_id),
            health: PlayerHealth {
                current: 100.0,
                max: 100.0,
            },
            stamina: PlayerStamina {
                current: 100.0,
                max: 100.0,
            },
            weapon: NetworkedWeapon {
                ammo: 30,
                max_ammo: 30,
                is_reloading: false,
            },
            flashlight: FlashlightState { is_on: false },
            // Note: Removed Avian3D physics components for now since they don't have serde support
            // position: Position(position),
            // rotation: Rotation::default(),
            // velocity: LinearVelocity::default(),
            // angular_velocity: AngularVelocity::default(),
            transform: Transform::from_translation(position),
            global_transform: GlobalTransform::default(),
            name: Name::new(format!("Networked Player {}", player_id)),
            replicate: Replicate::to_clients(NetworkTarget::All),
        }
    }
}
