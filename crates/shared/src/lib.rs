pub mod components;
pub mod entities;
pub mod gym;
pub mod inputs;
pub mod level;
pub mod navigation;
pub mod protocol;
pub mod render;

use avian3d::collision::CollisionDiagnostics;
use avian3d::dynamics::solver::SolverDiagnostics;
use avian3d::prelude::{PhysicsDiagnosticsPlugin, PhysicsPlugins};
use avian3d::spatial_query::SpatialQueryDiagnostics;

use bevy::prelude::{Plugin, Resource};

use std::net::SocketAddr;

use crate::inputs::SharedInputPlugin;

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

#[derive(bevy::prelude::Resource, Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum NetworkMode {
    #[default]
    Udp, // standard UDP networking (internet client server)
    Crossbeam, // for in-process messaging channel
    Local,     // for same-process in app communication
}

#[derive(Resource, Clone, Copy, Debug, Default)]
pub struct GymMode(pub bool);

pub struct SharedPlugin;
impl Plugin for SharedPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins(SharedInputPlugin);
        app.add_plugins(protocol::ProtocolPlugin);
        // Add diagnostics plugin and resource first so required resources exist
        app.add_plugins(PhysicsDiagnosticsPlugin);
        app.insert_resource(CollisionDiagnostics::default());
        app.insert_resource(SolverDiagnostics::default());
        app.insert_resource(SpatialQueryDiagnostics::default());
        app.add_plugins(PhysicsPlugins::default());
        app.add_plugins(navigation::NavigationPlugin);
        app.add_plugins(components::health::HealthPlugin);
        app.add_plugins(components::weapons::WeaponsPlugin);
    }
}
