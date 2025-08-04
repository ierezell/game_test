use crate::combat::{Health, Weapon};
use crate::enemies::Enemy;
use crate::menu::GameState;
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
                    update_stamina_ui,
                    update_enemy_health_bars,
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

#[derive(Component)]
pub struct EnemyHealthBar {
    pub enemy_entity: Entity,
}

fn setup_ui(mut commands: Commands) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                ..default()
            },
            GameUI,
        ))
        .with_children(|parent| {
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
                    hud.spawn(Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(30.0),
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        margin: UiRect::bottom(Val::Px(10.0)),
                        ..default()
                    })
                    .with_children(|health_row| {
                        health_row.spawn((
                            Text::new("Health: "),
                            TextFont {
                                font_size: 16.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));

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

                        health_row.spawn((
                            Text::new("Stamina: "),
                            TextFont {
                                font_size: 16.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));

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
            *crosshair_color = if controller.is_sprinting {
                Color::srgba(1.0, 1.0, 0.0, 0.6).into()
            } else if controller.is_crouching {
                Color::srgba(0.0, 1.0, 0.0, 0.8).into()
            } else {
                Color::srgba(1.0, 1.0, 1.0, 0.8).into()
            };
        }
    }
}

fn update_enemy_health_bars(
    mut commands: Commands,
    enemy_query: Query<(Entity, &Health, &GlobalTransform), (With<Enemy>, Without<Player>)>,
    mut health_bar_query: Query<(Entity, &mut Node, &mut BackgroundColor, &EnemyHealthBar)>,
    camera_query: Query<(&Camera, &GlobalTransform), (With<Camera3d>, Without<Enemy>)>,
    existing_bars: Query<&EnemyHealthBar>,
) {
    debug!(
        "Updating enemy health bars. Found {} enemies",
        enemy_query.iter().count()
    );

    if let Ok((camera, camera_transform)) = camera_query.single() {
        for (bar_entity, _, _, health_bar) in health_bar_query.iter() {
            if enemy_query.get(health_bar.enemy_entity).is_err() {
                debug!("Removing health bar for despawned enemy");
                commands.entity(bar_entity).despawn();
            }
        }

        for (enemy_entity, health, enemy_transform) in enemy_query.iter() {
            debug!(
                "Processing enemy {:?} with health {}/{}",
                enemy_entity, health.current, health.maximum
            );

            let has_health_bar = existing_bars
                .iter()
                .any(|bar| bar.enemy_entity == enemy_entity);

            if !has_health_bar {
                debug!("Spawning health bar for enemy {:?}", enemy_entity);
                spawn_enemy_health_bar(&mut commands, enemy_entity);
            } else {
                for (_, mut style, mut bg_color, health_bar) in health_bar_query.iter_mut() {
                    if health_bar.enemy_entity == enemy_entity {
                        let enemy_pos = enemy_transform.translation() + Vec3::Y * 2.5;
                        if let Ok(screen_pos) =
                            camera.world_to_viewport(camera_transform, enemy_pos)
                        {
                            style.left = Val::Px(screen_pos.x - 25.0);
                            style.top = Val::Px(screen_pos.y - 10.0);

                            let health_percent = health.current / health.maximum;
                            style.width = Val::Px(50.0 * health_percent);

                            debug!(
                                "Updating health bar for enemy {:?}: health_percent={}, width={}",
                                enemy_entity,
                                health_percent,
                                50.0 * health_percent
                            );

                            *bg_color = if health_percent > 0.6 {
                                Color::srgb(0.0, 1.0, 0.0).into()
                            } else if health_percent > 0.3 {
                                Color::srgb(1.0, 1.0, 0.0).into()
                            } else {
                                Color::srgb(1.0, 0.0, 0.0).into()
                            };
                        }
                    }
                }
            }
        }
    }
}

fn spawn_enemy_health_bar(commands: &mut Commands, enemy_entity: Entity) {
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            width: Val::Px(50.0),
            height: Val::Px(4.0),
            left: Val::Px(0.0),
            top: Val::Px(0.0),
            border: UiRect::all(Val::Px(1.0)),
            ..default()
        },
        BackgroundColor(Color::srgb(0.0, 1.0, 0.0)),
        BorderColor(Color::srgb(0.0, 0.0, 0.0)),
        EnemyHealthBar { enemy_entity },
        GameUI,
    ));
}
