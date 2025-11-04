use crate::menu::{AutoHost, AutoStart};
use bevy::color::palettes::tailwind::{GREEN_500, SLATE_700, SLATE_800};
use bevy::prelude::{
    AlignItems, App, BackgroundColor, Commands, CommandsStatesExt, DetectChanges, Entity,
    FlexDirection, IntoScheduleConfigs, JustifyContent, Name, Node, OnEnter, OnExit, Plugin, Query,
    Res, Resource, Text, TextFont, UiRect, Update, Val, With, debug, info,
};
use client::LocalPlayerId;
use lightyear::prelude::PeerId;
use shared::game_state::GameState;
use shared::protocol::LobbyState;

use std::thread;

pub struct LobbyPlugin;

impl Plugin for LobbyPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(LobbyState {
            players: Vec::new(),
            host_id: PeerId::Netcode(0),
            game_started: false,
        });

        app.add_systems(OnEnter(GameState::HostingLobby), on_host_game);
        app.add_systems(OnEnter(GameState::JoiningGame), on_join_game);
        app.add_systems(OnEnter(GameState::MainMenu), check_auto_host);
        app.add_systems(
            OnEnter(GameState::InLobby),
            (spawn_lobby_ui, setup_lobby_camera, check_auto_start),
        );
        app.add_systems(OnExit(GameState::InLobby), despawn_lobby_ui);
        app.add_systems(
            Update,
            update_lobby_ui.run_if(bevy::prelude::in_state(GameState::InLobby)),
        );
    }
}

#[derive(Resource)]
pub struct IsHost(pub bool);

// UI Components
#[derive(bevy::prelude::Component)]
pub struct LobbyUI;

#[derive(bevy::prelude::Component)]
pub struct LobbyCamera;

#[derive(bevy::prelude::Component)]
pub struct PlayerListContainer;

#[derive(bevy::prelude::Component)]
pub struct PlayButton;

#[derive(bevy::prelude::Component)]
pub struct LobbyStatusText;

/// Check if we should auto-start the game when entering lobby as host
fn check_auto_start(
    auto_start: Option<Res<AutoStart>>,
    is_host: Option<Res<IsHost>>,
    mut commands: Commands,
) {
    if let (Some(auto_start_res), Some(is_host_res)) = (auto_start, is_host) {
        if auto_start_res.0 && is_host_res.0 {
            info!("Auto-start enabled and we are host, starting game automatically");
            // Add a small delay to ensure lobby UI is fully set up
            std::thread::sleep(std::time::Duration::from_millis(100));
            commands.set_state(GameState::Loading);
        }
    }
}

/// Check if we should auto-host when entering main menu
fn check_auto_host(auto_host: Option<Res<AutoHost>>, mut commands: Commands) {
    if let Some(auto_host_res) = auto_host {
        if auto_host_res.0 {
            info!("Auto-hosting enabled, transitioning to HostingLobby");
            commands.set_state(GameState::HostingLobby);
        }
    }
}

/// System that handles hosting a game
/// Spawns a server in a separate thread and then connects the client to it
fn on_host_game(mut commands: Commands, local_player_id: Res<LocalPlayerId>) {
    info!("Starting to host a game...");

    thread::spawn(move || {
        debug!("Starting server thread...");
        let mut server_app = server::create_server_app(true);
        info!("Server started and running...");
        server_app.run();
    });

    commands.insert_resource(IsHost(true));

    // Initialize lobby state with host as first player
    commands.insert_resource(LobbyState {
        players: vec![PeerId::Netcode(local_player_id.0)],
        host_id: PeerId::Netcode(local_player_id.0),
        game_started: false,
    });

    info!("Waiting for server to start...");
    std::thread::sleep(std::time::Duration::from_millis(2000));
    commands.set_state(GameState::Connecting);

    info!("Hosting setup complete, connecting client...");
}

/// System that handles joining a game
/// For now, just connects directly. Later can add server browser/IP input
fn on_join_game(mut commands: Commands) {
    info!("Joining a game...");

    commands.insert_resource(IsHost(false));
    commands.set_state(GameState::Connecting);

    info!("Attempting to join game...");
}

fn setup_lobby_camera(mut commands: Commands) {
    commands.spawn((
        bevy::prelude::Camera2d,
        LobbyCamera,
        Name::new("LobbyCamera"),
    ));
    info!("Spawned lobby camera");
}

fn spawn_lobby_ui(
    mut commands: Commands,
    lobby_state: Res<LobbyState>,
    is_host: Option<Res<IsHost>>,
    local_player_id: Res<LocalPlayerId>,
) {
    info!("Spawning lobby UI");

    let is_host_player = is_host.map(|h| h.0).unwrap_or(false);

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                ..Default::default()
            },
            BackgroundColor(SLATE_800.into()),
            LobbyUI,
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("Game Lobby"),
                TextFont {
                    font_size: 40.0,
                    ..Default::default()
                },
                Node {
                    padding: UiRect::bottom(Val::Px(30.0)),
                    ..Default::default()
                },
            ));

            // Status text
            parent.spawn((
                Text::new(if is_host_player {
                    "You are the host - Press SPACE to start the game"
                } else {
                    "Waiting for host to start the game..."
                }),
                TextFont {
                    font_size: 20.0,
                    ..Default::default()
                },
                Node {
                    padding: UiRect::bottom(Val::Px(20.0)),
                    ..Default::default()
                },
                LobbyStatusText,
            ));

            // Player list container
            parent
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::all(Val::Px(20.0)),
                        ..Default::default()
                    },
                    BackgroundColor(SLATE_700.into()),
                    PlayerListContainer,
                ))
                .with_children(|list_parent| {
                    list_parent.spawn((
                        Text::new("Players:"),
                        TextFont {
                            font_size: 24.0,
                            ..Default::default()
                        },
                        Node {
                            padding: UiRect::bottom(Val::Px(10.0)),
                            ..Default::default()
                        },
                    ));

                    // Add current players
                    for (i, player_id) in lobby_state.players.iter().enumerate() {
                        let is_host_marker = if *player_id == lobby_state.host_id {
                            " (Host)"
                        } else {
                            ""
                        };
                        let is_you = if *player_id == PeerId::Netcode(local_player_id.0) {
                            " (You)"
                        } else {
                            ""
                        };

                        list_parent.spawn((
                            Text::new(format!("Player {}{}{}", i + 1, is_host_marker, is_you)),
                            TextFont {
                                font_size: 18.0,
                                ..Default::default()
                            },
                            Node {
                                padding: UiRect::bottom(Val::Px(5.0)),
                                ..Default::default()
                            },
                        ));
                    }
                });

            // Play button placeholder (simplified for now)
            if is_host_player {
                parent
                    .spawn((
                        Node {
                            padding: UiRect::all(Val::Px(15.0)),
                            margin: UiRect::top(Val::Px(30.0)),
                            ..Default::default()
                        },
                        BackgroundColor(GREEN_500.into()),
                        PlayButton,
                    ))
                    .with_children(|button_parent| {
                        button_parent.spawn((
                            Text::new("PLAY (Press SPACE)"),
                            TextFont {
                                font_size: 24.0,
                                ..Default::default()
                            },
                        ));
                    });
            }
        });
}

fn despawn_lobby_ui(
    mut commands: Commands,
    lobby_ui_query: Query<Entity, With<LobbyUI>>,
    lobby_camera_query: Query<Entity, With<LobbyCamera>>,
) {
    for entity in lobby_ui_query.iter() {
        commands.entity(entity).despawn();
    }

    for entity in lobby_camera_query.iter() {
        commands.entity(entity).despawn();
    }

    info!("Despawned lobby UI");
}

fn update_lobby_ui(
    lobby_state: Res<LobbyState>,
    player_list_query: Query<Entity, With<PlayerListContainer>>,
    mut commands: Commands,
    local_player_id: Res<LocalPlayerId>,
    input: Res<bevy::input::ButtonInput<bevy::input::keyboard::KeyCode>>,
    is_host: Option<Res<IsHost>>,
) {
    // Handle host input to start game
    if let Some(is_host_res) = is_host {
        if is_host_res.0 && input.just_pressed(bevy::input::keyboard::KeyCode::Space) {
            info!("Host pressed SPACE - starting game!");
            commands.set_state(GameState::Loading);
            return;
        }
    }

    // Update UI if lobby state changed
    if !lobby_state.is_changed() {
        return;
    }

    info!(
        "Updating lobby UI with {} players",
        lobby_state.players.len()
    );

    // Rebuild player list when lobby state changes
    for container_entity in player_list_query.iter() {
        // Clear player list and rebuild
        commands.entity(container_entity).clear_children();

        commands
            .entity(container_entity)
            .with_children(|list_parent| {
                list_parent.spawn((
                    Text::new("Players:"),
                    TextFont {
                        font_size: 24.0,
                        ..Default::default()
                    },
                    Node {
                        padding: UiRect::bottom(Val::Px(10.0)),
                        ..Default::default()
                    },
                ));

                // Add current players
                for (i, player_id) in lobby_state.players.iter().enumerate() {
                    let is_host_marker = if *player_id == lobby_state.host_id {
                        " (Host)"
                    } else {
                        ""
                    };
                    let is_you = if *player_id == PeerId::Netcode(local_player_id.0) {
                        " (You)"
                    } else {
                        ""
                    };

                    list_parent.spawn((
                        Text::new(format!("Player {}{}{}", i + 1, is_host_marker, is_you)),
                        TextFont {
                            font_size: 18.0,
                            ..Default::default()
                        },
                        Node {
                            padding: UiRect::bottom(Val::Px(5.0)),
                            ..Default::default()
                        },
                    ));
                }
            });
    }
}
