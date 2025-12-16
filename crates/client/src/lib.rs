pub mod camera;
pub mod debug;
pub mod entities;
pub mod game;
pub mod input;
pub mod lobby;
pub mod network;
use crate::camera::RenderPlugin;
use crate::debug::DebugPlugin;
use crate::entities::ClientEntitiesPlugin;
use crate::game::GameClientPlugin;
use crate::input::ClientInputPlugin;
use crate::lobby::ClientLobbyPlugin;
use crate::network::ClientNetworkPlugin;
use bevy::log::LogPlugin;
use bevy::prelude::{
    App, AssetApp, AssetPlugin, DefaultPlugins, Image, Mesh, PluginGroup, Resource, Shader,
    StandardMaterial, States, default,
};
use bevy::state::app::AppExtStates;
use bevy::window::{Window, WindowPlugin};

use lightyear::prelude::client::ClientPlugins;

use std::time::Duration;

#[derive(Resource)]
pub struct LocalPlayerId(pub u64);

#[derive(Resource)]
pub struct Headless(pub bool);

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

use shared::NetworkMode;

pub fn create_client_app(client_id: u64, asset_path: String, headless: bool, network_mode: NetworkMode) -> App {
    let mut client_app = App::new();
    let client_id = if client_id == 0 { 1 } else { client_id };
    client_app.insert_resource(Headless(headless));

    if headless {
        // Add AssetPlugin first to enable asset initialization
        client_app.add_plugins(AssetPlugin {
            file_path: asset_path.clone(),
            ..Default::default()
        });

        // Manually initialize assets that are usually added by RenderPlugin/PbrPlugin
        // This must happen BEFORE other plugins in DefaultPlugins (like UiPlugin) try to use them
        client_app.init_asset::<Mesh>();
        client_app.init_asset::<StandardMaterial>();
        client_app.init_asset::<Shader>();
        client_app.init_asset::<Image>();

        client_app.add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: None,
                    exit_condition: bevy::window::ExitCondition::DontExit,
                    ..default()
                })
                .disable::<AssetPlugin>() // Already added manually
                .disable::<LogPlugin>()
                .disable::<bevy::winit::WinitPlugin>()
                .disable::<bevy::render::RenderPlugin>()
                .disable::<bevy::pbr::PbrPlugin>()
                .disable::<bevy::sprite::SpritePlugin>()
                .disable::<bevy::audio::AudioPlugin>()
                .disable::<bevy::gilrs::GilrsPlugin>()
                .disable::<bevy::ui::UiPlugin>()
                .disable::<bevy::text::TextPlugin>(),
        );
    } else {
        client_app.add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: format!("Game Test - Client {}", client_id),
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
        client_app.add_plugins(RenderPlugin);
    }

    // IMPORTANT: SharedPlugin must be added BEFORE ClientPlugins
    // to ensure protocol registration happens before lightyear initialization
    client_app.insert_resource(network_mode);
    client_app.add_plugins(shared::SharedPlugin);
    client_app.add_plugins(ClientPlugins {
        tick_duration: Duration::from_secs_f64(1.0 / shared::FIXED_TIMESTEP_HZ),
    });

    client_app.insert_resource(LocalPlayerId(client_id));
    client_app.add_plugins(ClientNetworkPlugin);
    client_app.add_plugins(ClientInputPlugin);
    if !headless {
        client_app.add_plugins(DebugPlugin);
    }
    client_app.add_plugins(ClientEntitiesPlugin);
    client_app.add_plugins(ClientLobbyPlugin);
    client_app.add_plugins(GameClientPlugin);

    client_app.init_state::<ClientGameState>();
    client_app.insert_state(ClientGameState::LocalMenu);

    client_app
}
