use crate::ClientGameState;
use crate::LocalPlayerId;
use crate::menu::AutoStart;

use bevy::color::palettes::tailwind::{GREEN_500, SLATE_700, SLATE_800};

use bevy::ecs::query::Changed;
use bevy::prelude::{
    AlignItems, App, BackgroundColor, Camera2d, Click, Commands, Component, Entity, FlexDirection,
    IntoScheduleConfigs, JustifyContent, Name, Node, On, OnEnter, OnExit, Plugin, Pointer, Query,
    Res, Single, Text, TextFont, UiRect, Update, Val, With, in_state, info,
};

use lightyear::prelude::MessageSender;
use lightyear::prelude::MetadataChannel;
use shared::protocol::{HostStartGameEvent, LobbyState};

pub struct ClientLobbyPlugin;
impl Plugin for ClientLobbyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(ClientGameState::Lobby),
            (spawn_lobby_ui, spawn_lobby_camera),
        );
        app.add_systems(
            OnExit(ClientGameState::Lobby),
            (despawn_lobby_ui, despawn_lobby_camera),
        );
        // Add system to handle auto-start periodically in lobby
        app.add_systems(OnEnter(ClientGameState::Lobby), handle_auto_start);
        // Add system to update lobby text when LobbyState changes
        app.add_systems(
            Update,
            update_lobby_text.run_if(in_state(ClientGameState::Lobby)),
        );
    }
}

fn handle_auto_start(
    auto_start: Option<Res<AutoStart>>,
    lobby_state: Query<&LobbyState>,
    local_player_id: Res<LocalPlayerId>,
    mut sender: Single<&mut MessageSender<HostStartGameEvent>>,
) {
    if let Some(auto_start_res) = auto_start {
        info!("AutoStart True, Checking for auto-start conditions...");
        if auto_start_res.0 {
            if let Ok(lobby_data) = lobby_state.single() {
                if lobby_data.host_id == local_player_id.0 {
                    bevy::log::info!("🚀 Auto-starting game as host");
                    sender.send::<MetadataChannel>(HostStartGameEvent);
                }
            }
        }
    }
}

#[derive(Component)]
pub struct LobbyCamera;

fn spawn_lobby_camera(mut commands: Commands) {
    commands.spawn((Camera2d, LobbyCamera, Name::new("LobbyCamera")));
}

fn despawn_lobby_camera(mut commands: Commands, q_lobby_camera: Query<Entity, With<LobbyCamera>>) {
    for entity in &q_lobby_camera {
        commands.entity(entity).despawn();
    }
}

#[derive(Component)]
pub struct LobbyUI;

#[derive(Component)]
pub struct PlayButton;

#[derive(Component)]
pub struct LobbyStatusText;

#[derive(Component)]
pub struct PlayerListContainer;

#[derive(Component)]
pub struct PlayerText;

fn spawn_lobby_ui(mut commands: Commands) {
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
            // Connecting...
            parent.spawn((
                Text::new("Connecting to server..."),
                TextFont {
                    font_size: 24.0,
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
                        PlayerListContainer,
                    ));
                });
        });
}

fn despawn_lobby_ui(mut commands: Commands, lobby_ui_query: Query<Entity, With<LobbyUI>>) {
    for entity in lobby_ui_query.iter() {
        commands.entity(entity).despawn();
    }
}

fn update_lobby_text(
    lobby_state: Query<&LobbyState, Changed<LobbyState>>,
    local_player_id: Res<LocalPlayerId>,
    mut status_text_query: Query<&mut Text, With<LobbyStatusText>>,
    player_list_container: Query<Entity, With<PlayerListContainer>>,
    player_text_entities: Query<Entity, With<PlayerText>>,
    lobby_ui: Query<Entity, With<LobbyUI>>,
    mut commands: Commands,
) {
    // Only proceed if we have a lobby state
    if let Ok(lobby_data) = lobby_state.single() {
        info!(
            "🎯 CLIENT: Received lobby state with {} players, host_id: {}, local_id: {}",
            lobby_data.players.len(),
            lobby_data.host_id,
            local_player_id.0
        );
        let is_host_player = lobby_data.host_id == local_player_id.0;

        // Update status text
        for mut status_text in status_text_query.iter_mut() {
            **status_text = if is_host_player {
                "You are the host - You can start the game.".to_string()
            } else {
                "Waiting for host to start the game...".to_string()
            };
        }

        // Add play button for the host player
        if is_host_player {
            if let Ok(lobby_entity) = lobby_ui.single() {
                commands.entity(lobby_entity).with_children(|parent| {
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
                            button_parent
                                .spawn((
                                    Text::new("PLAY"),
                                    TextFont {
                                        font_size: 24.0,
                                        ..Default::default()
                                    },
                                ))
                                .observe(|_click: On<Pointer<Click>>,  mut sender: Single<&mut MessageSender<HostStartGameEvent>>| {
                                    info!("🚀 CLIENT: Play button clicked, sending HostStartGameEvent to server");
                                    sender.send::<MetadataChannel>(HostStartGameEvent);
                                });
                        });
                });
            }
        }

        // For the player list, we need to rebuild it completely
        // First, despawn existing player text entities
        for entity in player_text_entities.iter() {
            commands.entity(entity).despawn();
        }

        // Then spawn new player text entities
        for container_entity in player_list_container.iter() {
            commands.entity(container_entity).with_children(|parent| {
                for (i, player_id) in lobby_data.players.iter().enumerate() {
                    let is_host_marker = if *player_id == lobby_data.host_id {
                        " (Host)"
                    } else {
                        ""
                    };
                    let is_you = if *player_id == local_player_id.0 {
                        " (You)"
                    } else {
                        ""
                    };

                    parent.spawn((
                        Text::new(format!("Player {}{}{}", i + 1, is_host_marker, is_you)),
                        TextFont {
                            font_size: 18.0,
                            ..Default::default()
                        },
                        Node {
                            padding: UiRect::bottom(Val::Px(5.0)),
                            ..Default::default()
                        },
                        PlayerText,
                    ));
                }
            });
        }
    }
}
