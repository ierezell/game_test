use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::Duration;

use bevy::prelude::{App, Plugin, Resource};

use avian3d::PhysicsPlugins;

use protocol::ProtocolPlugin;

pub mod ai_bot;
pub mod enemy;
pub mod entity_implementations;
pub mod entity_spawner;
pub mod entity_traits;
pub mod game_state;
pub mod health;
pub mod input;
pub mod navigation_pathfinding;
pub mod protocol;

pub mod render;
pub mod scene;
pub mod stamina;
pub mod weapons;

#[cfg(test)]
mod tests;

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
        app.add_plugins(crate::ai_bot::BotPlugin);
        app.add_plugins(crate::enemy::EnemyPlugin);
        app.add_plugins(crate::navigation_pathfinding::NavigationPlugin);
        app.add_plugins(crate::health::HealthPlugin);
        app.add_plugins(crate::weapons::WeaponPlugin);
        app.add_plugins(crate::stamina::StaminaPlugin);
    }
}
