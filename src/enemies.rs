use crate::combat::Health;
use crate::menu::GameState;
use crate::player::Player;
use avian3d::prelude::*;
use bevy::prelude::*;

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                enemy_ai_system,
                enemy_movement,
                enemy_attack_system,
                enemy_death_system,
                sleeper_awakening_system,
                sound_detection_system,
            )
                .run_if(in_state(GameState::InGame)),
        );
    }
}

#[derive(Component)]
pub struct Enemy {
    pub enemy_type: EnemyType,
    pub aggro_range: f32,
    pub attack_range: f32,
    pub move_speed: f32,
    pub attack_damage: f32,
    pub attack_cooldown: f32,
    pub last_attack_time: f32,
    pub state: EnemyState,
    pub target: Option<Entity>,
}

#[derive(Clone, Copy, PartialEq)]
pub enum EnemyType {
    Striker,
}

#[derive(Clone, Copy, PartialEq)]
pub enum EnemyState {
    Dormant,
    Idle,
    Patrolling,
    Investigating,
    Chasing,
    Attacking,
    Searching,

    Dead,
}

#[derive(Component)]
pub struct EnemyAI {
    pub detection_radius: f32,
    pub patrol_points: Vec<Vec3>,
    pub current_patrol_index: usize,
    pub last_known_player_position: Option<Vec3>,
    pub search_timer: f32,
}

fn enemy_ai_system(
    time: Res<Time>,
    mut enemy_query: Query<
        (Entity, &mut Enemy, &mut EnemyAI, &Transform),
        Without<crate::player::Player>,
    >,
    player_query: Query<(Entity, &Transform), With<crate::player::Player>>,
) {
    for (_enemy_entity, mut enemy, mut ai, enemy_transform) in enemy_query.iter_mut() {
        let _current_time = time.elapsed_secs();

        let mut closest_player: Option<(Entity, f32)> = None;
        for (player_entity, player_transform) in player_query.iter() {
            let distance = enemy_transform
                .translation
                .distance(player_transform.translation);

            // For dormant enemies, always check proximity regardless of detection radius
            // For active enemies, use detection radius
            let should_detect =
                enemy.state == EnemyState::Dormant || distance <= ai.detection_radius;

            if should_detect {
                if let Some((_, closest_distance)) = closest_player {
                    if distance < closest_distance {
                        closest_player = Some((player_entity, distance));
                    }
                } else {
                    closest_player = Some((player_entity, distance));
                }
            }
        }

        match enemy.state {
            EnemyState::Dormant => {
                // Dormant enemies should wake up when players get close
                for (player_entity, player_transform) in player_query.iter() {
                    let distance = enemy_transform
                        .translation
                        .distance(player_transform.translation);

                    // Wake up if player is within 3.0 units (close proximity)
                    if distance <= 3.0 {
                        enemy.state = EnemyState::Chasing;
                        enemy.target = Some(player_entity);
                        ai.last_known_player_position = Some(player_transform.translation);
                        debug!(
                            "Enemy awakened by close proximity! Distance: {:.2}",
                            distance
                        );
                        break;
                    } else if distance <= 5.0 {
                        // Debug: log when player is getting close but not close enough
                        debug!(
                            "Player approaching dormant enemy. Distance: {:.2}",
                            distance
                        );
                    }
                }
            }
            EnemyState::Idle | EnemyState::Patrolling => {
                if let Some((player_entity, distance)) = closest_player {
                    if distance <= enemy.aggro_range {
                        enemy.state = EnemyState::Chasing;
                        enemy.target = Some(player_entity);
                        ai.last_known_player_position = player_query
                            .get(player_entity)
                            .ok()
                            .map(|(_, transform)| transform.translation);
                    }
                }
            }
            EnemyState::Investigating => {
                if let Some((player_entity, distance)) = closest_player {
                    if distance <= enemy.aggro_range {
                        enemy.state = EnemyState::Chasing;
                        enemy.target = Some(player_entity);
                    }
                }
                ai.search_timer -= time.delta_secs();
                if ai.search_timer <= 0.0 {
                    enemy.state = EnemyState::Patrolling;
                }
            }
            EnemyState::Chasing => {
                if let Some((player_entity, distance)) = closest_player {
                    enemy.target = Some(player_entity);
                    ai.last_known_player_position = player_query
                        .get(player_entity)
                        .ok()
                        .map(|(_, transform)| transform.translation);

                    if distance <= enemy.attack_range {
                        enemy.state = EnemyState::Attacking;
                    }
                } else {
                    enemy.state = EnemyState::Searching;
                    ai.search_timer = 5.0;
                }
            }
            EnemyState::Attacking => {
                if let Some((_, distance)) = closest_player {
                    if distance > enemy.attack_range {
                        enemy.state = EnemyState::Chasing;
                    }
                } else {
                    enemy.state = EnemyState::Searching;
                    ai.search_timer = 3.0;
                }
            }
            EnemyState::Searching => {
                ai.search_timer -= time.delta_secs();
                if ai.search_timer <= 0.0 {
                    enemy.state = EnemyState::Patrolling;
                    enemy.target = None;
                    ai.last_known_player_position = None;
                }

                if let Some((player_entity, _)) = closest_player {
                    enemy.state = EnemyState::Chasing;
                    enemy.target = Some(player_entity);
                }
            }

            EnemyState::Dead => {}
        }
    }
}

fn enemy_movement(
    _time: Res<Time>,
    mut enemy_query: Query<
        (&Enemy, &mut EnemyAI, &mut LinearVelocity, &mut Transform),
        Without<crate::player::Player>,
    >,
    player_query: Query<&Transform, With<crate::player::Player>>,
) {
    for (enemy, mut ai, mut velocity, mut transform) in enemy_query.iter_mut() {
        let mut movement_direction = Vec3::ZERO;

        match enemy.state {
            EnemyState::Patrolling => {
                if !ai.patrol_points.is_empty() {
                    let target_point = ai.patrol_points[ai.current_patrol_index];
                    let distance_to_point = transform.translation.distance(target_point);

                    if distance_to_point < 1.0 {
                        ai.current_patrol_index =
                            (ai.current_patrol_index + 1) % ai.patrol_points.len();
                    } else {
                        let horizontal_direction = Vec3::new(
                            target_point.x - transform.translation.x,
                            0.0,
                            target_point.z - transform.translation.z,
                        )
                        .normalize();
                        movement_direction = horizontal_direction;
                    }
                }
            }
            EnemyState::Chasing => {
                if let Some(target_entity) = enemy.target {
                    if let Ok(player_transform) = player_query.get(target_entity) {
                        let horizontal_direction = Vec3::new(
                            player_transform.translation.x - transform.translation.x,
                            0.0,
                            player_transform.translation.z - transform.translation.z,
                        )
                        .normalize();
                        movement_direction = horizontal_direction;
                        ai.last_known_player_position = Some(player_transform.translation);
                    }
                }
            }
            EnemyState::Searching => {
                if let Some(last_pos) = ai.last_known_player_position {
                    let distance_to_last_pos = transform.translation.distance(last_pos);
                    if distance_to_last_pos > 1.0 {
                        let horizontal_direction = Vec3::new(
                            last_pos.x - transform.translation.x,
                            0.0,
                            last_pos.z - transform.translation.z,
                        )
                        .normalize();
                        movement_direction = horizontal_direction;
                    }
                }
            }
            EnemyState::Attacking => {
                if let Some(target_entity) = enemy.target {
                    if let Ok(player_transform) = player_query.get(target_entity) {
                        let look_direction = Vec3::new(
                            player_transform.translation.x - transform.translation.x,
                            0.0,
                            player_transform.translation.z - transform.translation.z,
                        )
                        .normalize();
                        if look_direction.length() > 0.1 {
                            transform.look_to(look_direction, Vec3::Y);
                        }
                    }
                }
            }
            _ => {}
        }

        if movement_direction.length() > 0.1 {
            velocity.x = movement_direction.x * enemy.move_speed;
            velocity.z = movement_direction.z * enemy.move_speed;

            transform.look_to(movement_direction, Vec3::Y);
        } else {
            velocity.x *= 0.8;
            velocity.z *= 0.8;
        }
    }
}

fn enemy_attack_system(
    time: Res<Time>,
    mut enemy_query: Query<(&mut Enemy, &Transform), Without<crate::player::Player>>,
    mut player_query: Query<&mut crate::combat::Health, With<crate::player::Player>>,
) {
    let current_time = time.elapsed_secs();

    for (mut enemy, _enemy_transform) in enemy_query.iter_mut() {
        if enemy.state == EnemyState::Attacking {
            if current_time - enemy.last_attack_time >= enemy.attack_cooldown {
                if let Some(target_entity) = enemy.target {
                    if let Ok(mut player_health) = player_query.get_mut(target_entity) {
                        player_health.take_damage(enemy.attack_damage, current_time);
                        enemy.last_attack_time = current_time;

                        debug!(
                            "Enemy {} attacked player for {} damage!",
                            match enemy.enemy_type {
                                EnemyType::Striker => "Striker",
                            },
                            enemy.attack_damage
                        );
                    }
                }
            }
        }
    }
}

fn enemy_death_system(
    mut enemy_query: Query<(Entity, &mut Enemy, &Health), Without<crate::player::Player>>,
) {
    for (entity, mut enemy, health) in enemy_query.iter_mut() {
        debug!(
            "Enemy {:?} health check: current={}, max={}, is_dead={}",
            entity,
            health.current,
            health.maximum,
            health.is_dead()
        );

        if health.is_dead() && enemy.state != EnemyState::Dead {
            enemy.state = EnemyState::Dead;
            debug!(
                "Enemy {:?} marked as dead (health: {})",
                entity, health.current
            );
        }
    }
}

fn sleeper_awakening_system(
    mut enemy_query: Query<(&mut Enemy, &mut EnemyAI, &Transform)>,
    player_query: Query<&Transform, (With<Player>, Without<Enemy>)>,
    _time: Res<Time>,
) {
    for (mut enemy, mut ai, transform) in enemy_query.iter_mut() {
        // This system handles sleeper-specific awakening (different from general dormant awakening)
        if enemy.state == EnemyState::Dormant {
            for player_transform in player_query.iter() {
                let distance = transform.translation.distance(player_transform.translation);

                // Sleepers have a slightly larger awakening range
                if distance < 4.0 {
                    enemy.state = EnemyState::Chasing;
                    ai.last_known_player_position = Some(player_transform.translation);
                    debug!(
                        "Sleeper awakened by close proximity! Distance: {:.2}",
                        distance
                    );
                    break;
                }
            }
        }
    }
}

fn sound_detection_system(
    mut enemy_query: Query<(&mut Enemy, &mut EnemyAI, &Transform)>,
    player_query: Query<(&crate::player::PlayerController, &Transform), With<Player>>,
) {
    for (mut enemy, mut ai, enemy_transform) in enemy_query.iter_mut() {
        if enemy.state == EnemyState::Patrolling || enemy.state == EnemyState::Idle {
            for (player_controller, player_transform) in player_query.iter() {
                let distance = enemy_transform
                    .translation
                    .distance(player_transform.translation);

                let detection_range = if player_controller.is_sprinting {
                    15.0
                } else if player_controller.speed > 3.0 {
                    8.0
                } else {
                    5.0
                };

                if distance < detection_range && enemy.state != EnemyState::Chasing {
                    enemy.state = EnemyState::Investigating;
                    ai.search_timer = 3.0;
                    ai.last_known_player_position = Some(player_transform.translation);
                    break;
                }
            }
        }
    }
}
