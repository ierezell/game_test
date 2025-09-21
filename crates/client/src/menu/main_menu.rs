use crate::game_state::GameState;
use bevy::log::debug;
use bevy::prelude::IntoScheduleConfigs;

use bevy::prelude::OnEnter;
use bevy::prelude::{Click, CommandsStatesExt, Entity, Pointer, TextFont, Trigger};
use bevy::render::camera::Camera;
use bevy::{
    color::palettes::tailwind::SLATE_800,
    prelude::{
        AlignItems, App, BackgroundColor, Camera2d, Commands, Component, FlexDirection,
        JustifyContent, Name, Node, Plugin, Query, Text, Transform, UiRect, Val, With, default,
    },
};

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::MainMenu), spawn_main_menu_ui);
        app.add_systems(OnEnter(GameState::MainMenu), spawn_menu_camera);
        app.add_systems(
            OnEnter(GameState::ConnectingRemote),
            (despawn_main_menu_buttons, on_client_begin_connecting).chain(),
        );
        app.add_systems(OnEnter(GameState::Loading), on_client_begin_loading);
        app.add_systems(OnEnter(GameState::Playing), despawn_main_menu_ui);
        // app.add_systems(OnExit(GameState::MainMenu), despawn_menu_camera);
    }
}
#[derive(Component)]
pub struct MenuCamera;

fn spawn_menu_camera(mut commands: Commands) {
    // Bevy 0.16: Use Camera2d component directly for UI rendering
    commands.spawn((
        Camera {
            order: 1,
            ..default()
        },
        Camera2d,
        Transform::from_xyz(0.0, 0.0, 999.9),
        MenuCamera,
        Name::new("MenuCamera"),
    ));
    debug!("Spawned fallback 2D camera for menu (z=999.9)");
}

#[derive(Component)]
pub struct MainMenu;

#[derive(Component)]
pub struct MainMenuStatusText;

#[derive(Component)]
pub struct ConnectButton;

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
                    Text::new("My Game"),
                    TextFont {
                        font_size: 30.,
                        ..default()
                    },
                    Node {
                        padding: UiRect::bottom(Val::Px(200.)),
                        ..default()
                    },
                ))
                .insert(MainMenuStatusText);

            child_builder
                .spawn((
                    Text::new("Connect"),
                    Node {
                        padding: UiRect::bottom(Val::Px(20.)),
                        ..default()
                    },
                ))
                .insert(ConnectButton)
                .observe(|_click: Trigger<Pointer<Click>>, mut commands: Commands| {
                    debug!("Connect button clicked, transitioning to ConnectingRemote");
                    commands.set_state(GameState::ConnectingRemote);
                });
        });
}

fn despawn_main_menu_buttons(
    mut commands: Commands,
    q_connect_buttons: Query<Entity, With<ConnectButton>>,
) {
    for entity in &q_connect_buttons {
        commands.entity(entity).despawn();
    }
    debug!("Despawned main menu connect buttons");
}

fn on_client_begin_loading(mut q_status_text: Query<&mut Text, With<MainMenuStatusText>>) {
    for mut text in q_status_text.iter_mut() {
        text.0 = String::from("Loading game...");
    }
    debug!("Main menu status: Loading game...");
}

fn on_client_begin_connecting(mut q_status_text: Query<&mut Text, With<MainMenuStatusText>>) {
    for mut text in q_status_text.iter_mut() {
        text.0 = String::from("Connecting");
    }
    debug!("Main menu status: Connecting");
}

fn despawn_main_menu_ui(mut commands: Commands, q_main_menu: Query<Entity, With<MainMenu>>) {
    for entity in &q_main_menu {
        commands.entity(entity).despawn();
    }
    debug!("Despawned main menu UI");
}
