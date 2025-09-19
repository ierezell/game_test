use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::Duration;

use bevy::prelude::{App, Or, Plugin, Resource, Vec3, With};

use avian3d::prelude::*;

use lightyear::prelude::{Interpolated, PreSpawned, Predicted, Replicated, ReplicationGroup};
use protocol::ProtocolPlugin;
pub mod game_state;
pub mod input;
pub mod protocol;
pub mod render;
pub mod scene;

pub struct SharedSettings {
    pub protocol_id: u64,
    pub private_key: [u8; 32],
}

pub const SHARED_SETTINGS: SharedSettings = SharedSettings {
    protocol_id: 0x1122334455667788,
    private_key: [0; 32],
};

pub const FIXED_TIMESTEP_HZ: f64 = 64.0;
pub const SEND_INTERVAL: Duration = Duration::from_millis(100);

pub const SERVER_BIND_ADDR: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 5001);

pub const SERVER_ADDR: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 5001);
pub const CLIENT_ADDR: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 4000);

#[derive(Clone, Debug, Resource)]
pub enum NetTransport {
    Udp,
    // TODO: Enable these transports by adding the correct Cargo features and imports
    // Crossbeam,
    // WebTransport,
    // WebSocket,
}

#[derive(Clone)]
pub struct SharedPlugin;

impl Plugin for SharedPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((ProtocolPlugin, PhysicsPlugins::default()));

        // Configure gravity for realistic physics simulation
        app.insert_resource(Gravity(Vec3::new(0.0, -9.81, 0.0)));
    }
}

pub const REPLICATION_GROUP_PREDICTED: ReplicationGroup = ReplicationGroup::new_id(42);

pub type Simulated = Or<(With<Predicted>, With<PreSpawned>, With<Replicated>)>;
pub type Rendered = Or<(Simulated, With<Interpolated>)>;
