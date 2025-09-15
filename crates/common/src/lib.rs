use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use avian3d::{PhysicsPlugins, prelude::PhysicsInterpolationPlugin};
use bevy::prelude::{App, FixedPostUpdate, FixedUpdate, Or, Plugin, PluginGroup, Resource, With};
use leafwing_input_manager::plugin::InputManagerPlugin;
use lightyear::prelude::{Interpolated, PreSpawned, Predicted, Replicated, ReplicationGroup};
use protocol::ProtocolPlugin;
pub mod game_state;
pub mod input;
pub mod protocol;
pub struct CommonPlugin;
use crate::input::NetworkedInput;

// Server binds to all interfaces (0.0.0.0) to accept connections from any IP
pub const SERVER_BIND_ADDR: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 5001);
// Client connects to localhost (127.0.0.1) for local testing
pub const SERVER_ADDR: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 5001);
pub const CLIENT_ADDR: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 4000);
pub const PRIVATE_KEY: [u8; 32] = [0u8; 32];
pub const PROTOCOL_ID: u64 = 0x1122334455667788;

/// Supported transports for Lightyear networking
#[derive(Clone, Debug, Resource)]
pub enum NetTransport {
    Udp,
    // TODO: Enable these transports by adding the correct Cargo features and imports
    // Crossbeam,
    // WebTransport,
    // WebSocket,
}

impl Plugin for CommonPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            InputManagerPlugin::<NetworkedInput>::default(),
            ProtocolPlugin,
            PhysicsPlugins::new(FixedPostUpdate)
                .build()
                .disable::<PhysicsInterpolationPlugin>(),
        ));
        app.add_systems(FixedUpdate, input::shared_player_movement);
    }
}

pub const REPLICATION_GROUP_PREDICTED: ReplicationGroup = ReplicationGroup::new_id(42);

pub type Simulated = Or<(With<Predicted>, With<PreSpawned>, With<Replicated>)>;
pub type Rendered = Or<(Simulated, With<Interpolated>)>;
