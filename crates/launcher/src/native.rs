#![cfg(not(target_family = "wasm"))]

use bevy::prelude::App;
use clap::{Parser, ValueEnum};
use client::app::{add_audio_to_client_app, add_basics_to_client_app, add_network_to_client_app};

use server::app::{add_basics_to_server_app, add_network_to_server_app};

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
}

pub fn run() {
    let cli = Cli::parse();
    let asset_path = "../../assets".to_string();

    match cli.mode {
        Mode::Client => {
            if cli.client_id == 0 {
                panic!(
                    "No --client_id specified. To connect, specify a client id with --client_id <id>"
                );
            }

            let mut client_app = App::new();
            add_basics_to_client_app(
                &mut client_app,
                asset_path.clone(),
                cli.autoconnect,
                cli.client_id,
            );
            add_network_to_client_app(&mut client_app, cli.client_id);
            add_audio_to_client_app(&mut client_app);
            client_app.run();
        }
        Mode::Server => {
            let mut server_app = App::new();
            add_basics_to_server_app(&mut server_app, cli.headless);
            add_network_to_server_app(&mut server_app);
            server_app.run();
        }
    }
}
