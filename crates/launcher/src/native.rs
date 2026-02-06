use crate::AutoJoin;
use crate::host::create_host_app;
use crate::local_menu::LocalMenuPlugin;
use clap::{Parser, ValueEnum};
use client::create_client_app;
use client::lobby::AutoStart;
use server::create_server_app;
use shared::{GymMode, NetworkMode};

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
    cargo run --bin launcher -- server --stop-after 30          # Start server, stop after 30 seconds
    cargo run --bin launcher -- client --auto-host --stop-after 60 # Auto-host, stop after 1 minute
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
    #[arg(help = "Automatically start the game when hosting (requires --auto-host)")]
    auto_start: bool,

    #[arg(long)]
    #[arg(help = "Automatically stop the game after X seconds (0 = disabled)")]
    stop_after: Option<u64>,

    #[arg(long, default_value_t = false)]
    #[arg(help = "Use gym mode (test environment with simple square room and one NPC)")]
    gym: bool,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Mode {
    Client,
    Server,
    Host,
}

pub fn run() {
    let cli = Cli::parse();

    match cli.mode {
        Mode::Client => {
            let mut client_app = create_client_app(
                cli.client_id,
                "../../assets".to_string(),
                cli.headless,
                NetworkMode::Udp,
            );
            client_app.add_plugins(LocalMenuPlugin);
            client_app.insert_resource(GymMode(cli.gym));

            if cli.auto_start {
                client_app.insert_resource(AutoStart(true));
            }

            if cli.auto_join {
                client_app.insert_resource(AutoJoin(true));
            }

            if let Some(stop_after_seconds) = cli.stop_after
                && stop_after_seconds > 0
            {
                std::thread::spawn(move || {
                    std::thread::sleep(std::time::Duration::from_secs(stop_after_seconds));
                    println!("Auto-stopping after {} seconds", stop_after_seconds);
                    std::process::exit(0);
                });
            }

            client_app.run();
        }
        Mode::Server => {
            let mut server_app = create_server_app(cli.headless, NetworkMode::Udp);
            if cli.gym {
                server_app.insert_resource(GymMode(cli.gym));
            }

            if let Some(stop_after_seconds) = cli.stop_after
                && stop_after_seconds > 0
            {
                std::thread::spawn(move || {
                    std::thread::sleep(std::time::Duration::from_secs(stop_after_seconds));
                    println!("Auto-stopping server after {} seconds", stop_after_seconds);
                    std::process::exit(0);
                });
            }

            server_app.run();
        }
        Mode::Host => {
            let mut host_app = create_host_app(cli.headless, "../../assets".to_string());

            if cli.gym {
                host_app.insert_resource(GymMode(cli.gym));
            }

            if let Some(stop_after_seconds) = cli.stop_after
                && stop_after_seconds > 0
            {
                std::thread::spawn(move || {
                    std::thread::sleep(std::time::Duration::from_secs(stop_after_seconds));
                    println!("Auto-stopping after {} seconds", stop_after_seconds);
                    std::process::exit(0);
                });
            }

            host_app.run();
        }
    }
}
