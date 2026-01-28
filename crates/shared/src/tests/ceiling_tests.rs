//! Tests for ceiling implementation in the static level

use avian3d::prelude::*;

#[test]
fn test_ceiling_constants() {
    // Verify ceiling-related constants are defined
    use crate::create_static_level::*;
    
    assert_eq!(WALL_HEIGHT, 10.0, "Wall height should be 10.0");
    assert_eq!(FLOOR_THICKNESS, 1.0, "Floor thickness should be 1.0");
}

#[test]
fn test_collider_creation() {
    // Test that colliders can be created with expected shapes
    let _floor_collider = Collider::cuboid(10.0, 0.1, 10.0);
    let _ceiling_collider = Collider::cuboid(10.0, 0.1, 10.0);
    let _wall_collider = Collider::cuboid(0.1, 2.0, 10.0);
}
