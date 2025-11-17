use bevy::prelude::*;
use shared::navigation::{NavigationAgent, NavigationTarget, PatrolRoute, setup_patrol};

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
            NavigationAgent::bot(2.0),
            NavigationTarget::new(Vec3::new(10.0, 1.0, 0.0)),
        ))
        .id();

    println!("✓ NPC created with entity ID: {:?}", npc_entity);

    // Verify components exist
    let has_agent = app.world().get::<NavigationAgent>(npc_entity).is_some();
    let has_target = app.world().get::<NavigationTarget>(npc_entity).is_some();
    let has_transform = app.world().get::<Transform>(npc_entity).is_some();
    let has_name = app.world().get::<Name>(npc_entity).is_some();

    assert!(has_agent, "NavigationAgent component should exist");
    assert!(has_target, "NavigationTarget component should exist");
    assert!(has_transform, "Transform component should exist");
    assert!(has_name, "Name component should exist");

    println!("✓ All navigation components verified");

    // Test NavigationAgent properties
    let agent = app.world().get::<NavigationAgent>(npc_entity).unwrap();
    assert_eq!(agent.speed, 2.0, "Agent speed should be 2.0");
    assert_eq!(
        agent.arrival_threshold, 1.0,
        "Arrival threshold should be 1.0"
    );
    assert!(
        !agent.stop_at_destination,
        "Bot should not stop at destination"
    );
    assert!(matches!(
        agent.behavior,
        shared::navigation::NavigationBehavior::Patrol
    ));

    // Test NavigationTarget properties
    let target = app.world().get::<NavigationTarget>(npc_entity).unwrap();
    assert_eq!(
        target.destination,
        Vec3::new(10.0, 1.0, 0.0),
        "Target destination should match"
    );
    assert!(target.path.is_empty(), "Path should start empty");
    assert!(
        !target.pathfinding_in_progress,
        "Pathfinding should not be in progress"
    );

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
        !patrol_route.ping_pong,
        "Should not be ping-pong patrol by default"
    );
    assert_eq!(patrol_route.current_index, 0, "Should start at index 0");
    assert!(patrol_route.forward, "Should start moving forward");

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

    // Test basic loop patrol (default)
    let loop_route = PatrolRoute::new(waypoints.clone());
    assert_eq!(loop_route.points.len(), 3, "Should have 3 waypoints");
    assert!(!loop_route.ping_pong, "Should not be ping-pong by default");
    assert_eq!(loop_route.wait_time, 2.0, "Default wait time should be 2.0");
    assert!(loop_route.forward, "Should start moving forward");
    assert_eq!(loop_route.current_index, 0, "Should start at index 0");

    // Test ping-pong patrol
    let mut pingpong_route = PatrolRoute::new(waypoints.clone()).ping_pong(3.0);
    assert!(pingpong_route.ping_pong, "Should be ping-pong");
    assert_eq!(pingpong_route.wait_time, 3.0, "Wait time should be 3.0");

    // Test route advancement for loop
    let mut loop_route = PatrolRoute::new(waypoints.clone());
    assert_eq!(loop_route.current_index, 0);

    loop_route.advance();
    assert_eq!(loop_route.current_index, 1, "Should advance to index 1");

    loop_route.advance();
    assert_eq!(loop_route.current_index, 2, "Should advance to index 2");

    loop_route.advance();
    assert_eq!(loop_route.current_index, 0, "Should loop back to index 0");

    // Test route advancement for ping-pong
    pingpong_route.current_index = 0;
    pingpong_route.forward = true;

    pingpong_route.advance();
    assert_eq!(pingpong_route.current_index, 1, "Should advance to index 1");
    assert!(pingpong_route.forward, "Should still be moving forward");

    pingpong_route.advance();
    assert_eq!(pingpong_route.current_index, 2, "Should advance to index 2");
    assert!(!pingpong_route.forward, "Should now be moving backward");

    pingpong_route.advance();
    assert_eq!(
        pingpong_route.current_index, 1,
        "Should move back to index 1"
    );
    assert!(!pingpong_route.forward, "Should still be moving backward");

    pingpong_route.advance();
    assert_eq!(
        pingpong_route.current_index, 0,
        "Should move back to index 0"
    );
    assert!(pingpong_route.forward, "Should now be moving forward again");

    println!(
        "✓ Loop patrol: {} waypoints, ping_pong={}",
        loop_route.points.len(),
        loop_route.ping_pong
    );
    println!(
        "✓ Ping-pong patrol: {} waypoints, ping_pong={}, wait_time={}",
        pingpong_route.points.len(),
        pingpong_route.ping_pong,
        pingpong_route.wait_time
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
        3.0,   // speed
        false, // ping_pong
    );

    // Apply commands by updating the app once
    app.update();

    // Verify setup_patrol added all components
    let nav_query = app
        .world_mut()
        .query::<(&NavigationAgent, &NavigationTarget, &PatrolRoute)>()
        .get(app.world(), npc_entity);

    assert!(
        nav_query.is_ok(),
        "setup_patrol should add all navigation components"
    );

    let (agent, target, patrol) = nav_query.unwrap();

    // Verify agent configuration
    assert_eq!(agent.speed, 3.0, "Agent speed should be 3.0");
    assert!(matches!(
        agent.behavior,
        shared::navigation::NavigationBehavior::Patrol
    ));

    // Verify patrol route
    assert_eq!(patrol.points.len(), 4, "Should have 4 patrol points");
    assert_eq!(
        patrol.points, patrol_points,
        "Patrol points should match input"
    );
    assert!(!patrol.ping_pong, "Should not be ping-pong");

    // Verify initial target is set to first patrol point
    assert_eq!(
        target.destination, patrol_points[0],
        "Target should be first patrol point"
    );

    println!("✓ setup_patrol function configuration verified");
    println!("✓ Agent: speed={}, behavior=Patrol", agent.speed);
    println!(
        "✓ Route: {} points, ping_pong={}",
        patrol.points.len(),
        patrol.ping_pong
    );

    println!("✓ setup_patrol basic test passed!");
}
