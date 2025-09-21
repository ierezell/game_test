use crate::audio::GameAudioPlugin;
use crate::camera::CameraPlugin;
use crate::game_state::GameLifecyclePlugin;
use crate::input::ClientInputPlugin;
use crate::menu::MenuPlugin;
use crate::network::NetworkPlugin;
use crate::render::RenderPlugin;

use bevy::prelude::*;
use bevy::prelude::{AssetPlugin, default};
use bevy::window::{Window, WindowPlugin};
use lightyear::prelude::client::ClientPlugins;

use shared::SharedPlugin;
use std::time::Duration;

#[derive(Resource, Clone)]
pub struct ClientAssetPath(pub String);

#[derive(Resource)]
pub struct LocalPlayerId(pub u64);

pub fn add_basics_to_client_app(
    app: &mut App,
    asset_path: String,
    autoconnect: bool,
    client_id: u64,
) -> &mut App {
    let offset_x = (100.0 * (client_id as f32)) as i32;
    let offset_y = (100.0 * (client_id as f32)) as i32;

    app.add_plugins((
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: format!("Yolo Game - Client {}", client_id),
                    resolution: (1280., 720.).into(), // Better resolution for first-person camera
                    position: WindowPosition::At((offset_x, offset_y).into()),
                    ..default()
                }),
                ..default()
            })
            .set(AssetPlugin {
                file_path: asset_path,
                ..Default::default()
            }),
        RenderPlugin,
        MenuPlugin,
        SharedPlugin,
        GameLifecyclePlugin,
        ClientInputPlugin,
        CameraPlugin,
    ));

    app.insert_resource(crate::network::AutoConnect(autoconnect));

    app
}

pub fn add_network_to_client_app(app: &mut App, client_id: u64) -> &mut App {
    // Lightyear's ClientPlugins
    app.add_plugins(ClientPlugins {
        tick_duration: Duration::from_secs_f64(1.0 / shared::FIXED_TIMESTEP_HZ),
    });
    
    app.insert_resource(LocalPlayerId(client_id));
    debug!("🔧 Client configured with Netcode PeerId: {}", client_id);

    app.add_plugins(NetworkPlugin);
    app
}

pub fn add_audio_to_client_app(app: &mut App) -> &mut App {
    app.add_plugins(GameAudioPlugin);
    app
}

