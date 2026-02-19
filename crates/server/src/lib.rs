pub mod entities;
pub mod input;
pub mod lobby;
pub mod network;
pub mod render;

use bevy::MinimalPlugins;
use bevy::prelude::{App, DefaultPlugins, PluginGroup, States, default};
use bevy::state::app::AppExtStates;
use bevy::window::{Window, WindowPlugin};
use lightyear::prelude::server::ServerPlugins;
use std::time::Duration;

use crate::entities::ServerEntitiesPlugin;
use crate::lobby::ServerLobbyPlugin;
use crate::network::ServerNetworkPlugin;
use crate::render::RenderPlugin;
use shared::{NetworkMode, SharedPlugin};
#[derive(States, Debug, Default, Clone, PartialEq, Eq, Hash)]
pub enum ServerGameState {
    #[default]
    Lobby,
    Loading,
    Playing,
}

pub fn create_server_app(headless: bool, network_mode: NetworkMode) -> App {
    let mut app = App::new();
    if headless {
        app.add_plugins((
            MinimalPlugins,
            // LogPlugin disabled for tests to avoid "Could not set global logger" error
            // bevy::log::LogPlugin::default(),
            bevy::state::app::StatesPlugin,
            bevy::diagnostic::DiagnosticsPlugin,
            bevy::asset::AssetPlugin::default(),
            bevy::scene::ScenePlugin,
            bevy::mesh::MeshPlugin,
            bevy::animation::AnimationPlugin,
        ));
    } else {
        app.add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Yolo Game - Server".to_string(),
                        resolution: (400, 200).into(),
                        ..default()
                    }),
                    ..default()
                })
                .set(bevy::asset::AssetPlugin {
                    file_path: "../../../assets".to_string(),
                    ..Default::default()
                }),
        )
        .add_plugins(RenderPlugin);
    }

    app.insert_resource(network_mode);
    app.add_plugins(SharedPlugin);
    app.add_plugins(ServerPlugins {
        tick_duration: Duration::from_secs_f64(1.0 / shared::FIXED_TIMESTEP_HZ),
    });
    app.add_plugins(ServerNetworkPlugin);
    app.add_plugins(ServerLobbyPlugin);
    app.add_plugins(ServerEntitiesPlugin);
    app.init_state::<ServerGameState>();
    app.insert_state(ServerGameState::Lobby);

    app
}

#[cfg(test)]
mod tests {
    use super::{ServerGameState, create_server_app};
    use shared::NetworkMode;

    #[test]
    fn create_headless_server_initializes_lobby_state() {
        let app = create_server_app(true, NetworkMode::Local);
        let state = app.world().resource::<bevy::prelude::State<ServerGameState>>();
        assert_eq!(state.get(), &ServerGameState::Lobby);
    }

    #[test]
    fn create_server_in_udp_mode_sets_network_resource() {
        let app = create_server_app(true, NetworkMode::Udp);
        let network_mode = app.world().resource::<NetworkMode>();
        assert_eq!(*network_mode, NetworkMode::Udp);
    }
}
