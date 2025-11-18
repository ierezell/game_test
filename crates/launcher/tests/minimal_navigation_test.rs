use bevy::prelude::*;
use shared::navigation::{PatrolRoute, PatrolState, SimpleNavigationAgent, setup_patrol};

/// Test that navigation components can be created and accessed
#[test]
fn test_navigation_components_basic() {
    println!("=== Testing Basic Navigation Components ===");

    // Create minimal headless app
    let mut app = App::new();

    // Create NPC with navigation components
    let npc_entity = app
        .world_mut()
        .spawn((
            Name::new("Test_NPC"),
            Transform::from_xyz(0.0, 1.0, 0.0),
            SimpleNavigationAgent::bot(),
            PatrolState::new(),
        ))
        .id();

    println!("✓ NPC created with entity ID: {:?}", npc_entity);

    // Verify components exist
    let has_agent = app
        .world()
        .get::<SimpleNavigationAgent>(npc_entity)
        .is_some();
    let has_patrol_state = app.world().get::<PatrolState>(npc_entity).is_some();
    let has_transform = app.world().get::<Transform>(npc_entity).is_some();
    let has_name = app.world().get::<Name>(npc_entity).is_some();

    assert!(has_agent, "SimpleNavigationAgent component should exist");
    assert!(has_patrol_state, "PatrolState component should exist");
    assert!(has_transform, "Transform component should exist");
    assert!(has_name, "Name component should exist");

    println!("✓ All navigation components verified");

    // Test SimpleNavigationAgent properties
    let agent = app
        .world()
        .get::<SimpleNavigationAgent>(npc_entity)
        .unwrap();
    assert_eq!(agent.speed, 3.0, "Agent speed should be 3.0 for bot()");
    assert_eq!(
        agent.arrival_threshold, 1.5,
        "Arrival threshold should be 1.5 for bot()"
    );
    assert_eq!(
        agent.current_target, None,
        "Bot should start with no target"
    );

    // Test PatrolState properties
    let patrol_state = app.world().get::<PatrolState>(npc_entity).unwrap();
    assert_eq!(
        patrol_state.current_target_index, 0,
        "Should start at index 0"
    );
    assert_eq!(patrol_state.wait_timer, 0.0, "Wait timer should start at 0");
    assert_eq!(
        patrol_state.wait_duration, 2.0,
        "Default wait duration should be 2.0"
    );
    assert!(patrol_state.forward, "Should start moving forward");

    println!("✓ Navigation component properties verified");

    // Create a patrol route
    let patrol_waypoints = vec![
        Vec3::new(0.0, 1.0, 0.0),
        Vec3::new(10.0, 1.0, 0.0),
        Vec3::new(10.0, 1.0, 10.0),
        Vec3::new(0.0, 1.0, 10.0),
    ];

    app.world_mut()
        .entity_mut(npc_entity)
        .insert(PatrolRoute::new(patrol_waypoints.clone()));

    // Verify patrol route was added
    let patrol_route = app.world().get::<PatrolRoute>(npc_entity).unwrap();
    assert_eq!(patrol_route.points.len(), 4, "Should have 4 waypoints");
    assert_eq!(patrol_route.points[0], Vec3::new(0.0, 1.0, 0.0));
    assert_eq!(patrol_route.points[1], Vec3::new(10.0, 1.0, 0.0));
    assert!(
        patrol_route.ping_pong,
        "Should be ping-pong patrol by default"
    );

    println!("✓ Patrol route configuration verified");
    println!("✓ Navigation components basic test passed!");
}

/// Test patrol route types and behavior
#[test]
fn test_patrol_route_types() {
    println!("=== Testing Patrol Route Types ===");

    let waypoints = vec![
        Vec3::new(0.0, 1.0, 0.0),
        Vec3::new(10.0, 1.0, 0.0),
        Vec3::new(10.0, 1.0, 10.0),
    ];

    // Test basic patrol route (ping-pong by default)
    let ping_pong_route = PatrolRoute::new(waypoints.clone());
    assert_eq!(ping_pong_route.points.len(), 3, "Should have 3 waypoints");
    assert!(ping_pong_route.ping_pong, "Should be ping-pong by default");

    // Test ping-pong behavior using get_next_target method
    let mut forward = true;
    if let Some((next_target, next_index)) = ping_pong_route.get_next_target(0, &mut forward) {
        assert_eq!(next_target, waypoints[1], "Should target second waypoint");
        assert_eq!(next_index, 1, "Should advance to index 1");
        assert!(forward, "Should still be moving forward");
    }

    // Test reaching end and reversing
    let mut forward = true;
    if let Some((next_target, next_index)) = ping_pong_route.get_next_target(2, &mut forward) {
        assert_eq!(next_target, waypoints[1], "Should target previous waypoint");
        assert_eq!(next_index, 1, "Should move back to index 1");
        assert!(!forward, "Should now be moving backward");
    }

    println!(
        "✓ Ping-pong patrol: {} waypoints, ping_pong={}",
        ping_pong_route.points.len(),
        ping_pong_route.ping_pong
    );

    println!("✓ Patrol route types test passed!");
}

/// Test setup_patrol helper function with minimal setup
#[test]
fn test_setup_patrol_basic() {
    println!("=== Testing setup_patrol Helper Function ===");

    let mut app = App::new();

    // Create basic NPC
    let npc_entity = app
        .world_mut()
        .spawn((
            Name::new("Patrol_Test_NPC"),
            Transform::from_xyz(5.0, 1.0, 5.0),
        ))
        .id();

    // Use setup_patrol function
    let patrol_points = vec![
        Vec3::new(5.0, 1.0, 5.0),
        Vec3::new(15.0, 1.0, 5.0),
        Vec3::new(15.0, 1.0, 15.0),
        Vec3::new(5.0, 1.0, 15.0),
    ];

    setup_patrol(
        &mut app.world_mut().commands(),
        npc_entity,
        patrol_points.clone(),
        3.0, // speed
    );

    // Apply commands by updating the app once
    app.update();

    // Verify setup_patrol added all components
    let nav_query = app
        .world_mut()
        .query::<(&SimpleNavigationAgent, &PatrolState, &PatrolRoute)>()
        .get(app.world(), npc_entity);

    assert!(
        nav_query.is_ok(),
        "setup_patrol should add all navigation components"
    );

    let (agent, patrol_state, patrol) = nav_query.unwrap();

    // Verify agent configuration
    assert_eq!(agent.speed, 3.0, "Agent speed should be 3.0");
    assert_eq!(
        agent.current_target,
        Some(patrol_points[0]),
        "Agent should have initial target set"
    );

    // Verify patrol route
    assert_eq!(patrol.points.len(), 4, "Should have 4 patrol points");
    assert_eq!(
        patrol.points, patrol_points,
        "Patrol points should match input"
    );
    assert!(patrol.ping_pong, "Should be ping-pong by default");

    // Verify patrol state is initialized
    assert_eq!(
        patrol_state.current_target_index, 0,
        "Should start at first patrol point"
    );

    println!("✓ setup_patrol function configuration verified");
    println!(
        "✓ Agent: speed={}, current_target={:?}",
        agent.speed, agent.current_target
    );
    println!(
        "✓ Route: {} points, ping_pong={}",
        patrol.points.len(),
        patrol.ping_pong
    );

    println!("✓ setup_patrol basic test passed!");
}
