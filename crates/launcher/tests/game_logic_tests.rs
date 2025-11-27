mod common;

use bevy::prelude::*;
use bevy::asset::AssetEvent;
use common::*;
use shared::{
    input::{PlayerAction, FpsController, get_movement_direction, get_mouse_look_delta},
    entities::{PlayerPhysicsBundle, NpcPhysicsBundle, color_from_id},
};
use leafwing_input_manager::prelude::ActionState;
use avian3d::prelude::*;

/// Test FPS controller input processing
#[test]
fn test_fps_controller_input_processing() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    // Create an action state for testing
    let mut action_state = ActionState::<PlayerAction>::default();
    
    // Test movement input
    action_state.set_axis_pair(&PlayerAction::Move, Vec2::new(1.0, 0.0)); // Move right
    let movement = get_movement_direction(&action_state);
    assert_eq!(movement, Vec2::new(1.0, 0.0), "Should detect rightward movement");

    // Test look input
    action_state.set_axis_pair(&PlayerAction::Look, Vec2::new(0.1, -0.1)); // Look right and up
    let look_delta = get_mouse_look_delta(&action_state);
    assert_eq!(look_delta, Vec2::new(0.1, -0.1), "Should detect look input");

    // Test button input
    action_state.press(&PlayerAction::Jump);
    assert!(action_state.pressed(&PlayerAction::Jump), "Jump should be pressed");
}

/// Test FPS controller initialization and default values
#[test]
fn test_fps_controller_defaults() {
    let controller = FpsController::default();
    
    assert_eq!(controller.gravity, 9.1);
    assert_eq!(controller.walk_speed, 100.0);
    assert_eq!(controller.run_speed, 150.0);
    assert_eq!(controller.jump_speed, 8.5);
    assert_eq!(controller.pitch, 0.0);
    assert_eq!(controller.yaw, 0.0);
    assert_eq!(controller.ground_tick, 0);
}

/// Test player physics bundle configuration
#[test]
fn test_player_physics_bundle() {
    let bundle = PlayerPhysicsBundle::default();
    
    // Verify physics properties are reasonable for player
    assert_eq!(bundle.mass.0, 80.0, "Player should have realistic mass");
    assert_eq!(bundle.restitution, Restitution::ZERO, "Player shouldn't be bouncy");
    assert_eq!(bundle.friction.dynamic_coefficient, 0.5, "Player should have moderate friction");
    
    // Verify proper body type for player
    match bundle.rigid_body {
        RigidBody::Dynamic => {},
        _ => panic!("Player should have dynamic rigid body"),
    }
}

/// Test NPC physics bundle configuration
#[test]
fn test_npc_physics_bundle() {
    let bundle = NpcPhysicsBundle::default();
    
    // Verify NPC physics properties differ from player where appropriate
    assert_eq!(bundle.mass.0, 70.0, "NPC should be lighter than player");
    assert_eq!(bundle.friction.dynamic_coefficient, 0.2, "NPC should have different friction");
    
    // Verify proper body type for NPC
    match bundle.rigid_body {
        RigidBody::Dynamic => {},
        _ => panic!("NPC should have dynamic rigid body"),
    }
}

/// Test color generation from ID (used for player identification)
#[test]
fn test_color_from_id_generation() {
    let color1 = color_from_id(1);
    let color2 = color_from_id(2);
    let color3 = color_from_id(1); // Same ID should produce same color
    
    // Different IDs should produce different colors
    assert_ne!(color1, color2, "Different IDs should produce different colors");
    
    // Same ID should produce same color
    assert_eq!(color1, color3, "Same ID should produce same color");
    
    // Colors should be in valid range (assuming HSL with saturation 0.8, lightness 0.6)
    // We can't test the exact values due to HSL to RGB conversion, but we can verify they're valid
    assert!((0.0..=1.0).contains(&color1.alpha()), "Color alpha should be in valid range");
}

/// Test input deadzone handling
#[test]
fn test_input_deadzone_handling() {
    let mut action_state = ActionState::<PlayerAction>::default();
    
    // Test very small input (should be filtered out by deadzone)
    action_state.set_axis_pair(&PlayerAction::Move, Vec2::new(0.0001, 0.0001));
    let movement = get_movement_direction(&action_state);
    assert_eq!(movement, Vec2::ZERO, "Tiny inputs should be filtered by deadzone");

    // Test input just above deadzone
    action_state.set_axis_pair(&PlayerAction::Move, Vec2::new(0.1, 0.1));
    let movement = get_movement_direction(&action_state);
    assert_ne!(movement, Vec2::ZERO, "Inputs above deadzone should be detected");

    // Test input clamping (values > 1.0 should be clamped to 1.0)
    action_state.set_axis_pair(&PlayerAction::Move, Vec2::new(2.0, 2.0));
    let movement = get_movement_direction(&action_state);
    assert!(movement.length() <= 1.0 + f32::EPSILON, "Input should be clamped to maximum length of 1.0");
}

/// Test entity spawning with game components
#[test]
fn test_entity_spawning_with_components() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    // Spawn a player entity with physics
    let player_entity = app.world_mut().spawn((
        PlayerPhysicsBundle::default(),
        FpsController::default(),
        ActionState::<PlayerAction>::default(),
        Transform::from_xyz(0.0, 1.0, 0.0),
    )).id();

    // Spawn an NPC entity
    let npc_entity = app.world_mut().spawn((
        NpcPhysicsBundle::default(),
        Transform::from_xyz(5.0, 1.0, 0.0),
    )).id();

    // Verify entities exist and have expected components
    assert!(app.world().get::<FpsController>(player_entity).is_some(), "Player should have FPS controller");
    assert!(app.world().get::<ActionState<PlayerAction>>(player_entity).is_some(), "Player should have action state");
    assert!(app.world().get::<Transform>(player_entity).is_some(), "Player should have transform");
    assert!(app.world().get::<Mass>(player_entity).is_some(), "Player should have mass component");
    
    assert!(app.world().get::<Transform>(npc_entity).is_some(), "NPC should have transform");
    assert!(app.world().get::<Mass>(npc_entity).is_some(), "NPC should have mass component");
    assert!(app.world().get::<FpsController>(npc_entity).is_none(), "NPC should not have FPS controller");
}

/// Test physics components setup without full simulation
#[test]
fn test_physics_components_setup() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    // Spawn entity with physics bundle components individually
    let entity = app.world_mut().spawn((
        RigidBody::Dynamic,
        Collider::capsule(1.5, 0.5),
        Mass(80.0),
        Transform::from_xyz(0.0, 10.0, 0.0),
        LinearVelocity::ZERO,
    )).id();

    // Verify all physics components are present
    let entity_exists = app.world().get::<Transform>(entity).is_some();
    assert!(entity_exists, "Entity should exist");
    
    // Verify physics components are present
    assert!(app.world().get::<RigidBody>(entity).is_some(), "Entity should have rigid body component");
    assert!(app.world().get::<Collider>(entity).is_some(), "Entity should have collider component");
    assert!(app.world().get::<LinearVelocity>(entity).is_some(), "Entity should have velocity component");
    assert!(app.world().get::<Mass>(entity).is_some(), "Entity should have mass component");
    
    // Test component values
    let mass = app.world().get::<Mass>(entity).unwrap();
    assert_eq!(mass.0, 80.0, "Mass should be 80.0");
    
    let velocity = app.world().get::<LinearVelocity>(entity).unwrap();
    assert_eq!(velocity.0, Vec3::ZERO, "Velocity should start at zero");
}

/// Test multiple entities with different physics properties
#[test]
fn test_multiple_entities_physics_properties() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    // Spawn multiple entities with different masses
    let heavy_entity = app.world_mut().spawn((
        RigidBody::Dynamic,
        Collider::sphere(1.0),
        Mass(100.0),
        Transform::from_xyz(0.0, 5.0, 0.0),
    )).id();

    let light_entity = app.world_mut().spawn((
        RigidBody::Dynamic,
        Collider::sphere(1.0),
        Mass(10.0),
        Transform::from_xyz(2.0, 5.0, 0.0),
    )).id();

    // Verify different masses
    let heavy_mass = app.world().get::<Mass>(heavy_entity).unwrap().0;
    let light_mass = app.world().get::<Mass>(light_entity).unwrap().0;
    
    assert_eq!(heavy_mass, 100.0, "Heavy entity should have expected mass");
    assert_eq!(light_mass, 10.0, "Light entity should have expected mass");
    assert_ne!(heavy_mass, light_mass, "Entities should have different masses");

    // Verify different positions
    let heavy_pos = app.world().get::<Transform>(heavy_entity).unwrap();
    let light_pos = app.world().get::<Transform>(light_entity).unwrap();
    
    assert_ne!(heavy_pos.translation, light_pos.translation, "Entities should have different positions");

    // Both entities should exist with all components
    assert!(app.world().get::<Transform>(heavy_entity).is_some(), "Heavy entity should exist");
    assert!(app.world().get::<Transform>(light_entity).is_some(), "Light entity should exist");
    assert!(app.world().get::<RigidBody>(heavy_entity).is_some(), "Heavy entity should have rigid body");
    assert!(app.world().get::<RigidBody>(light_entity).is_some(), "Light entity should have rigid body");
}