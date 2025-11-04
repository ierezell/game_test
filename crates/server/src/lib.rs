pub mod gameplay;
pub mod network;
pub mod render;

pub fn create_server_app(headless: bool) -> bevy::prelude::App {
    use bevy::prelude::{DefaultPlugins, MinimalPlugins, PluginGroup, default};
    use bevy::state::app::StatesPlugin;
    use bevy::window::{Window, WindowPlugin};
    use lightyear::prelude::server::ServerPlugins;
    use std::time::Duration;

    use crate::gameplay::ServerGameplayPlugin;
    use crate::network::NetworkPlugin;
    use crate::render::RenderPlugin;
    use shared::SharedPlugin;
    let mut app = bevy::prelude::App::new();
    if headless {
        // Use MinimalPlugins for truly headless operation (no window, no rendering)
        app.add_plugins(MinimalPlugins);
        // Add StatesPlugin separately since MinimalPlugins doesn't include it
        app.add_plugins(StatesPlugin);
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
    app.add_plugins(ServerPlugins {
        tick_duration: Duration::from_secs_f64(1.0 / shared::FIXED_TIMESTEP_HZ),
    });
    app.add_plugins(NetworkPlugin)
        .add_plugins(ServerGameplayPlugin);
    app
}
