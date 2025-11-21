use crate::menu::{AutoHost, AutoJoin};
use crate::ClientGameState;
use bevy::{
    color::palettes::tailwind::SLATE_800,
    prelude::{
        debug, default, info, AlignItems, App, BackgroundColor, Camera2d, Click, Commands,
        CommandsStatesExt, Component, Entity, FlexDirection, JustifyContent, Name, Node, On,
        OnEnter, OnExit, Plugin, Pointer, Query, Res, Text, TextFont, UiRect, Val, With,
    },
};
use server::create_server_app;
use std::thread;

pub struct LocalMenuPlugin;

impl Plugin for LocalMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(ClientGameState::LocalMenu),
            (conditional_auto_host, conditional_auto_join),
        );
        app.add_systems(
            OnEnter(ClientGameState::LocalMenu),
            (spawn_main_menu_ui, spawn_menu_camera),
        );
        app.add_systems(
            OnExit(ClientGameState::LocalMenu),
            (despawn_menu_camera, despawn_main_menu_ui),
        );
    }
}

fn conditional_auto_host(auto_host: Option<Res<AutoHost>>, mut commands: Commands) {
    if let Some(auto_host_res) = auto_host {
        if auto_host_res.0 {
            commands.remove_resource::<AutoHost>();
            on_host_game(commands);
        }
    }
}

fn conditional_auto_join(auto_join: Option<Res<AutoJoin>>, mut commands: Commands) {
    if let Some(auto_join_res) = auto_join {
        if auto_join_res.0 {
            commands.remove_resource::<AutoJoin>();
            on_join_game(commands);
        }
    }
}

fn on_host_game(mut commands: Commands) {
    let server_handle = thread::spawn(move || {
        let mut server_app = create_server_app(true);
        server_app.run();
    });

    commands.set_state(ClientGameState::Connecting);

    std::mem::forget(server_handle);
    commands.set_state(ClientGameState::Lobby);
}

fn on_join_game(mut commands: Commands) {
    commands.set_state(ClientGameState::Lobby);
}

#[derive(Component)]
pub struct MenuCamera;

fn despawn_menu_camera(mut commands: Commands, q_menu_camera: Query<Entity, With<MenuCamera>>) {
    for entity in &q_menu_camera {
        commands.entity(entity).despawn();
    }
}

fn spawn_menu_camera(mut commands: Commands) {
    commands.spawn((Camera2d, MenuCamera, Name::new("MenuCamera")));
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
                .observe(|_click: On<Pointer<Click>>, commands: Commands| {
                    on_host_game(commands);
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
                .observe(|_click: On<Pointer<Click>>, commands: Commands| {
                    on_join_game(commands);
                });
        });
}

fn despawn_main_menu_ui(mut commands: Commands, q_main_menu: Query<Entity, With<MainMenu>>) {
    for entity in &q_main_menu {
        commands.entity(entity).despawn();
    }
}
