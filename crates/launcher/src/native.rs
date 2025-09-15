#![cfg(not(target_family = "wasm"))]

use bevy::prelude::App;
use clap::{Parser, ValueEnum};
use client::app::{add_audio_to_client_app, add_basics_to_client_app, add_network_to_client_app};

use common::NetTransport;
use server::app::{add_basics_to_server_app, add_network_to_server_app};
use std::time::Duration;
const FIXED_TIMESTEP_HZ: f64 = 64.0;

#[derive(Parser)]
#[command(name = "yolo-game")]
#[command(version = "0.1")]
#[command(about = "Multiplayer survival horror game launcher")]
struct Cli {
    #[arg(value_enum)]
    mode: Mode,

    #[arg(short, long, default_value_t = 0)]
    client_id: u64,

    #[arg(long, default_value_t = false)]
    headless: bool,

    #[arg(long, default_value_t = false)]
    autoconnect: bool,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Mode {
    Client,
    Server,
    Host,
}

pub fn run() {
    let cli = Cli::parse();
    let asset_path = "../../assets".to_string();
    let transport = NetTransport::Udp;

    match cli.mode {
        Mode::Client => {
            if cli.client_id == 0 {
                panic!(
                    "No --client_id specified. To connect, specify a client id with --client_id <id>"
                );
            }

            let transport = NetTransport::Udp;
            let mut client_app = App::new();
            add_basics_to_client_app(&mut client_app, asset_path.clone(), cli.autoconnect);
            add_network_to_client_app(&mut client_app, cli.client_id, transport);

            add_audio_to_client_app(&mut client_app);

            client_app.run();
        }
        Mode::Server => {
            let mut server_app = App::new();
            add_basics_to_server_app(&mut server_app, asset_path, cli.headless);
            add_network_to_server_app(&mut server_app);
            server_app.run();
        }
        Mode::Host => {
            if cli.client_id == 0 {
                panic!(
                    "No --client_id specified for host mode. Specify a unique client id with --client_id <id>"
                );
            }
            // TODO: Implement host mode (client + server in same app)
        }
    }
}
