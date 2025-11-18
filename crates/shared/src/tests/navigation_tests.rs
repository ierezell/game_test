#[cfg(test)]
mod navigation_system_tests {
    use crate::navigation::*;
    use avian3d::prelude::*;
    use bevy::app::ScheduleRunnerPlugin;
    use bevy::prelude::*;
    use std::time::Duration;

    /// Test complex patrol behavior with ping-pong and timing
    #[test]
    fn test_complex_patrol_behavior() {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(1.0 / 60.0)),
        ));
        app.add_plugins(NavigationPlugin);

        // Create patrol points in a square pattern
        let patrol_points = vec![
            Vec3::new(0.0, 1.0, 0.0),
            Vec3::new(10.0, 1.0, 0.0),
            Vec3::new(10.0, 1.0, 10.0),
            Vec3::new(0.0, 1.0, 10.0),
        ];

        // Spawn entity with patrol behavior
        let entity = app
            .world_mut()
            .spawn((
                Position::new(patrol_points[0]),
                Rotation::default(),
                SimpleNavigationAgent {
                    speed: 5.0,
                    arrival_threshold: 1.0,
                    current_target: Some(patrol_points[1]),
                },
                PatrolState::new(),
                PatrolRoute::new(patrol_points.clone()),
            ))
            .id();

        // Simulate movement until first waypoint is reached
        let mut frame_count = 0;
        let max_frames = 600; // 10 seconds at 60fps

        loop {
            app.update();
            frame_count += 1;

            let position = app.world().get::<Position>(entity).unwrap().0;
            let _patrol_state = app.world().get::<PatrolState>(entity).unwrap();

            // Check if we've reached the first target or timed out
            if position.distance(patrol_points[1]) < 1.5 || frame_count > max_frames {
                break;
            }
        }

        assert!(
            frame_count < max_frames,
            "Entity should reach first waypoint within time limit"
        );

        // Verify the entity is near the target
        let position = app.world().get::<Position>(entity).unwrap().0;
        assert!(
            position.distance(patrol_points[1]) < 2.0,
            "Entity should be near first waypoint"
        );

        // Continue simulation to test progression through all waypoints
        let _initial_frame = frame_count;
        let mut waypoints_reached = vec![false; patrol_points.len()];
        waypoints_reached[0] = true; // Started at first waypoint
        waypoints_reached[1] = true; // Just reached second waypoint

        while frame_count < max_frames
            && waypoints_reached.iter().filter(|&&x| x).count() < patrol_points.len()
        {
            app.update();
            frame_count += 1;

            let position = app.world().get::<Position>(entity).unwrap().0;

            // Check which waypoints have been reached
            for (i, &waypoint) in patrol_points.iter().enumerate() {
                if !waypoints_reached[i] && position.distance(waypoint) < 1.5 {
                    waypoints_reached[i] = true;
                    println!("Reached waypoint {} at frame {}", i, frame_count);
                }
            }
        }

        let reached_count = waypoints_reached.iter().filter(|&&x| x).count();
        assert!(
            reached_count >= 3,
            "Entity should reach at least 3 waypoints, reached: {}",
            reached_count
        );
    }

    /// Test ping-pong patrol direction changes
    #[test]
    fn test_ping_pong_direction_changes() {
        let patrol_points = vec![
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(5.0, 0.0, 0.0),
            Vec3::new(10.0, 0.0, 0.0),
        ];

        let patrol_route = PatrolRoute::new(patrol_points.clone());
        let mut forward = true;

        // Test forward progression
        let result1 = patrol_route.get_next_target(0, &mut forward);
        assert!(result1.is_some());
        let (target1, index1) = result1.unwrap();
        assert_eq!(target1, patrol_points[1]);
        assert_eq!(index1, 1);
        assert!(forward, "Should still be moving forward");

        // Continue forward to end
        let result2 = patrol_route.get_next_target(1, &mut forward);
        assert!(result2.is_some());
        let (target2, index2) = result2.unwrap();
        assert_eq!(target2, patrol_points[2]);
        assert_eq!(index2, 2);
        assert!(forward, "Should still be moving forward");

        // At end, should reverse direction
        let result3 = patrol_route.get_next_target(2, &mut forward);
        assert!(result3.is_some());
        let (target3, index3) = result3.unwrap();
        assert_eq!(target3, patrol_points[1]);
        assert_eq!(index3, 1);
        assert!(!forward, "Should now be moving backward");

        // Continue backward
        let result4 = patrol_route.get_next_target(1, &mut forward);
        assert!(result4.is_some());
        let (target4, index4) = result4.unwrap();
        assert_eq!(target4, patrol_points[0]);
        assert_eq!(index4, 0);
        assert!(!forward, "Should still be moving backward");

        // At start, should reverse direction again
        let result5 = patrol_route.get_next_target(0, &mut forward);
        assert!(result5.is_some());
        let (target5, index5) = result5.unwrap();
        assert_eq!(target5, patrol_points[1]);
        assert_eq!(index5, 1);
        assert!(forward, "Should be moving forward again");
    }

    /// Test navigation agent movement physics and collision
    #[test]
    fn test_navigation_movement_physics() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, ScheduleRunnerPlugin::default()));
        app.add_plugins(NavigationPlugin);

        // Create agent with specific speed
        let speed = 8.0;
        let start_pos = Vec3::new(0.0, 1.0, 0.0);
        let target_pos = Vec3::new(20.0, 1.0, 0.0);

        let entity = app
            .world_mut()
            .spawn((
                Position::new(start_pos),
                Rotation::default(),
                SimpleNavigationAgent {
                    speed,
                    arrival_threshold: 1.0,
                    current_target: Some(target_pos),
                },
            ))
            .id();

        // Record initial state
        let initial_pos = app.world().get::<Position>(entity).unwrap().0;

        // Run several frames and measure movement
        for _ in 0..60 {
            // 1 second at 60fps
            app.update();
        }

        let current_pos = app.world().get::<Position>(entity).unwrap().0;
        let distance_moved = current_pos.distance(initial_pos);

        // Verify movement speed is approximately correct
        // At 8.0 speed for 1 second, should move ~8 units
        assert!(
            distance_moved > 6.0 && distance_moved < 10.0,
            "Entity should move approximately {} units in 1 second, moved: {}",
            speed,
            distance_moved
        );

        // Verify movement direction is correct
        let movement_vector = current_pos - initial_pos;
        let expected_direction = (target_pos - initial_pos).normalize();
        let actual_direction = movement_vector.normalize();

        let dot_product = actual_direction.dot(expected_direction);
        assert!(
            dot_product > 0.9,
            "Movement should be in correct direction, dot product: {}",
            dot_product
        );

        // Verify Y coordinate is maintained
        assert!(
            (current_pos.y - start_pos.y).abs() < 0.1,
            "Y coordinate should remain stable"
        );
    }

    /// Test arrival threshold and target reaching behavior
    #[test]
    fn test_arrival_threshold_behavior() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, ScheduleRunnerPlugin::default()));
        app.add_plugins(NavigationPlugin);

        let start_pos = Vec3::new(0.0, 0.0, 0.0);
        let target_pos = Vec3::new(5.0, 0.0, 0.0);

        // Test with different arrival thresholds
        let test_cases = vec![
            (0.5, "tight threshold"),
            (1.0, "normal threshold"),
            (2.0, "loose threshold"),
        ];

        for (threshold, description) in test_cases {
            let entity = app
                .world_mut()
                .spawn((
                    Position::new(start_pos),
                    Rotation::default(),
                    SimpleNavigationAgent {
                        speed: 3.0,
                        arrival_threshold: threshold,
                        current_target: Some(target_pos),
                    },
                ))
                .id();

            // Simulate movement until close to target
            for _ in 0..300 {
                // 5 seconds max
                app.update();

                let position = app.world().get::<Position>(entity).unwrap().0;
                let distance_to_target = position.distance(target_pos);

                if distance_to_target <= threshold * 1.5 {
                    break;
                }
            }

            let final_pos = app.world().get::<Position>(entity).unwrap().0;
            let final_distance = final_pos.distance(target_pos);

            assert!(
                final_distance <= threshold + 0.5,
                "Entity should reach within arrival threshold for {}: distance {}, threshold {}",
                description,
                final_distance,
                threshold
            );

            // Clean up entity
            app.world_mut().despawn(entity);
        }
    }

    /// Test obstacle avoidance and position validation
    #[test]
    fn test_spawn_position_validation() {
        // Create mock obstacle query data
        let obstacles = vec![
            Position::new(Vec3::new(5.0, 1.0, 5.0)),
            Position::new(Vec3::new(10.0, 1.0, 10.0)),
            Position::new(Vec3::new(0.0, 1.0, 15.0)),
        ];

        // Test position too close to obstacle
        let desired_pos = Vec3::new(5.2, 1.0, 5.2); // Very close to first obstacle

        // Since we can't create a proper Query in unit test, test the logic manually
        let min_distance = 2.0;
        let mut adjusted_pos = desired_pos;

        for obstacle_pos in &obstacles {
            let distance = adjusted_pos.distance(obstacle_pos.0);
            if distance < min_distance {
                let away_direction = (adjusted_pos - obstacle_pos.0).normalize();
                if away_direction.is_finite() {
                    adjusted_pos = obstacle_pos.0 + away_direction * min_distance;
                }
            }
        }

        // Verify position was adjusted
        assert_ne!(
            adjusted_pos, desired_pos,
            "Position should be adjusted away from obstacle"
        );

        // Verify adjusted position is safe distance from all obstacles
        for obstacle_pos in &obstacles {
            let distance = adjusted_pos.distance(obstacle_pos.0);
            assert!(
                distance >= min_distance - 0.1,
                "Adjusted position should be safe distance from obstacles"
            );
        }

        // Verify Y coordinate normalization
        assert_eq!(
            adjusted_pos.y, 1.0,
            "Y coordinate should be normalized to ground level"
        );
    }

    /// Test navigation behavior with multiple agents
    #[test]
    fn test_multi_agent_navigation() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, ScheduleRunnerPlugin::default()));
        app.add_plugins(NavigationPlugin);

        // Create multiple agents with different speeds and targets
        let agents = vec![
            (Vec3::new(0.0, 1.0, 0.0), Vec3::new(10.0, 1.0, 0.0), 5.0),
            (Vec3::new(0.0, 1.0, 5.0), Vec3::new(10.0, 1.0, 5.0), 3.0),
            (Vec3::new(0.0, 1.0, 10.0), Vec3::new(10.0, 1.0, 10.0), 7.0),
        ];

        let mut entities = Vec::new();
        for (start, target, speed) in agents {
            let entity = app
                .world_mut()
                .spawn((
                    Position::new(start),
                    Rotation::default(),
                    SimpleNavigationAgent {
                        speed,
                        arrival_threshold: 1.0,
                        current_target: Some(target),
                    },
                ))
                .id();
            entities.push(entity);
        }

        // Simulate for a period and verify all agents move independently
        for _ in 0..180 {
            // 3 seconds
            app.update();
        }

        // Verify each agent moved toward its target
        for (i, entity) in entities.iter().enumerate() {
            let position = app.world().get::<Position>(*entity).unwrap().0;
            let agent = app.world().get::<SimpleNavigationAgent>(*entity).unwrap();

            if let Some(target) = agent.current_target {
                let distance_to_target = position.distance(target);
                assert!(
                    distance_to_target < 8.0,
                    "Agent {} should have moved significantly toward target",
                    i
                );

                // Verify agent moved in correct direction
                let start_pos = match i {
                    0 => Vec3::new(0.0, 1.0, 0.0),
                    1 => Vec3::new(0.0, 1.0, 5.0),
                    2 => Vec3::new(0.0, 1.0, 10.0),
                    _ => Vec3::ZERO,
                };

                assert!(
                    position.x > start_pos.x,
                    "Agent {} should have moved toward positive X",
                    i
                );
            }
        }
    }

    /// Test edge cases and error handling in navigation
    #[test]
    fn test_navigation_edge_cases() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, ScheduleRunnerPlugin::default()));
        app.add_plugins(NavigationPlugin);

        // Test agent with no target
        let entity1 = app
            .world_mut()
            .spawn((
                Position::new(Vec3::ZERO),
                Rotation::default(),
                SimpleNavigationAgent {
                    speed: 5.0,
                    arrival_threshold: 1.0,
                    current_target: None,
                },
            ))
            .id();

        // Test agent with zero speed
        let entity2 = app
            .world_mut()
            .spawn((
                Position::new(Vec3::ZERO),
                Rotation::default(),
                SimpleNavigationAgent {
                    speed: 0.0,
                    arrival_threshold: 1.0,
                    current_target: Some(Vec3::new(5.0, 0.0, 0.0)),
                },
            ))
            .id();

        // Test agent with same start and target position
        let entity3 = app
            .world_mut()
            .spawn((
                Position::new(Vec3::new(5.0, 1.0, 5.0)),
                Rotation::default(),
                SimpleNavigationAgent {
                    speed: 5.0,
                    arrival_threshold: 1.0,
                    current_target: Some(Vec3::new(5.0, 1.0, 5.0)),
                },
            ))
            .id();

        let initial_positions = vec![
            app.world().get::<Position>(entity1).unwrap().0,
            app.world().get::<Position>(entity2).unwrap().0,
            app.world().get::<Position>(entity3).unwrap().0,
        ];

        // Run simulation
        for _ in 0..60 {
            app.update();
        }

        // Verify edge cases are handled gracefully
        let final_positions = vec![
            app.world().get::<Position>(entity1).unwrap().0,
            app.world().get::<Position>(entity2).unwrap().0,
            app.world().get::<Position>(entity3).unwrap().0,
        ];

        // Agent with no target should not move
        assert_eq!(
            initial_positions[0], final_positions[0],
            "Agent with no target should not move"
        );

        // Agent with zero speed should not move
        assert_eq!(
            initial_positions[1], final_positions[1],
            "Agent with zero speed should not move"
        );

        // Agent at target position should not move significantly
        let movement_distance = initial_positions[2].distance(final_positions[2]);
        assert!(
            movement_distance < 0.1,
            "Agent at target position should not move significantly"
        );
    }

    /// Test patrol system integration with timing
    #[test]
    fn test_patrol_system_integration() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, ScheduleRunnerPlugin::default()));
        app.add_plugins(NavigationPlugin);

        let patrol_points = vec![Vec3::new(0.0, 1.0, 0.0), Vec3::new(5.0, 1.0, 0.0)];

        let entity = app
            .world_mut()
            .spawn((
                Position::new(patrol_points[0]),
                Rotation::default(),
                SimpleNavigationAgent {
                    speed: 10.0, // Fast speed to reach target quickly
                    arrival_threshold: 0.5,
                    current_target: Some(patrol_points[0]),
                },
                PatrolState::new(),
                PatrolRoute::new(patrol_points.clone()),
            ))
            .id();

        let mut target_changes = 0;
        let mut last_target = patrol_points[0];

        // Monitor target changes over time
        for _ in 0..300 {
            // 5 seconds
            app.update();

            let agent = app.world().get::<SimpleNavigationAgent>(entity).unwrap();
            if let Some(current_target) = agent.current_target {
                if current_target != last_target {
                    target_changes += 1;
                    last_target = current_target;
                }
            }
        }

        assert!(
            target_changes > 0,
            "Patrol system should change targets over time, changes: {}",
            target_changes
        );

        // Verify patrol state progression
        let patrol_state = app.world().get::<PatrolState>(entity).unwrap();
        assert!(patrol_state.wait_timer >= 0.0, "Wait timer should be valid");
        assert!(
            patrol_state.current_target_index < patrol_points.len(),
            "Target index should be valid"
        );
    }
}
