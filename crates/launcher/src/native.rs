use client::create_client_app;

use clap::{Parser, ValueEnum};

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
    cargo run --bin launcher -- client --auto-join --client-id 2 # Auto-join a game
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
    #[arg(help = "Automatically join a game on startup")]
    auto_join: bool,

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

    match cli.mode {
        Mode::Client => {
            let mut client_app = create_client_app(
                cli.client_id,
                "../../assets".to_string(),
                cli.auto_host,
                cli.auto_join,
                cli.auto_start,
            );
            client_app.run();
        }
        Mode::Server => {
            let mut server_app = server::create_server_app(cli.headless);
            server_app.run();
        }
    }
}
