use crate::combat::{Health, Weapon};
use crate::game_state::GameState;
use crate::player::Player;
use bevy::prelude::*;

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::InGame), setup_ui)
            .add_systems(OnExit(GameState::InGame), cleanup_ui)
            .add_systems(
                Update,
                (
                    update_health_ui,
                    update_ammo_ui,
                    update_crosshair,
                    update_stamina_ui, // Add the missing stamina update system
                )
                    .run_if(in_state(GameState::InGame)),
            );
    }
}

#[derive(Component)]
pub struct GameUI;

#[derive(Component)]
pub struct HealthBar;

#[derive(Component)]
pub struct AmmoDisplay;

#[derive(Component)]
pub struct Crosshair;

#[derive(Component)]
pub struct StaminaBar;

fn setup_ui(mut commands: Commands) {
    // UI Root
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                ..default()
            },
            GameUI, // Mark as game UI for cleanup
        ))
        .with_children(|parent| {
            // Crosshair
            parent
                .spawn((
                    Node {
                        width: Val::Px(20.0),
                        height: Val::Px(20.0),
                        position_type: PositionType::Absolute,
                        left: Val::Percent(50.0),
                        top: Val::Percent(50.0),
                        margin: UiRect::all(Val::Px(-10.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.8)),
                    Crosshair,
                ))
                .with_children(|crosshair| {
                    // Horizontal line
                    crosshair.spawn((
                        Node {
                            width: Val::Px(10.0),
                            height: Val::Px(2.0),
                            position_type: PositionType::Absolute,
                            left: Val::Px(5.0),
                            top: Val::Px(9.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.9)),
                    ));

                    // Vertical line
                    crosshair.spawn((
                        Node {
                            width: Val::Px(2.0),
                            height: Val::Px(10.0),
                            position_type: PositionType::Absolute,
                            left: Val::Px(9.0),
                            top: Val::Px(5.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.9)),
                    ));
                });

            // Bottom HUD
            parent
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(120.0),
                        position_type: PositionType::Absolute,
                        bottom: Val::Px(0.0),
                        padding: UiRect::all(Val::Px(20.0)),
                        flex_direction: FlexDirection::Column,
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
                ))
                .with_children(|hud| {
                    // Health and Stamina row
                    hud.spawn(Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(30.0),
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        margin: UiRect::bottom(Val::Px(10.0)),
                        ..default()
                    })
                    .with_children(|health_row| {
                        // Health label
                        health_row.spawn((
                            Text::new("Health: "),
                            TextFont {
                                font_size: 16.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));

                        // Health bar background
                        health_row
                            .spawn((
                                Node {
                                    width: Val::Px(200.0),
                                    height: Val::Px(20.0),
                                    margin: UiRect::right(Val::Px(20.0)),
                                    ..default()
                                },
                                BackgroundColor(Color::srgb(0.3, 0.1, 0.1)),
                            ))
                            .with_children(|health_bg| {
                                // Health bar fill
                                health_bg.spawn((
                                    Node {
                                        width: Val::Percent(100.0),
                                        height: Val::Percent(100.0),
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgb(0.8, 0.2, 0.2)),
                                    HealthBar,
                                ));
                            });

                        // Stamina label
                        health_row.spawn((
                            Text::new("Stamina: "),
                            TextFont {
                                font_size: 16.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));

                        // Stamina bar background
                        health_row
                            .spawn((
                                Node {
                                    width: Val::Px(200.0),
                                    height: Val::Px(20.0),
                                    ..default()
                                },
                                BackgroundColor(Color::srgb(0.1, 0.1, 0.3)),
                            ))
                            .with_children(|stamina_bg| {
                                // Stamina bar fill
                                stamina_bg.spawn((
                                    Node {
                                        width: Val::Percent(100.0),
                                        height: Val::Percent(100.0),
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgb(0.2, 0.2, 0.8)),
                                    StaminaBar,
                                ));
                            });
                    });

                    // Ammo display
                    hud.spawn((
                        Text::new("Ammo: 30/30"),
                        TextFont {
                            font_size: 24.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        Node {
                            align_self: AlignSelf::FlexEnd,
                            ..default()
                        },
                        AmmoDisplay,
                    ));
                });
        });
}

fn cleanup_ui(mut commands: Commands, ui_query: Query<Entity, With<GameUI>>) {
    for entity in ui_query.iter() {
        commands.entity(entity).despawn();
    }
}

fn update_health_ui(
    player_query: Query<&Health, (With<Player>, Changed<Health>)>,
    mut health_bar_query: Query<&mut Node, With<HealthBar>>,
) {
    for health in player_query.iter() {
        if let Ok(mut node) = health_bar_query.single_mut() {
            let health_percentage = (health.current / health.maximum) * 100.0;
            node.width = Val::Percent(health_percentage);
        }
    }
}

fn update_ammo_ui(
    weapon_query: Query<&Weapon, Changed<Weapon>>,
    mut ammo_display_query: Query<(&mut Text, &mut TextColor), With<AmmoDisplay>>,
) {
    for weapon in weapon_query.iter() {
        if let Ok((mut text, mut text_color)) = ammo_display_query.single_mut() {
            if weapon.is_reloading {
                **text = "Reloading...".to_string();
                text_color.0 = Color::srgb(1.0, 1.0, 0.0);
            } else {
                **text = format!("Ammo: {}/{}", weapon.ammo_current, weapon.ammo_max);
                text_color.0 = if weapon.ammo_current == 0 {
                    Color::srgb(1.0, 0.3, 0.3)
                } else if weapon.ammo_current < weapon.ammo_max / 4 {
                    Color::srgb(1.0, 0.7, 0.3)
                } else {
                    Color::WHITE
                };
            }
        }
    }
}

fn update_stamina_ui(
    player_query: Query<
        &crate::player::PlayerController,
        (With<Player>, Changed<crate::player::PlayerController>),
    >,
    mut stamina_bar_query: Query<&mut Node, With<StaminaBar>>,
) {
    for controller in player_query.iter() {
        if let Ok(mut node) = stamina_bar_query.single_mut() {
            let stamina_percentage = (controller.stamina / controller.max_stamina) * 100.0;
            node.width = Val::Percent(stamina_percentage);
        }
    }
}

fn update_crosshair(
    player_query: Query<&crate::player::PlayerController, With<Player>>,
    mut crosshair_query: Query<&mut BackgroundColor, With<Crosshair>>,
) {
    for controller in player_query.iter() {
        if let Ok(mut crosshair_color) = crosshair_query.single_mut() {
            // Change crosshair color based on player state
            *crosshair_color = if controller.is_sprinting {
                Color::srgba(1.0, 1.0, 0.0, 0.6).into() // Yellow when sprinting
            } else if controller.is_crouching {
                Color::srgba(0.0, 1.0, 0.0, 0.8).into() // Green when crouching
            } else {
                Color::srgba(1.0, 1.0, 1.0, 0.8).into() // White default
            };
        }
    }
}
