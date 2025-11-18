#[cfg(test)]
mod input_system_tests {
    use crate::input::*;
    use avian3d::prelude::*;
    use bevy::prelude::*;
    use leafwing_input_manager::prelude::ActionState;

    /// Test complex player movement with deadzone handling
    #[test]
    fn test_movement_deadzone_and_normalization() {
        let mut action_state = ActionState::<PlayerAction>::default();
        let mut rotation = Rotation::default();
        let mut velocity = LinearVelocity::default();

        // Test input below deadzone threshold
        action_state.set_axis_pair(&PlayerAction::Move, Vec2::new(0.0005, 0.0005));
        shared_player_movement(&action_state, &mut rotation, &mut velocity);

        assert_eq!(
            velocity.0.x, 0.0,
            "Velocity should be zero for input below deadzone"
        );
        assert_eq!(
            velocity.0.z, 0.0,
            "Velocity should be zero for input below deadzone"
        );

        // Test input above deadzone
        action_state.set_axis_pair(&PlayerAction::Move, Vec2::new(0.5, 0.5));
        shared_player_movement(&action_state, &mut rotation, &mut velocity);

        assert_ne!(
            velocity.0.x, 0.0,
            "Velocity should not be zero for input above deadzone"
        );
        assert_ne!(
            velocity.0.z, 0.0,
            "Velocity should not be zero for input above deadzone"
        );

        // Test input normalization (diagonal movement should be normalized)
        action_state.set_axis_pair(&PlayerAction::Move, Vec2::new(1.0, 1.0));
        shared_player_movement(&action_state, &mut rotation, &mut velocity);

        let _velocity_magnitude = Vec2::new(velocity.0.x, velocity.0.z).length();
        assert!(
            (velocity.0.length() - MAX_SPEED).abs() < 0.1,
            "Diagonal movement should be normalized to max speed"
        );
    }

    /// Test mouse look sensitivity and rotation behavior
    #[test]
    fn test_mouse_look_rotation_behavior() {
        let mut action_state = ActionState::<PlayerAction>::default();
        let mut rotation = Rotation::default();
        let mut velocity = LinearVelocity::default();

        // Store initial rotation
        let initial_rotation = rotation.0;

        // Test small mouse movement (below deadzone)
        action_state.set_axis_pair(&PlayerAction::Look, Vec2::new(0.0005, 0.0005));
        shared_player_movement(&action_state, &mut rotation, &mut velocity);

        assert_eq!(
            rotation.0, initial_rotation,
            "Rotation should not change for input below deadzone"
        );

        // Test horizontal mouse movement (yaw)
        action_state.set_axis_pair(&PlayerAction::Look, Vec2::new(100.0, 0.0));
        shared_player_movement(&action_state, &mut rotation, &mut velocity);

        assert_ne!(
            rotation.0, initial_rotation,
            "Rotation should change for horizontal mouse movement"
        );

        // Verify rotation is normalized
        assert!(
            (rotation.0.length() - 1.0).abs() < 0.001,
            "Rotation quaternion should be normalized"
        );

        // Test that Y rotation affects movement direction
        rotation.0 = Quat::from_rotation_y(std::f32::consts::PI / 2.0); // 90 degrees
        action_state.set_axis_pair(&PlayerAction::Move, Vec2::new(0.0, 1.0)); // Forward
        shared_player_movement(&action_state, &mut rotation, &mut velocity);

        // After 90-degree rotation, forward movement should become sideways movement
        assert!(
            velocity.0.x.abs() > velocity.0.z.abs(),
            "Forward movement should become sideways after 90-degree rotation"
        );
    }

    /// Test complex input combinations and edge cases
    #[test]
    fn test_complex_input_combinations() {
        let mut action_state = ActionState::<PlayerAction>::default();
        let mut rotation = Rotation::default();
        let mut velocity = LinearVelocity::default();

        // Test simultaneous movement and look
        action_state.set_axis_pair(&PlayerAction::Move, Vec2::new(0.8, 0.6));
        action_state.set_axis_pair(&PlayerAction::Look, Vec2::new(50.0, 0.0));

        let initial_y_velocity = velocity.0.y;
        shared_player_movement(&action_state, &mut rotation, &mut velocity);

        // Movement should be applied
        assert_ne!(
            velocity.0.x, 0.0,
            "X velocity should be affected by movement"
        );
        assert_ne!(
            velocity.0.z, 0.0,
            "Z velocity should be affected by movement"
        );

        // Y velocity should be preserved (no vertical movement from horizontal input)
        assert_eq!(
            velocity.0.y, initial_y_velocity,
            "Y velocity should not be affected by horizontal movement"
        );

        // Rotation should be applied
        assert_ne!(
            rotation.0,
            Quat::IDENTITY,
            "Rotation should be affected by look input"
        );
    }

    /// Test movement direction relative to player rotation
    #[test]
    fn test_movement_relative_to_rotation() {
        let mut action_state = ActionState::<PlayerAction>::default();
        let mut rotation = Rotation::default();
        let mut velocity = LinearVelocity::default();

        // Test forward movement with no rotation
        action_state.set_axis_pair(&PlayerAction::Move, Vec2::new(0.0, 1.0));
        shared_player_movement(&action_state, &mut rotation, &mut velocity);

        let forward_velocity = velocity.0;
        assert!(
            forward_velocity.z < 0.0,
            "Forward movement should be negative Z"
        );
        assert!(
            forward_velocity.x.abs() < 0.01,
            "Forward movement should not affect X"
        );

        // Reset velocity and set 180-degree rotation
        velocity.0 = Vec3::ZERO;
        rotation.0 = Quat::from_rotation_y(std::f32::consts::PI);

        // Forward movement with 180-degree rotation should reverse direction
        shared_player_movement(&action_state, &mut rotation, &mut velocity);

        assert!(
            velocity.0.z > 0.0,
            "Forward movement with 180-degree rotation should be positive Z"
        );
        assert!(velocity.0.x.abs() < 0.01, "Should still not affect X");

        // Test strafe movement
        velocity.0 = Vec3::ZERO;
        rotation.0 = Quat::IDENTITY;
        action_state.set_axis_pair(&PlayerAction::Move, Vec2::new(1.0, 0.0));

        shared_player_movement(&action_state, &mut rotation, &mut velocity);

        assert!(velocity.0.x > 0.0, "Right strafe should be positive X");
        assert!(velocity.0.z.abs() < 0.01, "Strafe should not affect Z");
    }

    /// Test input clamping and boundary conditions
    #[test]
    fn test_input_clamping_and_boundaries() {
        let mut action_state = ActionState::<PlayerAction>::default();
        let mut rotation = Rotation::default();
        let mut velocity = LinearVelocity::default();

        // Test extreme input values (should be clamped)
        action_state.set_axis_pair(&PlayerAction::Move, Vec2::new(100.0, 100.0));
        shared_player_movement(&action_state, &mut rotation, &mut velocity);

        let velocity_magnitude = Vec2::new(velocity.0.x, velocity.0.z).length();
        assert!(
            (velocity_magnitude - MAX_SPEED).abs() < 0.01,
            "Extreme input should be clamped to max speed"
        );

        // Test zero input
        action_state.set_axis_pair(&PlayerAction::Move, Vec2::ZERO);
        shared_player_movement(&action_state, &mut rotation, &mut velocity);

        assert_eq!(
            velocity.0.x, 0.0,
            "Zero input should result in zero velocity"
        );
        assert_eq!(
            velocity.0.z, 0.0,
            "Zero input should result in zero velocity"
        );

        // Test negative input values
        action_state.set_axis_pair(&PlayerAction::Move, Vec2::new(-1.0, -1.0));
        shared_player_movement(&action_state, &mut rotation, &mut velocity);

        assert!(
            velocity.0.x < 0.0,
            "Negative X input should result in negative X velocity"
        );
        assert!(
            velocity.0.z > 0.0,
            "Negative Y input should result in positive Z velocity (forward is -Z)"
        );
    }

    /// Test rotation smoothing and accumulation
    #[test]
    fn test_rotation_accumulation() {
        let mut action_state = ActionState::<PlayerAction>::default();
        let mut rotation = Rotation::default();
        let mut velocity = LinearVelocity::default();

        // Apply multiple small rotations
        let small_rotation = 10.0;
        let mut accumulated_rotation = Quat::IDENTITY;

        for _ in 0..10 {
            action_state.set_axis_pair(&PlayerAction::Look, Vec2::new(small_rotation, 0.0));
            shared_player_movement(&action_state, &mut rotation, &mut velocity);

            // Rotation should accumulate
            assert_ne!(
                rotation.0, accumulated_rotation,
                "Rotation should change with each input"
            );
            accumulated_rotation = rotation.0;
        }

        // Total rotation should be significant
        let (_, yaw, _): (f32, f32, f32) = rotation.0.to_euler(EulerRot::YXZ);
        assert!(
            yaw.abs() > 0.1,
            "Multiple small rotations should accumulate to significant rotation"
        );

        // Test that rotation affects subsequent movement
        action_state.set_axis_pair(&PlayerAction::Look, Vec2::ZERO);
        action_state.set_axis_pair(&PlayerAction::Move, Vec2::new(0.0, 1.0));

        shared_player_movement(&action_state, &mut rotation, &mut velocity);

        // Movement direction should be affected by accumulated rotation
        assert_ne!(
            velocity.0.x, 0.0,
            "Accumulated rotation should affect movement direction"
        );
    }

    /// Test edge cases with very small and very large inputs
    #[test]
    fn test_input_edge_cases() {
        let mut action_state = ActionState::<PlayerAction>::default();
        let mut rotation = Rotation::default();
        let mut velocity = LinearVelocity::default();

        // Test extremely small input (should be filtered by deadzone)
        action_state.set_axis_pair(&PlayerAction::Move, Vec2::new(0.0001, 0.0001));
        action_state.set_axis_pair(&PlayerAction::Look, Vec2::new(0.0001, 0.0001));

        shared_player_movement(&action_state, &mut rotation, &mut velocity);

        assert_eq!(
            velocity.0.x, 0.0,
            "Extremely small input should be filtered"
        );
        assert_eq!(
            velocity.0.z, 0.0,
            "Extremely small input should be filtered"
        );
        assert_eq!(
            rotation.0,
            Quat::IDENTITY,
            "Extremely small look input should be filtered"
        );

        // Test NaN and infinite inputs (should be handled gracefully)
        action_state.set_axis_pair(&PlayerAction::Move, Vec2::new(f32::NAN, f32::INFINITY));
        shared_player_movement(&action_state, &mut rotation, &mut velocity);

        // Velocity should remain finite
        assert!(
            velocity.0.x.is_finite(),
            "Velocity should remain finite with NaN input"
        );
        assert!(
            velocity.0.z.is_finite(),
            "Velocity should remain finite with infinite input"
        );
        assert!(rotation.0.is_finite(), "Rotation should remain finite");
    }
}
