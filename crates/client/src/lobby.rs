use crate::ClientGameState;
use crate::LocalPlayerId;
use bevy::color::palettes::tailwind::{GREEN_500, SLATE_700, SLATE_800};
use bevy::ecs::system::SystemParam;

use bevy::prelude::{
    AlignItems, App, BackgroundColor, Camera2d, Click, Commands, Component, Entity, FlexDirection,
    IntoScheduleConfigs, JustifyContent, Local, Name, Node, On, OnEnter, OnExit, Plugin, Pointer,
    Query, Res, Resource, Text, TextFont, UiRect, Update, Val, With, in_state,
};
use bevy::text::FontSize;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};

use crate::Headless;
use lightyear::prelude::{Client, MessageSender};
use shared::debug::debug_println;
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
            (
                spawn_lobby_ui,
                spawn_lobby_camera,
                ensure_cursor_visible_in_lobby,
            )
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
    local_player_id: Res<LocalPlayerId>,
    mut sender_q: Query<&mut MessageSender<HostStartGameEvent>, With<Client>>,
) {
    // Only act when AutoStart is enabled
    if let Some(auto_start_res) = auto_start
        && auto_start_res.0
    {
        // Require lobby replication to be visible client-side
        let replicated_lobby = lobby_state.single().ok().cloned();

        if let Some(lobby_data) = replicated_lobby {
            // Require a MessageSender to be present (established link)
            if let Some(mut sender) = sender_q.iter_mut().next() {
                if lobby_data.host_id == local_player_id.0 {
                    debug_println(format_args!(
                        "DEBUG: handle_auto_start sending HostStartGameEvent"
                    ));
                    sender.send::<LobbyControlChannel>(HostStartGameEvent { requested: true });
                }
            } else {
                // No sender yet; wait until the network establishes it
                debug_println(format_args!(
                    "DEBUG: handle_auto_start - MessageSender not ready yet"
                ));
            }
        } else {
            // No lobby yet; will try again on next tick
            debug_println(format_args!("DEBUG: handle_auto_start - No LobbyState found"));
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
                    font_size: FontSize::Px(40.0),
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
                    font_size: FontSize::Px(24.0),
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
                            font_size: FontSize::Px(24.0),
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

fn despawn_lobby_ui(
    mut commands: Commands,
    lobby_ui_query: Query<Entity, With<LobbyUI>>,
    play_button_query: Query<Entity, With<PlayButton>>,
) {
    for entity in lobby_ui_query.iter() {
        commands.entity(entity).despawn();
    }
    for entity in play_button_query.iter() {
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
    lobby_state: Query<&LobbyState>,
    local_player_id: Res<LocalPlayerId>,
    mut ui_queries: LobbyUiQueries,
    mut commands: Commands,
    mut last_rendered: Local<Option<(Vec<u64>, u64, bool)>>,
) {
    if let Ok(lobby_data) = lobby_state.single() {
        let is_host_player = lobby_data.host_id == local_player_id.0;
        let current_signature = (lobby_data.players.clone(), lobby_data.host_id, is_host_player);
        let needs_render = last_rendered.as_ref() != Some(&current_signature)
            || (ui_queries.player_text.is_empty() && !lobby_data.players.is_empty());

        if !is_host_player {
            // Non-host players must never have a play button
            for button_entity in ui_queries.play_button.iter() {
                commands.entity(button_entity).despawn();
            }
        } else {
            // Deduplicate: if multiple play buttons exist, keep only the first
            let mut buttons = ui_queries.play_button.iter();
            if let Some(_first) = buttons.next() {
                for duplicate in buttons {
                    commands.entity(duplicate).despawn();
                }
            } else if let Ok(lobby_entity) = ui_queries.lobby_ui.single() {
                // Host player gets a single play button
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
                                        font_size: FontSize::Px(24.0),
                                        ..Default::default()
                                    },
                                ))
                                .observe(
                                    |_click: On<Pointer<Click>>,
                                     mut commands: Commands,
                                     play_button_q: Query<Entity, With<PlayButton>>,
                                     mut sender_q: Query<
                                        &mut MessageSender<HostStartGameEvent>,
                                        With<Client>,
                                    >| {
                                        for btn in play_button_q.iter() {
                                            commands.entity(btn).despawn();
                                        }
                                        if let Some(mut sender) = sender_q.iter_mut().next() {
                                            sender.send::<LobbyControlChannel>(HostStartGameEvent {
                                                requested: true,
                                            });
                                            commands.remove_resource::<AutoStart>();
                                        }
                                    },
                                );
                        });
                });
            }
        }

        if needs_render {
            for mut status_text in ui_queries.status_text.iter_mut() {
                **status_text = if is_host_player {
                    "You are the host - You can start the game.".to_string()
                } else {
                    "Waiting for host to start the game...".to_string()
                };
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
                            Text::new(format!(
                                "Player {} (ID: {}){}{}",
                                i + 1,
                                player_id,
                                is_host_marker,
                                is_you
                            )),
                            TextFont {
                                font_size: FontSize::Px(18.0),
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

            *last_rendered = Some(current_signature);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        LobbyStatusText, LobbyUI, PlayButton, PlayerListContainer, PlayerText,
        ensure_cursor_visible_in_lobby, update_lobby_text,
    };
    use crate::LocalPlayerId;
    use bevy::prelude::{App, MinimalPlugins, Text, Update, With};
    use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};
    use shared::protocol::LobbyState;

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

    #[test]
    fn host_spawns_play_button_and_non_host_removes_it() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(LocalPlayerId(1));
        app.add_systems(Update, update_lobby_text);

        app.world_mut().spawn(LobbyUI);
        app.world_mut().spawn((LobbyStatusText, Text::new("")));
        app.world_mut().spawn(PlayerListContainer);

        let lobby_entity = app
            .world_mut()
            .spawn(LobbyState {
                players: vec![1, 2],
                host_id: 1,
            })
            .id();

        app.update();

        let play_btn_count = app
            .world_mut()
            .query_filtered::<bevy::prelude::Entity, With<PlayButton>>()
            .iter(app.world())
            .count();
        assert_eq!(play_btn_count, 1, "Host should have exactly one PlayButton");

        // Now host changes to player 2
        let mut lobby = app.world_mut().get_mut::<LobbyState>(lobby_entity).unwrap();
        lobby.host_id = 2;

        app.update();

        let play_btn_count_after = app
            .world_mut()
            .query_filtered::<bevy::prelude::Entity, With<PlayButton>>()
            .iter(app.world())
            .count();
        assert_eq!(
            play_btn_count_after, 0,
            "Non-host should not have any PlayButton"
        );
    }

    #[test]
    fn play_button_duplicates_are_cleaned_up() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(LocalPlayerId(1));
        app.add_systems(Update, update_lobby_text);

        app.world_mut().spawn(LobbyUI);
        app.world_mut().spawn((LobbyStatusText, Text::new("")));
        app.world_mut().spawn(PlayerListContainer);

        // Spawn multiple duplicate play buttons manually
        app.world_mut().spawn(PlayButton);
        app.world_mut().spawn(PlayButton);
        app.world_mut().spawn(PlayButton);

        app.world_mut().spawn(LobbyState {
            players: vec![1],
            host_id: 1,
        });

        app.update();

        let play_btn_count = app
            .world_mut()
            .query_filtered::<bevy::prelude::Entity, With<PlayButton>>()
            .iter(app.world())
            .count();
        assert_eq!(
            play_btn_count, 1,
            "Duplicate PlayButtons should be pruned down to 1"
        );
    }

    #[test]
    fn player_list_representation_formats_labels_properly() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(LocalPlayerId(1));
        app.add_systems(Update, update_lobby_text);

        app.world_mut().spawn(LobbyUI);
        app.world_mut().spawn((LobbyStatusText, Text::new("")));
        let _container = app.world_mut().spawn(PlayerListContainer).id();

        app.world_mut().spawn(LobbyState {
            players: vec![1, 2],
            host_id: 1,
        });

        app.update();

        let world = app.world_mut();
        let mut text_q = world.query_filtered::<&Text, With<PlayerText>>();
        let texts: Vec<String> = text_q.iter(world).map(|t| t.0.clone()).collect();

        assert_eq!(texts.len(), 2, "Expected 2 PlayerText entities");
        assert!(
            texts.iter().any(|t| t == "Player 1 (ID: 1) (Host) (You)"),
            "Expected Player 1 text with Host and You markers, got: {:?}",
            texts
        );
        assert!(
            texts.iter().any(|t| t == "Player 2 (ID: 2)"),
            "Expected Player 2 text without markers, got: {:?}",
            texts
        );
    }
}
