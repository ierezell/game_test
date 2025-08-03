use crate::combat::{Health, Projectile};
use crate::enemies::Enemy;
use crate::game_state::GameState;
use crate::player::{Player, FirstPersonCamera};
use bevy::prelude::*;

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::MainMenu), setup_main_menu)
            .add_systems(OnExit(GameState::MainMenu), cleanup_main_menu)
            .add_systems(OnEnter(GameState::InGame), cleanup_previous_game)
            .add_systems(
                Update,
                (
                    main_menu_system.run_if(in_state(GameState::MainMenu)),
                    check_player_death.run_if(in_state(GameState::InGame)),
                ),
            )
            .add_systems(OnEnter(GameState::GameOver), setup_game_over_menu)
            .add_systems(OnExit(GameState::GameOver), cleanup_game_over_menu)
            .add_systems(
                Update,
                game_over_menu_system.run_if(in_state(GameState::GameOver)),
            );
    }
}

#[derive(Component)]
pub struct MainMenuUI;

#[derive(Component)]
pub struct GameOverUI;

#[derive(Component)]
pub struct MenuButton {
    pub action: MenuAction,
}

#[derive(Clone)]
pub enum MenuAction {
    StartGame,
    QuitGame,
    RestartGame,
    MainMenu,
}

fn setup_main_menu(mut commands: Commands) {
    // Main menu UI
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.9)),
            MainMenuUI,
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("GTFO-Like Game"),
                TextFont {
                    font_size: 48.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    margin: UiRect::bottom(Val::Px(50.0)),
                    ..default()
                },
            ));

            // Start Game Button
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(200.0),
                        height: Val::Px(50.0),
                        margin: UiRect::bottom(Val::Px(20.0)),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
                    MenuButton {
                        action: MenuAction::StartGame,
                    },
                ))
                .with_children(|button| {
                    button.spawn((
                        Text::new("Start Game"),
                        TextFont {
                            font_size: 20.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });

            // Quit Game Button
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(200.0),
                        height: Val::Px(50.0),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
                    MenuButton {
                        action: MenuAction::QuitGame,
                    },
                ))
                .with_children(|button| {
                    button.spawn((
                        Text::new("Quit"),
                        TextFont {
                            font_size: 20.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });

            // Instructions
            parent.spawn((
                Text::new("Press M to open debug menu\nWASD to move, Left Click to shoot\nShift to sprint, Ctrl to crouch"),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
                Node {
                    margin: UiRect::top(Val::Px(50.0)),
                    ..default()
                },
                TextLayout::new_with_justify(JustifyText::Center),
            ));
        });

    info!("Main menu setup complete");
}

fn cleanup_main_menu(mut commands: Commands, menu_query: Query<Entity, With<MainMenuUI>>) {
    for entity in menu_query.iter() {
        commands.entity(entity).despawn();
    }
}

// Clean up all game entities when starting/restarting game
pub fn cleanup_previous_game(
    mut commands: Commands,
    projectile_query: Query<Entity, With<Projectile>>,
    enemy_query: Query<Entity, With<Enemy>>,
    player_query: Query<Entity, (With<Player>, Without<FirstPersonCamera>)>,
) {
    info!("Cleaning up previous game entities...");
    
    // Remove all projectiles
    for entity in projectile_query.iter() {
        commands.entity(entity).despawn();
    }
    
    // Remove all enemies
    for entity in enemy_query.iter() {
        commands.entity(entity).despawn();
    }
    
    // Remove old player if exists (will be respawned by player plugin)
    // BUT DO NOT remove the camera
    for entity in player_query.iter() {
        commands.entity(entity).despawn();
    }
    
    info!("Previous game cleanup complete");
}fn main_menu_system(
    mut interaction_query: Query<
        (&Interaction, &MenuButton, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
    mut next_state: ResMut<NextState<GameState>>,
    mut exit: EventWriter<AppExit>,
) {
    for (interaction, menu_button, mut background_color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => match menu_button.action {
                MenuAction::StartGame => {
                    info!("Starting new game");
                    next_state.set(GameState::InGame);
                }
                MenuAction::QuitGame => {
                    info!("Quitting game");
                    exit.write(AppExit::Success);
                }
                MenuAction::RestartGame => {
                    info!("Restarting game");
                    next_state.set(GameState::InGame);
                }
                MenuAction::MainMenu => {
                    info!("Going to main menu");
                    next_state.set(GameState::MainMenu);
                }
            },
            Interaction::Hovered => {
                *background_color = Color::srgb(0.3, 0.3, 0.3).into();
            }
            Interaction::None => {
                *background_color = Color::srgb(0.2, 0.2, 0.2).into();
            }
        }
    }
}

fn check_player_death(
    player_query: Query<&Health, (With<Player>, Changed<Health>)>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for health in player_query.iter() {
        if health.current <= 0.0 {
            info!("Player died! Going to game over screen");
            next_state.set(GameState::GameOver);
        }
    }
}

fn setup_game_over_menu(mut commands: Commands) {
    // Game over UI
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(Color::srgba(0.5, 0.0, 0.0, 0.9)),
            GameOverUI,
        ))
        .with_children(|parent| {
            // Game Over Title
            parent.spawn((
                Text::new("GAME OVER"),
                TextFont {
                    font_size: 48.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.3, 0.3)),
                Node {
                    margin: UiRect::bottom(Val::Px(50.0)),
                    ..default()
                },
            ));

            // Restart Button
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(200.0),
                        height: Val::Px(50.0),
                        margin: UiRect::bottom(Val::Px(20.0)),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
                    MenuButton {
                        action: MenuAction::RestartGame,
                    },
                ))
                .with_children(|button| {
                    button.spawn((
                        Text::new("Restart"),
                        TextFont {
                            font_size: 20.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });

            // Main Menu Button
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(200.0),
                        height: Val::Px(50.0),
                        margin: UiRect::bottom(Val::Px(20.0)),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
                    MenuButton {
                        action: MenuAction::MainMenu,
                    },
                ))
                .with_children(|button| {
                    button.spawn((
                        Text::new("Main Menu"),
                        TextFont {
                            font_size: 20.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });

            // Quit Button
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(200.0),
                        height: Val::Px(50.0),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
                    MenuButton {
                        action: MenuAction::QuitGame,
                    },
                ))
                .with_children(|button| {
                    button.spawn((
                        Text::new("Quit"),
                        TextFont {
                            font_size: 20.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });
        });
}

fn cleanup_game_over_menu(mut commands: Commands, menu_query: Query<Entity, With<GameOverUI>>) {
    for entity in menu_query.iter() {
        commands.entity(entity).despawn();
    }
}

fn game_over_menu_system(
    mut interaction_query: Query<
        (&Interaction, &MenuButton, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
    mut next_state: ResMut<NextState<GameState>>,
    mut exit: EventWriter<AppExit>,
) {
    for (interaction, menu_button, mut background_color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => match menu_button.action {
                MenuAction::StartGame => {
                    info!("Starting new game");
                    next_state.set(GameState::InGame);
                }
                MenuAction::RestartGame => {
                    info!("Restarting game");
                    next_state.set(GameState::InGame);
                }
                MenuAction::QuitGame => {
                    info!("Quitting game");
                    exit.write(AppExit::Success);
                }
                MenuAction::MainMenu => {
                    info!("Going to main menu");
                    next_state.set(GameState::MainMenu);
                }
            },
            Interaction::Hovered => {
                *background_color = Color::srgb(0.3, 0.3, 0.3).into();
            }
            Interaction::None => {
                *background_color = Color::srgb(0.2, 0.2, 0.2).into();
            }
        }
    }
}
