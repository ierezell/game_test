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

pub struct RenderPlugin;

fn spawn_camera_if_none_exists(
    mut commands: Commands,
    existing_cameras: Query<Entity, With<Camera3d>>,
) {
    // Only spawn camera if none exists to avoid order ambiguities
    if existing_cameras.is_empty() {
        commands.spawn((
            Camera3d::default(),
            Camera {
                order: 0, // Set order 0 for render camera
                ..default()
            },
            Transform::from_xyz(-50.0, 50.0, 50.0).looking_at(Vec3::ZERO, Vec3::Y),
            Name::new("RenderCamera"),
        ));
    }
}

// If the headless server can't run it or doesn't need it
// It goes in this plugin
impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_camera_if_none_exists);
        app.add_plugins((
            //PhysicsDebugPlugin::default(),
            EguiPlugin::default(),
            WorldInspectorPlugin::default(),
        ));
    }
}
