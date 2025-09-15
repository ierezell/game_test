use bevy::app::ScheduleRunnerPlugin;
use bevy::asset::AssetPlugin;
use bevy::diagnostic::DiagnosticsPlugin;
use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy::transform::TransformPlugin;
use bevy::window::{Window, WindowPlugin};
use lightyear::netcode::Key;
use lightyear::prelude::input::InputBuffer;
use lightyear::prelude::server::*;
use lightyear::prelude::*;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::Duration;

use crate::network::NetworkPlugin;

use crate::render::RenderPlugin;
use common::CommonPlugin;
use common::NetTransport;
use common::protocol::ProtocolPlugin;

use avian3d::prelude::PhysicsDebugPlugin;
use bevy::{
    color::palettes::css::WHITE,
    prelude::{
        App, Camera, Camera3d, Commands, Entity, Name, Plugin, Query, Startup, Transform, Vec3,
        With, default,
    },
};
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};

#[derive(Resource, PartialEq, Eq, Clone, Debug)]
pub enum ServerMode {
    Windowed,
    Headless,
}

pub fn add_basics_to_server_app(app: &mut App, asset_path: String, headless: bool) -> &mut App {
    if headless {
        app.add_plugins((DefaultPlugins, CommonPlugin));
    } else {
        app.add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Server".to_string(),
                    ..default()
                }),
                ..default()
            }),
            CommonPlugin,
            RenderPlugin,
        ));
    }
    app
}

pub fn add_network_to_server_app(app: &mut App) -> &mut App {
    app.add_plugins(ServerPlugins {
        tick_duration: std::time::Duration::from_secs_f64(1.0 / 60.0),
        // TODO: Should the server and client duration be the same ?? If so, put it in common.
    });
    app.add_plugins(NetworkPlugin);

    app
}
