pub mod camera;
pub mod entity;
pub mod input;
pub mod menu;
pub mod network;
use crate::camera::RenderPlugin;
use crate::entity::ClientEntitiesPlugin;
use crate::input::ClientInputPlugin;
use crate::menu::{ClientLobbyPlugin, LocalMenuPlugin};
use crate::network::ClientNetworkPlugin;
use bevy::log::LogPlugin;
use bevy::prelude::{App, AssetPlugin, DefaultPlugins, PluginGroup, Resource, States, default};
use bevy::state::app::AppExtStates;
use bevy::window::{Window, WindowPlugin};
use lightyear::prelude::client::ClientPlugins;
use std::time::Duration;

#[derive(Resource)]
pub struct LocalPlayerId(pub u64);

#[derive(States, Debug, Default, Clone, PartialEq, Eq, Hash)]
pub enum ClientGameState {
    LocalMenu,
    Connecting,
    #[default]
    Lobby,
    Loading,
    Spawning,
    Playing,
}

pub fn create_client_app(
    client_id: u64,
    asset_path: String,
    auto_host: bool,
    auto_join: bool,
    auto_start: bool,
) -> App {
    let mut client_app = App::new();
    let client_id = if client_id == 0 { 1 } else { client_id };
    client_app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: if auto_host {
                        format!("Game Test - Host {}", client_id)
                    } else {
                        format!("Game Test - Client {}", client_id)
                    },
                    resolution: (1280, 720).into(),
                    ..default()
                }),
                ..default()
            })
            .set(AssetPlugin {
                file_path: asset_path,
                ..Default::default()
            })
            .disable::<LogPlugin>(), // Disable LogPlugin to avoid conflict with lightyear logging
    );

    // IMPORTANT: SharedPlugin must be added BEFORE ClientPlugins
    // to ensure protocol registration happens before lightyear initialization
    client_app.add_plugins(shared::SharedPlugin);

    client_app.add_plugins(ClientPlugins {
        tick_duration: Duration::from_secs_f64(1.0 / shared::FIXED_TIMESTEP_HZ),
    });

    client_app.insert_resource(LocalPlayerId(client_id));
    client_app.add_plugins(ClientNetworkPlugin);
    client_app.add_plugins(ClientInputPlugin);
    client_app.add_plugins(RenderPlugin);
    client_app.add_plugins(ClientEntitiesPlugin);
    client_app.add_plugins(LocalMenuPlugin);
    client_app.add_plugins(ClientLobbyPlugin);

    if auto_join {
        if !auto_host {
            client_app.insert_resource(crate::network::AutoJoin(true));
        }
    }

    if auto_host {
        client_app.insert_resource(crate::menu::AutoHost(true));
    }

    if auto_start {
        client_app.insert_resource(crate::menu::AutoStart(true));
    }

    client_app.init_state::<ClientGameState>();
    client_app.insert_state(ClientGameState::LocalMenu);

    client_app
}
