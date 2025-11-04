#![cfg(not(target_family = "wasm"))]

use bevy::prelude::App;
use client::LocalPlayerId;

use clap::{Parser, ValueEnum};

use crate::menu::{LobbyPlugin, LocalMenuPlugin};
use bevy::prelude::{AssetPlugin, default};
use bevy::prelude::{DefaultPlugins, PluginGroup, debug};
use bevy::window::{Window, WindowPlugin};
use client::game_state::GameLifecyclePlugin;
use client::network::NetworkPlugin;
use lightyear::prelude::client::ClientPlugins;

use std::time::Duration;

#[derive(Parser)]
#[command(name = "yolo-game")]
#[command(version = "0.1")]
#[command(about = "Multiplayer survival horror game launcher")]
#[command(long_about = "
Multiplayer survival horror game launcher

EXAMPLES:
    cargo run --bin launcher -- client                           # Start client in menu
    cargo run --bin launcher -- client --auto-host --client-id 1 # Auto-host and wait in lobby
    cargo run --bin launcher -- client --auto-host --auto-start  # Auto-host and auto-start game
    cargo run --bin launcher -- server                           # Start dedicated server
")]
struct Cli {
    #[arg(value_enum)]
    mode: Mode,

    #[arg(short, long, default_value_t = 0)]
    client_id: u64,

    #[arg(long, default_value_t = false)]
    headless: bool,

    #[arg(long, default_value_t = false)]
    auto_connect: bool,

    #[arg(long, default_value_t = false)]
    #[arg(help = "Automatically host a game on startup")]
    auto_host: bool,

    #[arg(long, default_value_t = false)]
    #[arg(help = "Automatically start the game when hosting (requires --auto-host)")]
    auto_start: bool,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Mode {
    Client,
    Server,
}

pub fn run() {
    let cli = Cli::parse();
    let asset_path = "../../assets".to_string();

    match cli.mode {
        Mode::Client => {
            let client_id = if cli.client_id == 0 { 1 } else { cli.client_id };

            let window_title = if cli.auto_host {
                format!("Game Test - Host {}", client_id)
            } else {
                format!("Game Test - Client {}", client_id)
            };

            let mut client_app = App::new();
            client_app.add_plugins(
                DefaultPlugins
                    .set(WindowPlugin {
                        primary_window: Some(Window {
                            title: window_title,
                            resolution: (1280, 720).into(),
                            ..default()
                        }),
                        ..default()
                    })
                    .set(AssetPlugin {
                        file_path: asset_path,
                        ..Default::default()
                    }),
            );

            client_app.add_plugins(ClientPlugins {
                tick_duration: Duration::from_secs_f64(1.0 / shared::FIXED_TIMESTEP_HZ),
            });

            client_app.insert_resource(LocalPlayerId(client_id));
            debug!("🔧 Client configured with Netcode PeerId: {}", client_id);

            client_app.add_plugins(NetworkPlugin);
            client_app.add_plugins(GameLifecyclePlugin);
            client_app.add_plugins(LocalMenuPlugin);
            client_app.add_plugins(LobbyPlugin);
            client_app.add_plugins(shared::SharedPlugin);
            client_app.add_plugins(client::render::RenderPlugin);
            client_app.add_plugins(client::input::ClientInputPlugin);
            client_app.add_plugins(client::entity::ClientRenderPlugin);

            if cli.auto_connect {
                // For auto-host scenarios, auto-connect will be handled by the lobby system
                // after the server is ready. For non-host scenarios, set it immediately.
                if !cli.auto_host {
                    client_app.insert_resource(client::network::AutoConnect(true));
                }
            }

            if cli.auto_host {
                client_app.insert_resource(crate::menu::AutoHost(true));
            }

            if cli.auto_start {
                client_app.insert_resource(crate::menu::AutoStart(true));
            }

            client_app.run();
        }
        Mode::Server => {
            let mut server_app = server::create_server_app(cli.headless);
            server_app.run();
        }
    }
}
