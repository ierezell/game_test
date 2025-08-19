use avian3d::prelude::*;
use bevy::{log::LogPlugin, prelude::*, window::ExitCondition};
use clap::{Parser, Subcommand};
use lightyear::prelude::client::ClientPlugins;
use lightyear::prelude::server::ServerPlugins;
use std::time::Duration;

mod audio;
mod combat;
mod debug_ui;
mod enemies;
mod events;
mod menu;
mod observers;
mod player;
mod scenes;
mod static_level;
mod ui;

// Networking modules (updated for Lightyear 0.23.0)
mod client;
mod protocol;
mod server;
mod shared;

#[derive(Parser)]
#[command(name = "gtfo-like-game")]
#[command(about = "A GTFO-like cooperative survival horror game")]
struct Cli {
    #[command(subcommand)]
    mode: Option<GameMode>,

    #[arg(short, long, default_value = "127.0.0.1")]
    server_addr: String,

    #[arg(short, long, default_value = "5000")]
    port: u16,
}

#[derive(Subcommand, Debug)]
enum GameMode {
    /// Run as a client
    Client {
        #[arg(short, long, default_value = "1")]
        client_id: u64,
    },
    /// Run as a server
    Server,
    /// Run as a host client (server + client in same process)
    HostClient {
        #[arg(short, long, default_value = "0")]
        client_id: u64,
    },
}

fn main() {
    println!("Starting main function");

    // Test if the issue is with argument parsing
    let args: Vec<String> = std::env::args().collect();
    println!("Arguments: {:?}", args);

    let cli = Cli::parse();
    println!("CLI parsed: {:?}", cli.mode);

    let mut app = App::new();
    println!("App created");

    // Add base plugins (conditionally for server vs client)
    match &cli.mode {
        Some(GameMode::Server) => {
            // Headless server - DefaultPlugins with no window
            app.add_plugins(
                DefaultPlugins
                    .set(WindowPlugin {
                        primary_window: None,                    // No window for server
                        exit_condition: ExitCondition::DontExit, // Prevent exit when no windows
                        ..default()
                    })
                    .set(LogPlugin {
                        filter: "error,gtfo_like_game=info,lightyear=info".into(),
                        level: bevy::log::Level::INFO,
                        ..default()
                    }),
            );

            // Add Lightyear server plugins
            app.add_plugins(ServerPlugins {
                tick_duration: Duration::from_secs_f64(1.0 / protocol::FIXED_TIMESTEP_HZ),
            });

            // Add protocol after Lightyear plugins
            app.add_plugins(protocol::ProtocolPlugin);
        }
        _ => {
            // Client/HostClient/SinglePlayer - full plugins with window
            app.add_plugins(
                DefaultPlugins
                    .set(WindowPlugin {
                        primary_window: Some(Window {
                            title: match &cli.mode {
                                Some(GameMode::Client { .. }) => {
                                    "GTFO-Like Game (Client)".to_string()
                                }
                                Some(GameMode::HostClient { .. }) => {
                                    "GTFO-Like Game (Host)".to_string()
                                }
                                None => "GTFO-Like Game (Single Player)".to_string(),
                                _ => "GTFO-Like Game".to_string(),
                            },
                            resolution: (1280.0, 720.0).into(),
                            ..default()
                        }),
                        ..default()
                    })
                    .set(LogPlugin {
                        filter: "error,gtfo_like_game=info,lightyear=info".into(),
                        level: bevy::log::Level::INFO,
                        ..default()
                    }),
            );

            // Add Lightyear client plugins for client modes
            if matches!(
                &cli.mode,
                Some(GameMode::Client { .. }) | Some(GameMode::HostClient { .. })
            ) {
                app.add_plugins(ClientPlugins {
                    tick_duration: Duration::from_secs_f64(1.0 / protocol::FIXED_TIMESTEP_HZ),
                });

                // Add protocol after Lightyear plugins
                app.add_plugins(protocol::ProtocolPlugin);
            }
        }
    }

    // Add physics (conditionally for server)
    match &cli.mode {
        Some(GameMode::Server) => {
            // Server only needs core physics, no debug rendering
            app.add_plugins(PhysicsPlugins::default());
        }
        _ => {
            // Client/HostClient/SinglePlayer - full physics with debug
            app.add_plugins(PhysicsPlugins::default());
            app.add_plugins(PhysicsDebugPlugin::default());

            // Add game state (only needed for UI)
            app.init_state::<menu::GameState>();
        }
    }

    // Add base game plugins (conditionally based on mode)
    let base_plugins = (
        combat::CombatPlugin,
        enemies::EnemyPlugin,
        events::GameEventsPlugin,
        observers::GameObserversPlugin,
    );

    // Add networking based on mode
    match cli.mode {
        Some(GameMode::Client { client_id }) => {
            app.add_plugins(base_plugins);
            app.add_plugins((
                ui::UIPlugin,
                audio::GameAudioPlugin,
                debug_ui::DebugUIPlugin,
                menu::MenuPlugin,
                static_level::StaticLevelPlugin,
                scenes::GameScenesPlugin,
            ));
            app.add_plugins(client::ClientPlugin);
            info!("Starting as client with ID: {}", client_id);
        }
        Some(GameMode::Server) => {
            app.add_plugins(base_plugins);
            // Note: UI, audio, scenes, and static level plugins disabled for server
            app.add_plugins(server::ServerPlugin);
            info!("Starting as server (headless)");
        }
        Some(GameMode::HostClient { client_id }) => {
            app.add_plugins(base_plugins);
            app.add_plugins((
                ui::UIPlugin,
                audio::GameAudioPlugin,
                debug_ui::DebugUIPlugin,
                menu::MenuPlugin,
                static_level::StaticLevelPlugin,
                scenes::GameScenesPlugin,
            ));
            app.add_plugins((client::ClientPlugin, server::ServerPlugin));
            info!("Starting as host client with ID: {}", client_id);
        }
        None => {
            app.add_plugins(base_plugins);
            app.add_plugins((
                ui::UIPlugin,
                audio::GameAudioPlugin,
                debug_ui::DebugUIPlugin,
                menu::MenuPlugin,
                static_level::StaticLevelPlugin,
                scenes::GameScenesPlugin,
            ));
            // Single player mode - add the regular player plugin
            app.add_plugins(player::PlayerPlugin);
            info!("Starting in single player mode");
        }
    }

    // Add startup systems (only for non-server modes)
    match &cli.mode {
        Some(GameMode::Server) => {
            // Server doesn't need camera or lighting
        }
        _ => {
            app.add_systems(Startup, (setup_camera, setup_basic_light));
        }
    }

    println!("About to run app");
    app.run();
    println!("App finished running");
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 1.7, 5.0).looking_at(Vec3::ONE, Vec3::Y),
        Name::new("First Person Camera"),
        player::FirstPersonCamera,
    ));
}

fn setup_basic_light(mut commands: Commands) {
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.8,
        affects_lightmapped_meshes: true,
    });
}
