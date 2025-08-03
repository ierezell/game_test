#![allow(dead_code)]

//! Integration tests for the GTFO-like game
//! Tests core gameplay mechanics end-to-end

use crate::combat::{Health, Projectile};
use crate::enemies::{Enemy, EnemyAI, EnemyState, EnemyType};
use crate::game_state::GameState;
use crate::player::{Player, PlayerAction};
use avian3d::prelude::*;
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

pub struct IntegrationTestPlugin;

impl Plugin for IntegrationTestPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                test_player_movement,
                test_player_shooting,
                test_enemy_damage,
                test_game_restart,
                run_comprehensive_test,
            )
                .run_if(in_state(GameState::InGame)),
        );
    }
}

#[derive(Resource, Default)]
pub struct TestResults {
    pub movement_test_passed: bool,
    pub shooting_test_passed: bool,
    pub enemy_damage_test_passed: bool,
    pub restart_test_passed: bool,
    pub tests_completed: bool,
}

#[derive(Component)]
pub struct TestMarker;

/// Test 1: Verify player movement works
fn test_player_movement(
    mut test_results: ResMut<TestResults>,
    mut player_query: Query<(&mut Transform, &mut ActionState<PlayerAction>), With<Player>>,
    time: Res<Time>,
) {
    if test_results.movement_test_passed {
        return;
    }

    for (transform, mut action_state) in player_query.iter_mut() {
        let initial_position = transform.translation;

        // Simulate WASD input
        action_state.press(&PlayerAction::Move);
        action_state.set_axis_pair(&PlayerAction::Move, Vec2::new(1.0, 0.0));

        // After a short time, check if player moved
        if time.elapsed_secs() > 1.0 {
            let new_position = transform.translation;
            let distance_moved = initial_position.distance(new_position);

            if distance_moved > 0.1 {
                info!(
                    "✓ Movement test PASSED - Player moved {:.2} units",
                    distance_moved
                );
                test_results.movement_test_passed = true;
            } else {
                warn!(
                    "✗ Movement test FAILED - Player didn't move (distance: {:.4})",
                    distance_moved
                );
            }
        }
    }
}

/// Test 2: Verify player can shoot and projectiles spawn
fn test_player_shooting(
    mut test_results: ResMut<TestResults>,
    mut player_query: Query<&mut ActionState<PlayerAction>, With<Player>>,
    projectile_query: Query<Entity, With<Projectile>>,
    time: Res<Time>,
) {
    if test_results.shooting_test_passed {
        return;
    }

    // Start shooting test after movement test
    if !test_results.movement_test_passed || time.elapsed_secs() < 2.0 {
        return;
    }

    let initial_projectile_count = projectile_query.iter().count();

    for mut action_state in player_query.iter_mut() {
        // Simulate primary fire input
        action_state.press(&PlayerAction::PrimaryFire);

        // Check if projectiles were created after a short delay
        if time.elapsed_secs() > 3.0 {
            let new_projectile_count = projectile_query.iter().count();

            if new_projectile_count > initial_projectile_count {
                info!(
                    "✓ Shooting test PASSED - {} projectiles spawned",
                    new_projectile_count
                );
                test_results.shooting_test_passed = true;
            } else {
                warn!("✗ Shooting test FAILED - No projectiles spawned");
            }
        }
    }
}

/// Test 3: Verify enemy takes damage and dies when shot
fn test_enemy_damage(
    mut test_results: ResMut<TestResults>,
    mut commands: Commands,
    enemy_query: Query<(Entity, &Health), With<Enemy>>,
    time: Res<Time>,
) {
    if test_results.enemy_damage_test_passed {
        return;
    }

    // Start damage test after shooting test
    if !test_results.shooting_test_passed || time.elapsed_secs() < 4.0 {
        return;
    }

    // Check if any enemies have taken damage or died
    let mut enemy_damaged = false;
    let mut enemy_died = false;

    for (entity, health) in enemy_query.iter() {
        if health.current < health.maximum {
            enemy_damaged = true;
            info!(
                "✓ Enemy took damage - Health: {}/{}",
                health.current, health.maximum
            );
        }

        if health.current <= 0.0 {
            enemy_died = true;
            info!("✓ Enemy died - Removing from world");
            commands.entity(entity).despawn();
        }
    }

    // Check if enemy was completely removed (despawned)
    if time.elapsed_secs() > 6.0 {
        let remaining_enemies = enemy_query.iter().count();
        if enemy_damaged || enemy_died || remaining_enemies == 0 {
            info!("✓ Enemy damage test PASSED - Enemy was damaged/killed");
            test_results.enemy_damage_test_passed = true;
        } else {
            warn!("✗ Enemy damage test FAILED - Enemy was not damaged");
        }
    }
}

/// Test 4: Verify game restart functionality works
fn test_game_restart(
    mut test_results: ResMut<TestResults>,
    mut next_state: ResMut<NextState<GameState>>,
    player_query: Query<Entity, With<Player>>,
    enemy_query: Query<Entity, With<Enemy>>,
    projectile_query: Query<Entity, With<Projectile>>,
    time: Res<Time>,
) {
    if test_results.restart_test_passed {
        return;
    }

    // Start restart test after damage test
    if !test_results.enemy_damage_test_passed || time.elapsed_secs() < 8.0 {
        return;
    }

    // Trigger restart by going to game over and back to in-game
    if time.elapsed_secs() > 8.0 && time.elapsed_secs() < 8.1 {
        info!("🔄 Testing game restart...");
        next_state.set(GameState::GameOver);
    }

    if time.elapsed_secs() > 9.0 && time.elapsed_secs() < 9.1 {
        next_state.set(GameState::InGame);
    }

    // Check if entities were properly cleaned up and respawned
    if time.elapsed_secs() > 10.0 {
        let player_count = player_query.iter().count();
        let enemy_count = enemy_query.iter().count();
        let projectile_count = projectile_query.iter().count();

        if player_count == 1 && enemy_count >= 1 && projectile_count == 0 {
            info!(
                "✓ Restart test PASSED - Entities properly reset (P:{}, E:{}, Pr:{})",
                player_count, enemy_count, projectile_count
            );
            test_results.restart_test_passed = true;
        } else {
            warn!(
                "✗ Restart test FAILED - Entities not properly reset (P:{}, E:{}, Pr:{})",
                player_count, enemy_count, projectile_count
            );
        }
    }
}

/// Comprehensive test runner that reports final results
fn run_comprehensive_test(mut test_results: ResMut<TestResults>, time: Res<Time>) {
    if test_results.tests_completed {
        return;
    }

    // Run final report after all tests
    if time.elapsed_secs() > 12.0 {
        let all_passed = test_results.movement_test_passed
            && test_results.shooting_test_passed
            && test_results.enemy_damage_test_passed
            && test_results.restart_test_passed;

        info!("🧪 =============== TEST RESULTS ===============");
        info!(
            "🧪 Movement Test:      {}",
            if test_results.movement_test_passed {
                "✓ PASS"
            } else {
                "✗ FAIL"
            }
        );
        info!(
            "🧪 Shooting Test:      {}",
            if test_results.shooting_test_passed {
                "✓ PASS"
            } else {
                "✗ FAIL"
            }
        );
        info!(
            "🧪 Enemy Damage Test:  {}",
            if test_results.enemy_damage_test_passed {
                "✓ PASS"
            } else {
                "✗ FAIL"
            }
        );
        info!(
            "🧪 Restart Test:       {}",
            if test_results.restart_test_passed {
                "✓ PASS"
            } else {
                "✗ FAIL"
            }
        );
        info!("🧪 ============================================");

        if all_passed {
            info!("🎉 ALL TESTS PASSED! Game is working correctly!");
        } else {
            warn!("💥 SOME TESTS FAILED! Check the logs above for details.");
        }

        test_results.tests_completed = true;
    }
}

/// Helper function to spawn a test enemy for testing
pub fn spawn_test_enemy(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    position: Vec3,
) -> Entity {
    let enemy_material = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.0, 0.0),
        emissive: Color::srgb(0.3, 0.0, 0.0).into(),
        ..default()
    });

    commands
        .spawn((
            Mesh3d(meshes.add(Capsule3d::new(0.4, 1.6))),
            MeshMaterial3d(enemy_material),
            Transform::from_translation(position),
            RigidBody::Dynamic,
            Collider::capsule(0.4, 1.6),
            Mass(50.0),
            LockedAxes::ROTATION_LOCKED,
            Enemy {
                enemy_type: EnemyType::Striker,
                aggro_range: 7.0,
                attack_range: 1.5,
                move_speed: 1.0,
                attack_damage: 20.0,
                attack_cooldown: 1.0,
                last_attack_time: 0.0,
                state: EnemyState::Patrolling,
                target: None,
            },
            EnemyAI {
                detection_radius: 7.0,
                patrol_points: vec![position],
                current_patrol_index: 0,
                last_known_player_position: None,
                search_timer: 0.0,
                reaction_time: 0.5,
            },
            Health {
                current: 60.0,
                maximum: 60.0,
                regeneration_rate: 0.0,
                last_damage_time: 0.0,
            },
            TestMarker,
            Name::new("Test Enemy"),
        ))
        .id()
}

/// Manual test runner (call this from debug UI or key binding)
pub fn run_manual_tests(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    info!("🧪 Starting manual integration tests...");

    // Add test results resource
    commands.insert_resource(TestResults::default());

    // Spawn a test enemy in front of player
    spawn_test_enemy(
        &mut commands,
        &mut meshes,
        &mut materials,
        Vec3::new(2.0, 0.8, 0.0),
    );

    info!("🧪 Test setup complete. Watch the logs for test results...");
}
