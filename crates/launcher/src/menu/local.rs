use bevy::prelude::{
    Click, CommandsStatesExt, Entity, On, OnEnter, Pointer, TextFont, debug, info,
};
use shared::game_state::GameState;

use bevy::{
    color::palettes::tailwind::SLATE_800,
    prelude::{
        AlignItems, App, BackgroundColor, Camera2d, Commands, Component, FlexDirection,
        JustifyContent, Name, Node, Plugin, Query, Text, UiRect, Val, With, default,
    },
};

pub struct LocalMenuPlugin;

impl Plugin for LocalMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::MainMenu),
            (spawn_main_menu_ui, spawn_menu_camera),
        );
        app.add_systems(OnEnter(GameState::Connecting), despawn_main_menu_buttons);
        app.add_systems(OnEnter(GameState::HostingLobby), despawn_main_menu_buttons);
        app.add_systems(OnEnter(GameState::JoiningGame), despawn_main_menu_buttons);
        // Clean up menu camera when leaving main menu for lobby
        app.add_systems(OnEnter(GameState::InLobby), despawn_menu_camera);
        app.add_systems(OnEnter(GameState::Loading), on_client_begin_loading);
        app.add_systems(
            OnEnter(GameState::Playing),
            (despawn_main_menu_ui, despawn_menu_camera),
        );
    }
}

#[derive(Component)]
pub struct MenuCamera;

fn despawn_menu_camera(mut commands: Commands, q_menu_camera: Query<Entity, With<MenuCamera>>) {
    for entity in &q_menu_camera {
        commands.entity(entity).despawn();
    }
    info!("Despawned menu camera");
}

fn spawn_menu_camera(mut commands: Commands) {
    commands.spawn((Camera2d, MenuCamera, Name::new("MenuCamera")));
    debug!("Spawned fallback 2D camera for menu (z=10.0)");
}

#[derive(Component)]
pub struct MainMenu;

#[derive(Component)]
pub struct MainMenuStatusText;

#[derive(Component)]
pub struct HostButton;

#[derive(Component)]
pub struct JoinButton;

fn spawn_main_menu_ui(mut commands: Commands, q_main_menu: Query<Entity, With<MainMenu>>) {
    for entity in &q_main_menu {
        commands.entity(entity).despawn();
    }
    debug!("Spawning main menu UI");

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(SLATE_800.into()),
            MainMenu,
        ))
        .with_children(|child_builder| {
            child_builder
                .spawn((
                    Text::new("Yolo Game"),
                    TextFont {
                        font_size: 30.,
                        ..default()
                    },
                    Node {
                        padding: UiRect::bottom(Val::Px(50.)),
                        ..default()
                    },
                ))
                .insert(MainMenuStatusText);

            child_builder
                .spawn((
                    Text::new("Host Game"),
                    Node {
                        padding: UiRect::bottom(Val::Px(20.)),
                        ..default()
                    },
                ))
                .insert(HostButton)
                .observe(|_click: On<Pointer<Click>>, mut commands: Commands| {
                    debug!("Host button clicked, transitioning to HostingLobby");
                    commands.set_state(GameState::HostingLobby);
                });

            child_builder
                .spawn((
                    Text::new("Join Game"),
                    Node {
                        padding: UiRect::bottom(Val::Px(20.)),
                        ..default()
                    },
                ))
                .insert(JoinButton)
                .observe(|_click: On<Pointer<Click>>, mut commands: Commands| {
                    debug!("Join button clicked, transitioning to JoiningGame");
                    commands.set_state(GameState::JoiningGame);
                });
        });
}

fn despawn_main_menu_buttons(
    mut commands: Commands,
    q_host_buttons: Query<Entity, With<HostButton>>,
    q_join_buttons: Query<Entity, With<JoinButton>>,
) {
    for entity in &q_host_buttons {
        commands.entity(entity).despawn();
    }
    for entity in &q_join_buttons {
        commands.entity(entity).despawn();
    }
    debug!("Despawned main menu buttons");
}

fn on_client_begin_loading(mut q_status_text: Query<&mut Text, With<MainMenuStatusText>>) {
    for mut text in q_status_text.iter_mut() {
        text.0 = String::from("Loading game...");
    }
    debug!("Main menu status: Loading game...");
}

fn despawn_main_menu_ui(
    mut commands: Commands,
    q_main_menu: Query<Entity, With<MainMenu>>,
    mut q_status_text: Query<&mut Text, With<MainMenuStatusText>>,
) {
    for entity in &q_main_menu {
        commands.entity(entity).despawn();
    }

    for mut text in q_status_text.iter_mut() {
        text.0 = String::from("Connecting");
    }
    debug!("Despawned main menu UI");
    debug!("Main menu status: Connecting");
}
