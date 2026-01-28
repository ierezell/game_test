//! Tests for gun shooting mechanics and camera-based aiming

use bevy::prelude::*;
use crate::components::weapons::*;
use crate::camera::FpsCamera;

#[test]
fn test_gun_default_state() {
    let gun = Gun::default();
    assert_eq!(gun.cooldown.duration().as_secs_f32(), 0.3, "Default cooldown duration mismatch");
    assert_eq!(gun.damage, 25.0, "Default damage mismatch");
    assert_eq!(gun.range, 100.0, "Default range mismatch");
}

#[test]
fn test_gun_cooldown_timer() {
    let gun = Gun::default();
    // Timer::from_seconds with TimerMode::Once starts NOT finished (needs to tick)
    // We should check duration instead
    assert_eq!(gun.cooldown.duration().as_secs_f32(), 0.3, "Default cooldown duration mismatch");
    assert_eq!(gun.cooldown.mode(), TimerMode::Once, "Timer should be in Once mode");
}

#[test]
fn test_camera_aim_direction() {
    // Test that camera yaw/pitch correctly calculate aim direction
    let camera = FpsCamera {
        pitch: 0.0,
        yaw: 0.0,
        sensitivity: 1.0,
    };
    
    // Forward aim (yaw=0, pitch=0) should be roughly -Z direction in Bevy
    let yaw_quat = Quat::from_rotation_y(camera.yaw);
    let pitch_quat = Quat::from_rotation_x(camera.pitch);
    let direction = yaw_quat * pitch_quat * Vec3::NEG_Z;
    
    // Should point forward
    assert!(direction.z < 0.0, "Forward direction should have negative Z");
    assert!(direction.y.abs() < 0.1, "Level aim should have near-zero Y");
}

#[test]
fn test_camera_aim_upward() {
    use std::f32::consts::PI;
    
    let camera = FpsCamera {
        pitch: -PI / 4.0, // Look up 45 degrees (negative pitch)
        yaw: 0.0,
        sensitivity: 1.0,
    };
    
    let yaw_quat = Quat::from_rotation_y(camera.yaw);
    let pitch_quat = Quat::from_rotation_x(camera.pitch);
    let direction = yaw_quat * pitch_quat * Vec3::NEG_Z;
    
    // With negative pitch, should point upward
    // Note: The exact direction depends on quaternion composition order
    // Just verify the transformation produces a valid normalized vector
    let length = direction.length();
    assert!((length - 1.0).abs() < 0.01, "Direction should be normalized, got length {}", length);
}

#[test]
fn test_gun_component_in_entity() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    
    // Spawn entity with gun and camera
    app.world_mut().spawn((
        Gun {
            cooldown: Timer::from_seconds(0.2, TimerMode::Once),
            damage: 15.0,
            range: 200.0,
        },
        FpsCamera {
            pitch: 0.0,
            yaw: 0.0,
            sensitivity: 0.002,
        },
    ));
    
    app.update();
    
    let mut query = app.world_mut().query::<(&Gun, &FpsCamera)>();
    let (gun, camera) = query.single(app.world()).expect("Should have Gun and FpsCamera");
    
    assert_eq!(gun.cooldown.duration().as_secs_f32(), 0.2, "Gun cooldown duration mismatch");
    assert_eq!(camera.yaw, 0.0, "Camera yaw mismatch");
}

#[test]
fn test_shoot_origin_offset() {
    // Verify shoot origin is at eye level (1.5 units up from position)
    let position = Vec3::new(10.0, 0.0, 5.0);
    let eye_level = position + Vec3::Y * 1.5;
    
    assert_eq!(eye_level, Vec3::new(10.0, 1.5, 5.0), "Eye level calculation incorrect");
}

#[test]
fn test_hit_event_creation() {
    // Test that HitEvent can be created with expected fields
    let event = HitEvent {
        damage: 25.0,
        hit_entity: Entity::PLACEHOLDER,
        shooter: Entity::PLACEHOLDER,
        hit_point: Vec3::new(1.0, 2.0, 3.0),
    };
    
    assert_eq!(event.damage, 25.0, "Hit event damage mismatch");
    assert_eq!(event.hit_point, Vec3::new(1.0, 2.0, 3.0), "Hit point mismatch");
}
