use crate::ClientGameState;
use crate::LocalPlayerId;
use bevy::color::palettes::tailwind::{GREEN_500, SLATE_700, SLATE_800};
use bevy::ecs::system::SystemParam;

use bevy::ecs::query::Changed;
use bevy::prelude::{
    AlignItems, App, BackgroundColor, Camera2d, Click, Commands, Component, Entity, FlexDirection,
    IntoScheduleConfigs, JustifyContent, Name, Node, On, OnEnter, OnExit, Plugin, Pointer, Query,
    Res, Resource, Single, Text, TextFont, UiRect, Update, Val, With, in_state,
};
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};

use crate::Headless;
use lightyear::prelude::{Confirmed, MessageSender};
use shared::protocol::{HostStartGameEvent, LobbyControlChannel, LobbyState};

#[derive(Resource)]
pub struct AutoStart(pub bool);

pub struct ClientLobbyPlugin;
impl Plugin for ClientLobbyPlugin {
    fn build(&self, app: &mut App) {
        fn is_not_headless(headless: Option<Res<Headless>>) -> bool {
            !headless.map(|h| h.0).unwrap_or(false)
        }

        app.add_systems(
            OnEnter(ClientGameState::Lobby),
            (spawn_lobby_ui, spawn_lobby_camera, ensure_cursor_visible_in_lobby)
                .run_if(is_not_headless),
        );
        app.add_systems(
            OnExit(ClientGameState::Lobby),
            (despawn_lobby_ui, despawn_lobby_camera).run_if(is_not_headless),
        );
        app.add_systems(
            Update,
            (handle_auto_start, update_lobby_text.run_if(is_not_headless))
                .run_if(in_state(ClientGameState::Lobby)),
        );
    }
}

fn ensure_cursor_visible_in_lobby(
    mut cursor_options_query: Query<&mut CursorOptions, With<PrimaryWindow>>,
) {
    if let Ok(mut cursor_options) = cursor_options_query.single_mut() {
        cursor_options.grab_mode = CursorGrabMode::None;
        cursor_options.visible = true;
    }
}

fn handle_auto_start(
    auto_start: Option<Res<AutoStart>>,
    lobby_state: Query<&LobbyState>,
    confirmed_lobby_state: Query<&Confirmed<LobbyState>>,
    local_player_id: Res<LocalPlayerId>,
    mut sender_q: Query<&mut MessageSender<HostStartGameEvent>>,
    mut commands: Commands,
) {
    // Only act when AutoStart is enabled
    if let Some(auto_start_res) = auto_start
        && auto_start_res.0
    {
        // Require lobby replication to be visible client-side
        let replicated_lobby = lobby_state
            .single()
            .ok()
            .cloned()
            .or_else(|| confirmed_lobby_state.single().ok().map(|lobby| lobby.0.clone()));

        if let Some(lobby_data) = replicated_lobby {
            // Require a MessageSender to be present (established link)
            if let Some(mut sender) = sender_q.iter_mut().next() {
                println!(
                    "DEBUG: handle_auto_start running. Host: {}, Local: {}, Players: {}",
                    lobby_data.host_id,
                    local_player_id.0,
                    lobby_data.players.len()
                );
                if lobby_data.host_id == local_player_id.0 && lobby_data.players.len() >= 2 {
                    println!("DEBUG: handle_auto_start sending HostStartGameEvent");
                    sender.send::<LobbyControlChannel>(HostStartGameEvent { requested: true });
                    commands.remove_resource::<AutoStart>();
                }
            } else {
                // No sender yet; wait until the network establishes it
                println!("DEBUG: handle_auto_start - MessageSender not ready yet");
            }
        } else {
            // No lobby yet; will try again on next tick
            println!("DEBUG: handle_auto_start - No LobbyState found");
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
                });
        });
}

fn despawn_lobby_ui(mut commands: Commands, lobby_ui_query: Query<Entity, With<LobbyUI>>) {
    for entity in lobby_ui_query.iter() {
        commands.entity(entity).despawn();
    }
}

#[derive(SystemParam)]
pub struct LobbyUiQueries<'w, 's> {
    pub status_text: Query<'w, 's, &'static mut Text, With<LobbyStatusText>>,
    pub player_list_container: Query<'w, 's, Entity, With<PlayerListContainer>>,
    pub player_text: Query<'w, 's, Entity, With<PlayerText>>,
    pub play_button: Query<'w, 's, Entity, With<PlayButton>>,
    pub lobby_ui: Query<'w, 's, Entity, With<LobbyUI>>,
}

#[allow(clippy::too_many_arguments)]
fn update_lobby_text(
    lobby_state: Query<&LobbyState, Changed<LobbyState>>,
    local_player_id: Res<LocalPlayerId>,
    mut ui_queries: LobbyUiQueries,
    mut commands: Commands,
) {
    if let Ok(lobby_data) = lobby_state.single() {
        let is_host_player = lobby_data.host_id == local_player_id.0;

        for mut status_text in ui_queries.status_text.iter_mut() {
            **status_text = if is_host_player {
                "You are the host - You can start the game.".to_string()
            } else {
                "Waiting for host to start the game...".to_string()
            };
        }

        if is_host_player
            && ui_queries.play_button.is_empty()
            && let Ok(lobby_entity) = ui_queries.lobby_ui.single()
        {
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
                                .observe(|_click: On<Pointer<Click>>,mut commands: Commands , mut sender: Single<&mut MessageSender<HostStartGameEvent>>| {
                                    sender.send::<LobbyControlChannel>(HostStartGameEvent { requested: true });
                                    commands.remove_resource::<AutoStart>();
                                });
                        });
                });
        }

        for entity in ui_queries.player_text.iter() {
            commands.entity(entity).despawn();
        }

        for container_entity in ui_queries.player_list_container.iter() {
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

#[cfg(test)]
mod tests {
    use super::ensure_cursor_visible_in_lobby;
    use bevy::prelude::{App, MinimalPlugins, Update};
    use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};

    #[test]
    fn lobby_cursor_is_visible_and_unlocked() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, ensure_cursor_visible_in_lobby);

        let window = app
            .world_mut()
            .spawn((
                PrimaryWindow,
                CursorOptions {
                    grab_mode: CursorGrabMode::Locked,
                    visible: false,
                    ..Default::default()
                },
            ))
            .id();

        app.update();

        let cursor_options = app
            .world()
            .get::<CursorOptions>(window)
            .expect("primary window should have cursor options");

        assert_eq!(cursor_options.grab_mode, CursorGrabMode::None);
        assert!(cursor_options.visible);
    }
}
