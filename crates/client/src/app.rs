use crate::audio::GameAudioPlugin;
use crate::game_state::GameLifecyclePlugin;
use crate::input::ClientInputPlugin;
use crate::menu::MenuPlugin;
use crate::network::NetworkPlugin;
use crate::render::RenderPlugin;

use bevy::prelude::*;
use bevy::prelude::{AssetPlugin, default};
use bevy::window::{Window, WindowPlugin};
use lightyear::prelude::client::ClientPlugins;

use lightyear::prelude::*;
use shared::SharedPlugin;
use shared::protocol::PlayerId;
use std::time::Duration;

#[derive(Resource, Clone)]
pub struct ClientAssetPath(pub String);

pub fn add_basics_to_client_app(app: &mut App, asset_path: String, autoconnect: bool) -> &mut App {
    app.add_plugins((
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Yolo Game - Client".to_string(),
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
    ));

    app.insert_resource(crate::network::AutoConnect(autoconnect));

    app
}

pub fn add_network_to_client_app(app: &mut App, client_id: u64) -> &mut App {
    // Lightyear's ClientPlugins - following examples pattern
    app.add_plugins(ClientPlugins {
        tick_duration: Duration::from_secs_f64(1.0 / shared::FIXED_TIMESTEP_HZ),
    });
    app.insert_resource(PlayerId(PeerId::Local(client_id)));

    app.add_plugins(NetworkPlugin);
    app
}

pub fn add_audio_to_client_app(app: &mut App) -> &mut App {
    app.add_plugins(GameAudioPlugin);
    app
}
