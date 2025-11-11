use crate::ClientGameState;
use crate::menu::{AutoHost, AutoJoin};
use bevy::{
    color::palettes::tailwind::SLATE_800,
    prelude::{
        AlignItems, App, BackgroundColor, Camera2d, Click, Commands, CommandsStatesExt, Component,
        Entity, FlexDirection, JustifyContent, Name, Node, On, OnEnter, OnExit, Plugin, Pointer,
        Query, Res, Text, TextFont, UiRect, Val, With, debug, default, info,
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

/// Check if we should auto-host when entering main menu
fn conditional_auto_host(auto_host: Option<Res<AutoHost>>, mut commands: Commands) {
    if let Some(auto_host_res) = auto_host {
        if auto_host_res.0 {
            info!("Auto-hosting enabled, starting host game sequence");
            commands.remove_resource::<AutoHost>();
            on_host_game(commands);
        }
    }
}

/// Check if we should auto-join when entering main menu
fn conditional_auto_join(auto_join: Option<Res<AutoJoin>>, mut commands: Commands) {
    if let Some(auto_join_res) = auto_join {
        if auto_join_res.0 {
            info!("Auto-join enabled, joining game sequence");
            commands.remove_resource::<AutoJoin>();
            on_join_game(commands);
        }
    }
}

/// System that handles hosting a game
/// Spawns a server in a separate thread and then connects the client to it
fn on_host_game(mut commands: Commands) {
    info!("🏠 Starting to host a game...");

    // Spawn server in a separate thread
    let server_handle = thread::spawn(move || {
        info!("🖥️ Starting server thread...");
        let mut server_app = create_server_app(true);
        server_app.run();
        info!("✅ Server started and running...");
    });

    // Give the server more time to start up properly to avoid port conflicts
    info!("⏳ Waiting for server to initialize...");
    thread::sleep(std::time::Duration::from_millis(3000));

    commands.set_state(ClientGameState::Connecting);

    info!("🚀 Hosting setup complete, connecting client to local server...");

    // Store the server handle so we can clean it up later if needed
    // For now we'll let it run detached
    std::mem::forget(server_handle);
    commands.set_state(ClientGameState::Lobby);
}

/// System that handles joining a game
fn on_join_game(mut commands: Commands) {
    info!("🔗 Joining a game...");
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
