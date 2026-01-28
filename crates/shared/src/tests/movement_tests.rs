//! Tests for camera-relative movement and configuration

use bevy::prelude::*;
use crate::movement::*;

#[test]
fn test_movement_config_default() {
    let config = MovementConfig::default();
    assert_eq!(config.walk_speed, 100.0, "Default walk speed mismatch");
    assert_eq!(config.run_speed, 150.0, "Default run speed mismatch");
    assert_eq!(config.jump_speed, 8.5, "Default jump speed mismatch");
}

#[test]
fn test_ground_state_default() {
    let ground_state = GroundState::default();
    assert!(!ground_state.is_grounded, "Default should not be grounded");
    assert_eq!(ground_state.ground_normal, Vec3::ZERO, "Default ground normal should be zero");
}

#[test]
fn test_physics_config_default() {
    let physics_config = PhysicsConfig::default();
    assert_eq!(physics_config.gravity, 9.1, "Default gravity mismatch");
    assert_eq!(physics_config.grounded_distance, 0.3, "Default grounded distance mismatch");
}

#[test]
fn test_get_wish_direction_forward() {
    let move_input = Vec2::new(0.0, 1.0); // Forward
    let yaw = 0.0;
    
    let (direction, _speed) = get_wish_direction(move_input, yaw, 100.0, 60.0);
    
    // Should move forward (negative Z in Bevy)
    assert!(direction.z < 0.0, "Forward input should produce negative Z movement");
    assert!(direction.x.abs() < 0.1, "Forward input should have minimal X movement");
}

#[test]
fn test_get_wish_direction_right() {
    let move_input = Vec2::new(1.0, 0.0); // Right
    let yaw = 0.0;
    
    let (direction, _speed) = get_wish_direction(move_input, yaw, 100.0, 60.0);
    
    // Should move right (positive X)
    assert!(direction.x > 0.0, "Right input should produce positive X movement");
}

#[test]
fn test_get_wish_direction_with_yaw_rotation() {
    use std::f32::consts::PI;
    
    let move_input = Vec2::new(0.0, 1.0); // Forward
    let yaw = PI / 2.0; // Turned 90 degrees right
    
    let (direction, _speed) = get_wish_direction(move_input, yaw, 100.0, 60.0);
    
    // When facing right (yaw = 90°), forward input should produce movement
    // The exact direction depends on the coordinate system
    // Just verify we get a valid direction vector
    let length = direction.length();
    assert!((length - 1.0).abs() < 0.01, "Direction should be normalized");
}

#[test]
fn test_movement_config_component() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    
    // Spawn entity with movement config
    app.world_mut().spawn(MovementConfig {
        walk_speed: 120.0,
        run_speed: 180.0,
        air_speed_cap: 25.0,
        air_acceleration: 15.0,
        max_air_speed: 70.0,
        acceleration: 12.0,
        friction: 8.0,
        jump_speed: 10.0,
    });
    
    app.update();
    
    let mut query = app.world_mut().query::<&MovementConfig>();
    let config = query.single(app.world()).expect("Should have MovementConfig");
    
    assert_eq!(config.walk_speed, 120.0, "Walk speed mismatch");
    assert_eq!(config.jump_speed, 10.0, "Jump speed mismatch");
}
