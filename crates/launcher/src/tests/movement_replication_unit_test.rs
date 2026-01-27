#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
pub mod test {
    use avian3d::prelude::{Position, Rotation};
    use bevy::prelude::*;
    use leafwing_input_manager::prelude::ActionState;
    use shared::input::PlayerAction;
    use shared::movement::{MovementConfig, GroundState};
    use shared::camera::FpsCamera;

    /// Test that MovementConfig properly replicates (has required traits)
    #[test]
    fn test_movement_config_has_replication_traits() {
        let config1 = MovementConfig {
            walk_speed: 50.0,
            run_speed: 100.0,
            ..Default::default()
        };
        
        // Clone trait required for replication
        let config2 = config1.clone();
        assert_eq!(config1.walk_speed, config2.walk_speed);
        assert_eq!(config1.run_speed, config2.run_speed);

        // PartialEq required for change detection
        assert!(config1 == config2);

        // Debug for diagnostics
        let debug_str = format!("{:?}", config1);
        assert!(debug_str.contains("walk_speed"));
        assert!(debug_str.contains("run_speed"));

        println!("✅ MovementConfig has all required traits for Lightyear replication");
    }

    /// Test that FpsCamera properly replicates (has required traits)
    #[test]
    fn test_fps_camera_has_replication_traits() {
        let camera1 = FpsCamera {
            pitch: 0.5,
            yaw: 1.0,
            ..Default::default()
        };
        
        // Clone trait required for replication
        let camera2 = camera1.clone();
        assert_eq!(camera1.pitch, camera2.pitch);
        assert_eq!(camera1.yaw, camera2.yaw);

        // PartialEq required for change detection
        assert!(camera1 == camera2);

        // Debug for diagnostics
        let debug_str = format!("{:?}", camera1);
        assert!(debug_str.contains("pitch"));
        assert!(debug_str.contains("yaw"));

        println!("✅ FpsCamera has all required traits for Lightyear replication");
    }

    /// Test that GroundState properly replicates (has required traits)
    #[test]
    fn test_ground_state_has_replication_traits() {
        let ground1 = GroundState {
            is_grounded: true,
            ground_tick: 5,
            ..Default::default()
        };
        
        // Clone trait required for replication
        let ground2 = ground1.clone();
        assert_eq!(ground1.is_grounded, ground2.is_grounded);
        assert_eq!(ground1.ground_tick, ground2.ground_tick);

        // PartialEq required for change detection
        assert!(ground1 == ground2);

        // Debug for diagnostics
        let debug_str = format!("{:?}", ground1);
        assert!(debug_str.contains("is_grounded"));
        assert!(debug_str.contains("ground_tick"));

        println!("✅ GroundState has all required traits for Lightyear replication");
    }

    /// Test that Position component updates correctly
    #[test]
    fn test_position_component_updates() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let entity = app.world_mut().spawn(Position(Vec3::ZERO)).id();

        // Verify initial position
        let pos1 = app.world().get::<Position>(entity).unwrap().0;
        assert_eq!(pos1, Vec3::ZERO);

        // Update position (simulating movement)
        app.world_mut().entity_mut(entity).insert(Position(Vec3::new(1.0, 0.0, 0.0)));

        // Verify position changed
        let pos2 = app.world().get::<Position>(entity).unwrap().0;
        assert_eq!(pos2.x, 1.0);
        assert_ne!(pos1, pos2);

        println!("✅ Position component updates correctly for replication");
    }

    /// Test that Rotation component updates correctly
    #[test]
    fn test_rotation_component_updates() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let entity = app.world_mut().spawn(Rotation(Quat::IDENTITY)).id();

        // Verify initial rotation
        let rot1 = app.world().get::<Rotation>(entity).unwrap().0;
        assert!(rot1.is_near_identity());

        // Update rotation (simulating turning)
        let new_rot = Quat::from_rotation_y(std::f32::consts::FRAC_PI_2);
        app.world_mut().entity_mut(entity).insert(Rotation(new_rot));

        // Verify rotation changed
        let rot2 = app.world().get::<Rotation>(entity).unwrap().0;
        assert!(!rot2.is_near_identity());
        let angle = rot1.angle_between(rot2);
        assert!((angle - std::f32::consts::FRAC_PI_2).abs() < 0.01);

        println!("✅ Rotation component updates correctly for replication");
    }

    /// Test that ActionState properly sets movement input
    #[test]
    fn test_action_state_movement_input() {
        let mut action_state = ActionState::<PlayerAction>::default();

        // Set forward movement
        action_state.set_axis_pair(&PlayerAction::Move, Vec2::new(0.0, 1.0));
        
        let movement = action_state.axis_pair(&PlayerAction::Move);
        assert_eq!(movement.y, 1.0);

        // Set strafe movement
        action_state.set_axis_pair(&PlayerAction::Move, Vec2::new(1.0, 0.0));
        
        let movement = action_state.axis_pair(&PlayerAction::Move);
        assert_eq!(movement.x, 1.0);

        // Stop movement
        action_state.set_axis_pair(&PlayerAction::Move, Vec2::ZERO);
        
        let movement = action_state.axis_pair(&PlayerAction::Move);
        assert_eq!(movement, Vec2::ZERO);

        println!("✅ ActionState correctly handles movement input for replication");
    }

    /// Test that components for a player entity can all coexist
    #[test]
    fn test_player_entity_component_bundle() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        // Spawn entity with all player components (new modular approach)
        let entity = app.world_mut().spawn((
            Position(Vec3::ZERO),
            Rotation(Quat::IDENTITY),
            MovementConfig::default(),
            FpsCamera::default(),
            GroundState::default(),
            ActionState::<PlayerAction>::default(),
        )).id();

        // Verify all components present
        assert!(app.world().get::<Position>(entity).is_some());
        assert!(app.world().get::<Rotation>(entity).is_some());
        assert!(app.world().get::<MovementConfig>(entity).is_some());
        assert!(app.world().get::<FpsCamera>(entity).is_some());
        assert!(app.world().get::<GroundState>(entity).is_some());
        assert!(app.world().get::<ActionState<PlayerAction>>(entity).is_some());

        // Update FpsCamera
        let camera = FpsCamera {
            pitch: 0.5,
            yaw: 1.0,
            ..Default::default()
        };
        app.world_mut().entity_mut(entity).insert(camera);

        let camera = app.world().get::<FpsCamera>(entity).unwrap();
        assert_eq!(camera.pitch, 0.5);
        assert_eq!(camera.yaw, 1.0);

        // Update MovementConfig
        let config = MovementConfig {
            walk_speed: 50.0,
            run_speed: 100.0,
            ..Default::default()
        };
        app.world_mut().entity_mut(entity).insert(config);

        let config = app.world().get::<MovementConfig>(entity).unwrap();
        assert_eq!(config.walk_speed, 50.0);
        assert_eq!(config.run_speed, 100.0);

        println!("✅ Player entity holds all required modular components for movement replication");
    }

    /// Test movement state transitions
    #[test]
    fn test_movement_state_transitions() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let entity = app.world_mut().spawn((
            Position(Vec3::ZERO),
            ActionState::<PlayerAction>::default(),
        )).id();

        // Start movement
        app.world_mut().entity_mut(entity).get_mut::<ActionState<PlayerAction>>()
            .unwrap().set_axis_pair(&PlayerAction::Move, Vec2::Y);

        let moving = app.world().get::<ActionState<PlayerAction>>(entity).unwrap()
            .axis_pair(&PlayerAction::Move);
        assert_ne!(moving, Vec2::ZERO);

        // Stop movement
        app.world_mut().entity_mut(entity).get_mut::<ActionState<PlayerAction>>()
            .unwrap().set_axis_pair(&PlayerAction::Move, Vec2::ZERO);

        let stopped = app.world().get::<ActionState<PlayerAction>>(entity).unwrap()
            .axis_pair(&PlayerAction::Move);
        assert_eq!(stopped, Vec2::ZERO);

        println!("✅ Movement state transitions work correctly");
    }
}
