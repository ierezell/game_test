use bevy::prelude::*;

use bevy::window::{Window, WindowPlugin};

use lightyear::prelude::server::*;
// use lightyear::prelude::{DeltaManager, Server};

use std::time::Duration;

use crate::gameplay::ServerGameplayPlugin;
use crate::network::NetworkPlugin;
use crate::render::RenderPlugin;
use shared::SharedPlugin;

use bevy::prelude::{App, default};

#[derive(Resource, PartialEq, Eq, Clone, Debug)]
pub enum ServerMode {
    Windowed,
    Headless,
}

pub fn add_basics_to_server_app(app: &mut App, headless: bool) -> &mut App {
    if headless {
        app.add_plugins(DefaultPlugins);
    } else {
        app.add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Yolo Game - Server".to_string(),
                    resolution: (400., 200.).into(),
                    ..default()
                }),
                ..default()
            }),
            RenderPlugin,
        ));
    }
    app.add_plugins(SharedPlugin);
    app
}

pub fn add_network_to_server_app(app: &mut App) -> &mut App {
    app.add_plugins(ServerPlugins {
        // Lightyear plugins
        // tick_duration: Duration::from_secs_f64(1.0 / shared::FIXED_TIMESTEP_HZ),
        tick_duration: Duration::from_secs_f64(1.0 / shared::FIXED_TIMESTEP_HZ),
    });
    app.add_plugins((NetworkPlugin, ServerGameplayPlugin));

    // Delta compression
    // let server = app
    //     .world_mut()
    //     .query_filtered::<Entity, With<Server>>()
    //     .single(app.world_mut())
    //     .unwrap();

    // // set some input-delay since we are predicting all entities
    // app.world_mut()
    //     .entity_mut(server)
    //     .insert(DeltaManager::default());
    app
}
