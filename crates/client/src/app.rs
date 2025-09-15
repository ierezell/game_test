use crate::audio::GameAudioPlugin;
use crate::game_state::GameLifecyclePlugin;
use crate::menu::MenuPlugin;
use crate::network::NetworkPlugin;
use crate::render::RenderPlugin;
use bevy::asset::AssetMetaCheck;
use bevy::prelude::*;
use bevy::prelude::{AssetMode, AssetPlugin, IntoScheduleConfigs, default};
use bevy::window::{Window, WindowPlugin};
use common::protocol::{PlayerId, ProtocolPlugin};
use common::{CommonPlugin, NetTransport};
use lightyear::prelude::client::ClientPlugins;
use lightyear::prelude::client::*;
use lightyear::prelude::input::InputBuffer;
use lightyear::prelude::*;

#[derive(Resource, Clone)]
pub struct ClientAssetPath(pub String);

pub fn add_basics_to_client_app(app: &mut App, asset_path: String, autoconnect: bool) -> &mut App {
    app.add_plugins((
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Client".to_string(),
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
        CommonPlugin,
        GameLifecyclePlugin,
    ));

    // Insert AutoConnect resource based on CLI flag
    app.insert_resource(crate::network::AutoConnect(autoconnect));

    app
}

pub fn add_network_to_client_app(
    app: &mut App,
    client_id: u64,
    transport: NetTransport,
) -> &mut App {
    // Lightyear's ClientPlugins
    app.add_plugins(ClientPlugins {
        tick_duration: std::time::Duration::from_secs_f64(1.0 / 60.0),
    });
    app.insert_resource(PlayerId(client_id));
    app.insert_resource(transport);
    app.add_plugins(NetworkPlugin);
    app
}

pub fn add_audio_to_client_app(app: &mut App) -> &mut App {
    app.add_plugins(GameAudioPlugin);
    app
}
