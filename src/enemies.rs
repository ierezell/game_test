#![allow(dead_code)]

use crate::combat::Health;
use crate::game_state::GameState;
use crate::player::Player;
use avian3d::prelude::*;
use bevy::prelude::*;

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        // Removed spawn_test_enemies to avoid duplicate enemies with static level
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
    Striker, // Fast, low health - like GTFO's striker
    Shooter, // Ranged attacks - like GTFO's shooter
    Tank,    // High health, slow - like GTFO's big striker
    Hybrid,  // Balanced
    Sleeper, // Dormant until awakened - classic GTFO mechanic
    Scout,   // Fast moving, calls for reinforcements
}

#[derive(Clone, Copy, PartialEq)]
pub enum EnemyState {
    Dormant, // Sleepers start here
    Idle,
    Patrolling,
    Investigating, // Heard something suspicious
    Chasing,
    Attacking,
    Searching,
    Calling, // Calling for backup
    Dead,
}

#[derive(Component)]
pub struct EnemyAI {
    pub detection_radius: f32,
    pub patrol_points: Vec<Vec3>,
    pub current_patrol_index: usize,
    pub last_known_player_position: Option<Vec3>,
    pub search_timer: f32,
    pub reaction_time: f32,
}

impl Enemy {
    pub fn new_striker() -> Self {
        Self {
            enemy_type: EnemyType::Striker,
            aggro_range: 8.0,
            attack_range: 1.5,
            move_speed: 6.0,
            attack_damage: 25.0,
            attack_cooldown: 1.0,
            last_attack_time: 0.0,
            state: EnemyState::Patrolling,
            target: None,
        }
    }

    pub fn new_shooter() -> Self {
        Self {
            enemy_type: EnemyType::Shooter,
            aggro_range: 12.0,
            attack_range: 8.0,
            move_speed: 3.0,
            attack_damage: 35.0,
            attack_cooldown: 2.0,
            last_attack_time: 0.0,
            state: EnemyState::Patrolling,
            target: None,
        }
    }

    pub fn new_sleeper() -> Self {
        Self {
            enemy_type: EnemyType::Sleeper,
            aggro_range: 15.0, // Wide detection when awakened
            attack_range: 1.5,
            move_speed: 7.0, // Fast when awakened
            attack_damage: 40.0,
            attack_cooldown: 1.5,
            last_attack_time: 0.0,
            state: EnemyState::Dormant, // Start dormant
            target: None,
        }
    }

    pub fn new_scout() -> Self {
        Self {
            enemy_type: EnemyType::Scout,
            aggro_range: 10.0,
            attack_range: 1.2,
            move_speed: 8.0,     // Very fast
            attack_damage: 20.0, // Lower damage but calls backup
            attack_cooldown: 0.8,
            last_attack_time: 0.0,
            state: EnemyState::Patrolling,
            target: None,
        }
    }

    pub fn new_tank() -> Self {
        Self {
            enemy_type: EnemyType::Tank,
            aggro_range: 6.0,
            attack_range: 2.0,
            move_speed: 2.0,
            attack_damage: 50.0,
            attack_cooldown: 3.0,
            last_attack_time: 0.0,
            state: EnemyState::Patrolling,
            target: None,
        }
    }
}

impl EnemyAI {
    pub fn new(patrol_points: Vec<Vec3>) -> Self {
        Self {
            detection_radius: 10.0,
            patrol_points,
            current_patrol_index: 0,
            last_known_player_position: None,
            search_timer: 0.0,
            reaction_time: 0.5,
        }
    }
}

fn spawn_test_enemies(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let striker_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.9, 0.1, 0.1), // Bright red for visibility
        emissive: LinearRgba::new(0.3, 0.0, 0.0, 1.0), // More emissive
        metallic: 0.1,
        perceptual_roughness: 0.8,
        ..default()
    });

    let shooter_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.1, 0.8, 0.1), // Bright green for visibility
        emissive: LinearRgba::new(0.0, 0.3, 0.0, 1.0), // More emissive
        metallic: 0.1,
        perceptual_roughness: 0.8,
        ..default()
    });

    let tank_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.7, 0.7, 0.1), // Bright yellow for visibility
        emissive: LinearRgba::new(0.2, 0.2, 0.0, 1.0), // More emissive
        metallic: 0.3,
        perceptual_roughness: 0.6,
        ..default()
    });

    let sleeper_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.1, 0.1, 0.7), // Bright blue for visibility
        emissive: LinearRgba::new(0.0, 0.0, 0.3, 1.0), // More emissive
        metallic: 0.0,
        perceptual_roughness: 0.9,
        ..default()
    });

    let scout_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.8, 0.4, 0.8), // Bright purple for visibility
        emissive: LinearRgba::new(0.3, 0.1, 0.3, 1.0), // More emissive
        metallic: 0.0,
        perceptual_roughness: 0.7,
        ..default()
    });

    // Spawn a Striker (active patrol) - cube shape, red color
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.4, 0.8, 0.4))), // Cube shape to distinguish
        MeshMaterial3d(striker_material.clone()),
        Transform::from_xyz(-15.0, 0.4, -10.0), // Lower Y position
        RigidBody::Dynamic,
        Collider::cuboid(0.2, 0.4, 0.2), // Smaller collider
        LockedAxes::ROTATION_LOCKED,
        Enemy::new_striker(),
        EnemyAI::new(vec![
            Vec3::new(-15.0, 0.4, -10.0),
            Vec3::new(-12.0, 0.4, -8.0),
            Vec3::new(-15.0, 0.4, -6.0),
        ]),
        Health::new(50.0),
        Name::new("Striker Enemy"),
    ));

    // Spawn Sleepers (dormant until awakened) - smaller and positioned away from spawn
    for i in 0..3 {
        commands.spawn((
            Mesh3d(meshes.add(Capsule3d::new(0.15, 0.6))), // Smaller, crouched appearance
            MeshMaterial3d(sleeper_material.clone()),
            Transform::from_xyz(8.0 + i as f32 * 3.0, 0.3, -5.0), // Lower and spread out
            RigidBody::Dynamic,
            Collider::capsule(0.6, 0.15), // Smaller collider
            LockedAxes::ROTATION_LOCKED,
            Enemy::new_sleeper(),
            EnemyAI::new(vec![]), // No patrol points when sleeping
            Health::new(75.0),
            Name::new(format!("Sleeper Enemy {}", i + 1)),
        ));
    }

    // Spawn a Scout (fast, calls for backup) - smaller and farther away
    commands.spawn((
        Mesh3d(meshes.add(Capsule3d::new(0.15, 0.7))), // Smaller, agile
        MeshMaterial3d(scout_material),
        Transform::from_xyz(20.0, 0.35, 0.0), // Far from spawn point
        RigidBody::Dynamic,
        Collider::capsule(0.7, 0.15), // Smaller collider
        LockedAxes::ROTATION_LOCKED,
        Enemy::new_scout(),
        EnemyAI::new(vec![
            Vec3::new(20.0, 0.35, 0.0),
            Vec3::new(25.0, 0.35, 5.0),
            Vec3::new(22.0, 0.35, 8.0),
            Vec3::new(18.0, 0.35, 3.0),
        ]),
        Health::new(30.0),
        Name::new("Scout Enemy"),
    ));

    // Spawn a Shooter (ranged enemy) - smaller and positioned away
    commands.spawn((
        Mesh3d(meshes.add(Capsule3d::new(0.2, 0.9))), // Smaller but slightly taller
        MeshMaterial3d(shooter_material),
        Transform::from_xyz(-8.0, 0.45, 5.0), // Lower position
        RigidBody::Dynamic,
        Collider::capsule(0.9, 0.2), // Smaller collider
        LockedAxes::ROTATION_LOCKED,
        Enemy::new_shooter(),
        EnemyAI::new(vec![
            Vec3::new(-8.0, 0.45, 5.0),
            Vec3::new(-5.0, 0.45, 7.0),
            Vec3::new(-10.0, 0.45, 8.0),
        ]),
        Health::new(75.0),
        Name::new("Shooter Enemy"),
    ));

    // Spawn a Tank (slow but powerful) - sphere shape, yellow color
    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(0.4))), // Sphere shape to distinguish as tank
        MeshMaterial3d(tank_material),
        Transform::from_xyz(0.0, 0.4, -20.0), // Far from spawn point
        RigidBody::Dynamic,
        Collider::sphere(0.4), // Sphere collider
        LockedAxes::ROTATION_LOCKED,
        Enemy::new_tank(),
        EnemyAI::new(vec![
            Vec3::new(0.0, 0.4, -20.0),
            Vec3::new(-3.0, 0.4, -18.0),
            Vec3::new(3.0, 0.4, -18.0),
        ]),
        Health::new(150.0),
        Name::new("Tank Enemy"),
    ));

    info!("Spawned test enemies: 1 Striker, 3 Sleepers, 1 Scout, 1 Shooter, 1 Tank");
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

        // Find closest player
        let mut closest_player: Option<(Entity, f32)> = None;
        for (player_entity, player_transform) in player_query.iter() {
            let distance = enemy_transform
                .translation
                .distance(player_transform.translation);

            if distance <= ai.detection_radius {
                if let Some((_, closest_distance)) = closest_player {
                    if distance < closest_distance {
                        closest_player = Some((player_entity, distance));
                    }
                } else {
                    closest_player = Some((player_entity, distance));
                }
            }
        }

        // Update AI state based on player detection
        match enemy.state {
            EnemyState::Dormant => {
                // Sleepers stay dormant until awakened by sound/damage
                // This is handled in the sleeper_awakening_system
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
                // Move towards suspicious sound/location
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
                    // Lost sight of player, start searching
                    enemy.state = EnemyState::Searching;
                    ai.search_timer = 5.0; // Search for 5 seconds
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

                // Check if player comes back into range during search
                if let Some((player_entity, _)) = closest_player {
                    enemy.state = EnemyState::Chasing;
                    enemy.target = Some(player_entity);
                }
            }
            EnemyState::Calling => {
                // Scout calls for backup - handled in separate system
                ai.search_timer -= time.delta_secs();
                if ai.search_timer <= 0.0 {
                    enemy.state = EnemyState::Chasing;
                }
            }
            EnemyState::Dead => {
                // Dead enemies don't do anything
            }
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
                        // Only calculate movement in X and Z directions (horizontal)
                        let horizontal_direction = Vec3::new(
                            target_point.x - transform.translation.x,
                            0.0, // Don't move in Y direction
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
                        // Only chase horizontally, don't fly towards player
                        let horizontal_direction = Vec3::new(
                            player_transform.translation.x - transform.translation.x,
                            0.0, // Don't move in Y direction
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
                        // Only search horizontally
                        let horizontal_direction = Vec3::new(
                            last_pos.x - transform.translation.x,
                            0.0, // Don't move in Y direction
                            last_pos.z - transform.translation.z,
                        )
                        .normalize();
                        movement_direction = horizontal_direction;
                    }
                }
            }
            EnemyState::Attacking => {
                // Don't move while attacking, just face the target
                if let Some(target_entity) = enemy.target {
                    if let Ok(player_transform) = player_query.get(target_entity) {
                        // Only look horizontally, don't tilt up/down
                        let look_direction = Vec3::new(
                            player_transform.translation.x - transform.translation.x,
                            0.0, // Keep level, don't look up or down
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

        // Apply movement
        if movement_direction.length() > 0.1 {
            velocity.x = movement_direction.x * enemy.move_speed;
            velocity.z = movement_direction.z * enemy.move_speed;
            // Keep Y velocity for gravity but don't add movement
            // velocity.y remains unchanged (for gravity/physics)

            // Rotate to face movement direction
            transform.look_to(movement_direction, Vec3::Y);
        } else {
            velocity.x *= 0.8; // Apply friction
            velocity.z *= 0.8;
            // Don't modify Y velocity - let gravity handle it
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

                        info!(
                            "Enemy {} attacked player for {} damage!",
                            match enemy.enemy_type {
                                EnemyType::Striker => "Striker",
                                EnemyType::Shooter => "Shooter",
                                EnemyType::Tank => "Tank",
                                EnemyType::Hybrid => "Hybrid",
                                EnemyType::Sleeper => "Sleeper",
                                EnemyType::Scout => "Scout",
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
    _commands: Commands,
    mut enemy_query: Query<(Entity, &mut Enemy, &Health), Without<crate::player::Player>>,
) {
    for (_entity, mut enemy, health) in enemy_query.iter_mut() {
        if health.is_dead() && enemy.state != EnemyState::Dead {
            enemy.state = EnemyState::Dead;
            info!("Enemy died!");

            // In a real game, you might want to play death animation,
            // drop loot, etc. For now, we'll just mark it as dead
            // You could despawn after a delay:
            // commands.entity(entity).despawn();
        }
    }
}

// GTFO-like sleeper awakening system
fn sleeper_awakening_system(
    mut enemy_query: Query<(&mut Enemy, &mut EnemyAI, &Transform)>,
    player_query: Query<&Transform, (With<Player>, Without<Enemy>)>,
    _time: Res<Time>,
) {
    for (mut enemy, mut ai, transform) in enemy_query.iter_mut() {
        if enemy.enemy_type == EnemyType::Sleeper && enemy.state == EnemyState::Dormant {
            // Check if player is too close
            for player_transform in player_query.iter() {
                let distance = transform.translation.distance(player_transform.translation);

                // Awaken if player gets too close (like in GTFO)
                if distance < 3.0 {
                    enemy.state = EnemyState::Chasing;
                    ai.last_known_player_position = Some(player_transform.translation);
                    info!("Sleeper awakened by close proximity!");
                    break;
                }
            }
        }
    }
}

// Sound detection system for stealth mechanics
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

                // Detect running/sprinting players from further away
                let detection_range = if player_controller.is_sprinting {
                    15.0
                } else if player_controller.speed > 3.0 {
                    8.0
                } else {
                    5.0 // Walking/crouching
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
