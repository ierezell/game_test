use bevy::prelude::{App, DefaultPlugins, PluginGroup, Resource, default};
use bevy::window::{Window, WindowPlugin};
use lightyear::prelude::server::ServerPlugins;
use std::time::Duration;

use crate::gameplay::ServerGameplayPlugin;
use crate::network::NetworkPlugin;
use crate::render::RenderPlugin;
use shared::SharedPlugin;

/// Determines server rendering mode
#[derive(Resource, PartialEq, Eq, Clone, Debug)]
pub enum ServerMode {
    /// Server with window for debugging
    Windowed,
    /// Headless server for production
    Headless,
}

/// Configure basic server application with appropriate plugins based on mode
///
/// # Arguments
/// * `app` - The Bevy app to configure
/// * `headless` - Whether to run in headless mode (no window)
pub fn add_basics_to_server_app(app: &mut App, headless: bool) -> &mut App {
    if headless {
        app.add_plugins(DefaultPlugins);
    } else {
        app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Yolo Game - Server".to_string(),
                resolution: (400, 200).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(RenderPlugin);
    }
    app.add_plugins(SharedPlugin);
    app
}

/// Add networking capabilities to the server application
///
/// Configures Lightyear server plugins with appropriate tick duration
/// and adds custom networking and gameplay plugins.
pub fn add_network_to_server_app(app: &mut App) -> &mut App {
    app.add_plugins(ServerPlugins {
        tick_duration: Duration::from_secs_f64(1.0 / shared::FIXED_TIMESTEP_HZ),
    });
    app.add_plugins(NetworkPlugin)
        .add_plugins(ServerGameplayPlugin);
    app
}
