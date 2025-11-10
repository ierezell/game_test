pub mod entities;
pub mod input;
pub mod level;
pub mod navigation_pathfinding;
pub mod protocol;
pub mod render;
use avian3d::prelude::PhysicsPlugins;

use bevy::prelude::Plugin;

use std::net::SocketAddr;

pub const SEND_INTERVAL: std::time::Duration = std::time::Duration::from_millis(16);
pub const SERVER_BIND_ADDR: SocketAddr = SocketAddr::new(
    std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)),
    8080,
);
pub struct SharedSettings {
    pub private_key: [u8; 32],
    pub protocol_id: u64,
}
pub const SHARED_SETTINGS: SharedSettings = SharedSettings {
    private_key: [0u8; 32], // dummy 32-byte key
    protocol_id: 42,
};
pub const SERVER_ADDR: SocketAddr = SocketAddr::new(
    std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)),
    8080,
);
pub const FIXED_TIMESTEP_HZ: f64 = 60.0;

pub struct SharedPlugin;
impl Plugin for SharedPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins(protocol::ProtocolPlugin);
        app.add_plugins(PhysicsPlugins::default());
    }
}
