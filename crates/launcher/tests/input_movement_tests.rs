mod common;

use bevy::prelude::*;
use shared::{
    input::{
        PlayerAction, FpsController, get_movement_direction, get_mouse_look_delta,
        update_look, MAX_SPEED, JUMP_HEIGHT, MOUSE_SENSITIVITY, PITCH_LIMIT_RADIANS,
        ROTATION_SMOOTHING_RATE, MOVEMENT_SPEED, FLOAT_HEIGHT,
    },
};
use leafwing_input_manager::prelude::ActionState;
use avian3d::prelude::*;

/// Test basic movement input processing
#[test]
fn test_movement_input_processing() {
    let mut action_state = ActionState::<PlayerAction>::default();
    
    // Test forward movement (W key)
    action_state.set_axis_pair(&PlayerAction::Move, Vec2::new(0.0, 1.0));
    let movement = get_movement_direction(&action_state);
    assert_eq!(movement, Vec2::new(0.0, 1.0), "Forward input should be processed correctly");
    
    // Test diagonal movement
    action_state.set_axis_pair(&PlayerAction::Move, Vec2::new(1.0, 1.0));
    let movement = get_movement_direction(&action_state);
    assert!((movement.length() - 1.0).abs() < 0.001, "Diagonal movement should be normalized to unit length, got: {}", movement.length());
    
    // Test no movement
    action_state.set_axis_pair(&PlayerAction::Move, Vec2::ZERO);
    let movement = get_movement_direction(&action_state);
    assert_eq!(movement, Vec2::ZERO, "No input should result in no movement");
}

/// Test look input processing and sensitivity
#[test]
fn test_look_input_processing() {
    let mut action_state = ActionState::<PlayerAction>::default();
    
    // Test mouse look input
    let mouse_delta = Vec2::new(0.5, -0.3);
    action_state.set_axis_pair(&PlayerAction::Look, mouse_delta);
    let look_delta = get_mouse_look_delta(&action_state);
    assert_eq!(look_delta, mouse_delta, "Mouse look input should be passed through");
    
    // Test very small input (should be filtered by deadzone)
    action_state.set_axis_pair(&PlayerAction::Look, Vec2::new(0.0001, 0.0001));
    let look_delta = get_mouse_look_delta(&action_state);
    assert_eq!(look_delta, Vec2::ZERO, "Very small look input should be filtered out");
}

/// Test FPS controller look update with pitch limits
#[test]
fn test_fps_controller_look_update() {
    let mut controller = FpsController::default();
    let mut action_state = ActionState::<PlayerAction>::default();
    
    // Test yaw update (horizontal look)
    action_state.set_axis_pair(&PlayerAction::Look, Vec2::new(1.0, 0.0));
    let initial_yaw = controller.yaw;
    update_look(&mut controller, &action_state);
    assert_ne!(controller.yaw, initial_yaw, "Yaw should change with horizontal input");
    
    // Test pitch limits (vertical look)
    controller.pitch = 0.0;
    action_state.set_axis_pair(&PlayerAction::Look, Vec2::new(0.0, -100.0)); // Large downward input
    update_look(&mut controller, &action_state);
    assert!(controller.pitch >= -PITCH_LIMIT_RADIANS, "Pitch should not exceed lower limit");
    assert!(controller.pitch <= PITCH_LIMIT_RADIANS, "Pitch should not exceed upper limit");
}

/// Test action state button handling
#[test]
fn test_action_state_buttons() {
    let mut action_state = ActionState::<PlayerAction>::default();
    
    // Test jump button
    assert!(!action_state.pressed(&PlayerAction::Jump), "Jump should not be pressed initially");
    action_state.press(&PlayerAction::Jump);
    assert!(action_state.pressed(&PlayerAction::Jump), "Jump should be pressed after press()");
    
    // Test sprint button
    action_state.press(&PlayerAction::Sprint);
    assert!(action_state.pressed(&PlayerAction::Sprint), "Sprint should be pressed");
    
    action_state.release(&PlayerAction::Sprint);
    assert!(!action_state.pressed(&PlayerAction::Sprint), "Sprint should not be pressed after release");
    
    // Test shooting
    action_state.press(&PlayerAction::Shoot);
    assert!(action_state.pressed(&PlayerAction::Shoot), "Shoot should be pressed");
    
    // Test aiming
    action_state.press(&PlayerAction::Aim);
    assert!(action_state.pressed(&PlayerAction::Aim), "Aim should be pressed");
}

/// Test multiple simultaneous inputs
#[test]
fn test_simultaneous_inputs() {
    let mut action_state = ActionState::<PlayerAction>::default();
    
    // Press multiple buttons simultaneously
    action_state.press(&PlayerAction::Jump);
    action_state.press(&PlayerAction::Sprint);
    action_state.set_axis_pair(&PlayerAction::Move, Vec2::new(1.0, 1.0));
    action_state.set_axis_pair(&PlayerAction::Look, Vec2::new(0.5, -0.2));
    
    // All should be active simultaneously
    assert!(action_state.pressed(&PlayerAction::Jump));
    assert!(action_state.pressed(&PlayerAction::Sprint));
    
    let movement = get_movement_direction(&action_state);
    assert_ne!(movement, Vec2::ZERO);
    
    let look_delta = get_mouse_look_delta(&action_state);
    assert_ne!(look_delta, Vec2::ZERO);
}

/// Test FPS controller physics integration setup
#[test]
fn test_fps_controller_physics_setup() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(AssetPlugin::default());
    app.add_plugins(PhysicsPlugins::default());
    
    // Spawn player entity with FPS controller and physics
    let player = app.world_mut().spawn((
        FpsController::default(),
        ActionState::<PlayerAction>::default(),
        Transform::from_xyz(0.0, 2.0, 0.0),
        LinearVelocity::ZERO,
        RigidBody::Dynamic,
        Collider::capsule(1.5, 0.5), // height, radius
        Mass(80.0),
    )).id();
    
    // Verify all required components are present
    assert!(app.world().get::<FpsController>(player).is_some(), "Should have FPS controller");
    assert!(app.world().get::<ActionState<PlayerAction>>(player).is_some(), "Should have action state");
    assert!(app.world().get::<Transform>(player).is_some(), "Should have transform");
    assert!(app.world().get::<LinearVelocity>(player).is_some(), "Should have velocity");
    assert!(app.world().get::<RigidBody>(player).is_some(), "Should have rigid body");
    assert!(app.world().get::<Collider>(player).is_some(), "Should have collider");
    assert!(app.world().get::<Mass>(player).is_some(), "Should have mass");
    
    // Check initial values
    let controller = app.world().get::<FpsController>(player).unwrap();
    assert_eq!(controller.pitch, 0.0);
    assert_eq!(controller.yaw, 0.0);
    assert_eq!(controller.ground_tick, 0);
    
    let velocity = app.world().get::<LinearVelocity>(player).unwrap();
    assert_eq!(velocity.0, Vec3::ZERO);
}

/// Test input clamping and normalization
#[test]
fn test_input_clamping_and_normalization() {
    let mut action_state = ActionState::<PlayerAction>::default();
    
    // Test extreme input values get clamped
    action_state.set_axis_pair(&PlayerAction::Move, Vec2::new(100.0, 100.0));
    let movement = get_movement_direction(&action_state);
    assert!(movement.length() <= 1.0 + f32::EPSILON, "Large input should be clamped to unit length");
    
    // Test normalized diagonal movement
    action_state.set_axis_pair(&PlayerAction::Move, Vec2::new(1.0, 1.0));
    let movement = get_movement_direction(&action_state);
    let expected_length = 1.0;
    assert!((movement.length() - expected_length).abs() < f32::EPSILON, "Diagonal input should be normalized");
}

/// Test movement constants are reasonable
#[test]
fn test_movement_constants() {
    // Verify movement constants are in reasonable ranges
    assert!(MAX_SPEED > 0.0, "Max speed should be positive");
    assert!(JUMP_HEIGHT > 0.0, "Jump height should be positive");
    assert!(MOUSE_SENSITIVITY > 0.0, "Mouse sensitivity should be positive");
    assert!(PITCH_LIMIT_RADIANS > 0.0, "Pitch limit should be positive");
    assert!(PITCH_LIMIT_RADIANS < std::f32::consts::PI, "Pitch limit should be less than 180 degrees");
    assert!(ROTATION_SMOOTHING_RATE > 0.0, "Rotation smoothing should be positive");
    assert!(MOVEMENT_SPEED > 0.0, "Movement speed should be positive");
    assert!(FLOAT_HEIGHT > 0.0, "Float height should be positive");
}

/// Test FPS controller ground detection state
#[test]
fn test_fps_controller_ground_detection() {
    let mut controller = FpsController::default();
    
    // Initial state should be not grounded
    assert_eq!(controller.ground_tick, 0, "Should start not grounded");
    
    // Simulate ground contact
    controller.ground_tick = 1;
    assert_eq!(controller.ground_tick, 1, "Should register ground contact");
    
    // Test grounded distance threshold
    assert!(controller.grounded_distance > 0.0, "Grounded distance should be positive");
    assert_eq!(controller.grounded_distance, 0.3, "Default grounded distance should be 0.3");
}

/// Test action state dual axis handling
#[test]
fn test_dual_axis_action_handling() {
    let mut action_state = ActionState::<PlayerAction>::default();
    
    // Test that Move and Look are properly configured as dual axis actions
    // Set movement input
    action_state.set_axis_pair(&PlayerAction::Move, Vec2::new(0.7, -0.5));
    let move_input = action_state.axis_pair(&PlayerAction::Move);
    assert_eq!(move_input.x, 0.7);
    assert_eq!(move_input.y, -0.5);
    
    // Set look input
    action_state.set_axis_pair(&PlayerAction::Look, Vec2::new(-0.3, 0.8));
    let look_input = action_state.axis_pair(&PlayerAction::Look);
    assert_eq!(look_input.x, -0.3);
    assert_eq!(look_input.y, 0.8);
    
    // Verify they're independent
    let move_after_look = action_state.axis_pair(&PlayerAction::Move);
    assert_eq!(move_after_look, Vec2::new(0.7, -0.5)); // Should be unchanged
}

/// Test FPS controller sensitivity settings
#[test]
fn test_fps_controller_sensitivity() {
    let mut controller = FpsController::default();
    let mut action_state = ActionState::<PlayerAction>::default();
    
    // Test default sensitivity
    assert_eq!(controller.sensitivity, 0.001, "Default sensitivity should be 0.001");
    
    // Test sensitivity affects yaw change
    let look_input = Vec2::new(1.0, 0.0);
    action_state.set_axis_pair(&PlayerAction::Look, look_input);
    
    let initial_yaw = controller.yaw;
    update_look(&mut controller, &action_state);
    let yaw_change = (controller.yaw - initial_yaw).abs();
    
    // Change sensitivity and test again
    controller.yaw = initial_yaw; // Reset
    controller.sensitivity = 0.002; // Double sensitivity
    update_look(&mut controller, &action_state);
    let double_sensitivity_change = (controller.yaw - initial_yaw).abs();
    
    // Double sensitivity should result in approximately double the change
    assert!(double_sensitivity_change > yaw_change, "Higher sensitivity should cause larger yaw changes");
}

/// Test physics parameter validation
#[test]
fn test_physics_parameters_validation() {
    let controller = FpsController::default();
    
    // Verify all physics parameters are positive and reasonable
    assert!(controller.gravity > 0.0, "Gravity should be positive");
    assert!(controller.walk_speed > 0.0, "Walk speed should be positive");
    assert!(controller.run_speed > controller.walk_speed, "Run speed should be faster than walk speed");
    assert!(controller.jump_speed > 0.0, "Jump speed should be positive");
    assert!(controller.acceleration > 0.0, "Acceleration should be positive");
    assert!(controller.friction > 0.0, "Friction should be positive");
    assert!(controller.air_acceleration > 0.0, "Air acceleration should be positive");
    assert!(controller.max_air_speed > 0.0, "Max air speed should be positive");
    assert!(controller.stop_speed >= 0.0, "Stop speed should be non-negative");
    
    // Verify friction and traction parameters
    assert!(controller.traction_normal_cutoff > 0.0, "Traction normal cutoff should be positive");
    assert!(controller.traction_normal_cutoff <= 1.0, "Traction normal cutoff should not exceed 1.0");
    assert!(controller.friction_speed_cutoff >= 0.0, "Friction speed cutoff should be non-negative");
}