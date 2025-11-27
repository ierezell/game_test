mod common;

use bevy::prelude::*;
use common::*;
use shared::{
    navigation::{
        SimpleNavigationAgent, PatrolRoute, PatrolState,
        validate_spawn_position, NavigationObstacle
    },
};
use avian3d::prelude::*;

/// Test navigation agent creation and properties
#[test]
fn test_navigation_agent_creation() {
    let agent = SimpleNavigationAgent::new(5.0);
    assert_eq!(agent.speed, 5.0);
    assert_eq!(agent.arrival_threshold, 1.0);
    assert_eq!(agent.current_target, None);

    let bot_agent = SimpleNavigationAgent::bot();
    assert_eq!(bot_agent.speed, 3.0);
    assert_eq!(bot_agent.arrival_threshold, 1.5);
    assert_eq!(bot_agent.current_target, None);
}

/// Test patrol route functionality
#[test]
fn test_patrol_route_functionality() {
    let points = vec![
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(5.0, 0.0, 0.0),
        Vec3::new(10.0, 0.0, 0.0),
    ];
    
    let route = PatrolRoute::new(points.clone());
    assert_eq!(route.points, points);
    assert!(route.ping_pong);

    // Test getting next target in ping-pong mode
    let mut forward = true;
    let (next_pos, next_index) = route.get_next_target(0, &mut forward).unwrap();
    assert_eq!(next_pos, Vec3::new(5.0, 0.0, 0.0));
    assert_eq!(next_index, 1);
    assert!(forward);

    // Test reaching the end and turning around
    let mut forward = true;
    let (next_pos, next_index) = route.get_next_target(2, &mut forward).unwrap();
    assert_eq!(next_pos, Vec3::new(5.0, 0.0, 0.0)); // Should go back
    assert_eq!(next_index, 1);
    assert!(!forward); // Should reverse direction
}

/// Test patrol route edge cases
#[test]
fn test_patrol_route_edge_cases() {
    // Empty route
    let empty_route = PatrolRoute::new(vec![]);
    let mut forward = true;
    assert!(empty_route.get_next_target(0, &mut forward).is_none());

    // Single point route
    let single_point = PatrolRoute::new(vec![Vec3::ZERO]);
    let mut forward = true;
    assert!(single_point.get_next_target(0, &mut forward).is_none());

    // Two point route
    let two_points = PatrolRoute::new(vec![Vec3::ZERO, Vec3::new(1.0, 0.0, 0.0)]);
    let mut forward = true;
    
    // From point 0, should go to point 1
    let (_next_pos, next_index) = two_points.get_next_target(0, &mut forward).unwrap();
    assert_eq!(next_index, 1);
    assert!(forward);
    
    // From point 1, should reverse and go to point 0
    let (_next_pos, next_index) = two_points.get_next_target(1, &mut forward).unwrap();
    assert_eq!(next_index, 0);
    assert!(!forward);
}

/// Test patrol state management
#[test]
fn test_patrol_state_management() {
    let mut state = PatrolState::default();
    assert_eq!(state.current_target_index, 0);
    assert_eq!(state.wait_timer, 0.0);
    assert_eq!(state.wait_duration, 2.0);
    assert!(state.forward);

    // Test timer progression
    state.wait_timer = 1.5;
    assert!(state.wait_timer < state.wait_duration);
    
    state.wait_timer = 2.5;
    assert!(state.wait_timer >= state.wait_duration);
}

/// System to test validation functionality
fn test_validation_system(
    obstacles: Query<&Position, With<NavigationObstacle>>,
) {
    // Test position that's clear (should remain unchanged)
    let clear_position = Vec3::new(0.0, 1.0, 0.0);
    let validated_position = validate_spawn_position(
        clear_position, 
        &obstacles, 
        1.0
    );
    // Position should be close to original if clear
    assert!((validated_position - clear_position).length() < 0.1);

    // Test position too close to obstacle (should be adjusted)
    let close_position = Vec3::new(5.1, 1.0, 5.1);
    let adjusted_position = validate_spawn_position(
        close_position, 
        &obstacles, 
        1.0
    );
    // Should be moved away from obstacle
    let distance_from_obstacle = adjusted_position.distance(Vec3::new(5.0, 1.0, 5.0));
    assert!(distance_from_obstacle >= 2.5, "Should be moved away from obstacle");
}

/// Test spawn position validation
#[test]
fn test_spawn_position_validation() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    // Create obstacles
    let _obstacle1 = app.world_mut().spawn((
        Position(Vec3::new(5.0, 1.0, 5.0)),
        NavigationObstacle,
    )).id();
    
    let _obstacle2 = app.world_mut().spawn((
        Position(Vec3::new(10.0, 1.0, 10.0)),
        NavigationObstacle,
    )).id();

    // Add a system to test validation
    app.add_systems(Update, test_validation_system);
    
    // Run the system to test validation
    app.update();
}

/// Test navigation agent with simple movement
#[test]
fn test_navigation_agent_target_setting() {
    let mut agent = SimpleNavigationAgent::new(4.0);
    assert_eq!(agent.current_target, None);

    // Set a target
    let target = Vec3::new(10.0, 0.0, 5.0);
    agent.current_target = Some(target);
    assert_eq!(agent.current_target, Some(target));

    // Test arrival threshold
    assert_eq!(agent.arrival_threshold, 1.0);
    agent.arrival_threshold = 2.5;
    assert_eq!(agent.arrival_threshold, 2.5);
}

/// Test navigation system components integration
#[test]
fn test_navigation_system_components() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    // Spawn entity with navigation components
    let nav_entity = app.world_mut().spawn((
        SimpleNavigationAgent::bot(),
        PatrolState::default(),
        PatrolRoute::new(vec![
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(10.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 10.0),
        ]),
        Position(Vec3::new(0.0, 1.0, 0.0)),
        Rotation::default(),
    )).id();

    // Verify all components are present
    assert!(app.world().get::<SimpleNavigationAgent>(nav_entity).is_some());
    assert!(app.world().get::<PatrolState>(nav_entity).is_some());
    assert!(app.world().get::<PatrolRoute>(nav_entity).is_some());
    assert!(app.world().get::<Position>(nav_entity).is_some());
    assert!(app.world().get::<Rotation>(nav_entity).is_some());

    // Test component values
    let agent = app.world().get::<SimpleNavigationAgent>(nav_entity).unwrap();
    assert_eq!(agent.speed, 3.0); // bot() creates agent with speed 3.0
    
    let route = app.world().get::<PatrolRoute>(nav_entity).unwrap();
    assert_eq!(route.points.len(), 3);
}

/// Test multiple navigation entities
#[test]
fn test_multiple_navigation_entities() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    // Create multiple navigation entities with different properties
    let fast_patrol = app.world_mut().spawn((
        SimpleNavigationAgent::new(8.0),
        PatrolState::default(),
        PatrolRoute::new(vec![Vec3::ZERO, Vec3::new(20.0, 0.0, 0.0)]),
        Position(Vec3::ZERO),
        Rotation::default(),
    )).id();

    let slow_patrol = app.world_mut().spawn((
        SimpleNavigationAgent::new(2.0),
        PatrolState::default(), 
        PatrolRoute::new(vec![Vec3::ZERO, Vec3::new(5.0, 0.0, 5.0)]),
        Position(Vec3::new(10.0, 1.0, 10.0)),
        Rotation::default(),
    )).id();

    // Verify they have different properties
    let fast_agent = app.world().get::<SimpleNavigationAgent>(fast_patrol).unwrap();
    let slow_agent = app.world().get::<SimpleNavigationAgent>(slow_patrol).unwrap();
    
    assert_eq!(fast_agent.speed, 8.0);
    assert_eq!(slow_agent.speed, 2.0);
    assert_ne!(fast_agent.speed, slow_agent.speed);

    // Verify different positions
    let fast_pos = app.world().get::<Position>(fast_patrol).unwrap();
    let slow_pos = app.world().get::<Position>(slow_patrol).unwrap();
    
    assert_ne!(fast_pos.0, slow_pos.0);
}

/// Test patrol route with circular path
#[test]
fn test_circular_patrol_route() {
    let points = vec![
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(5.0, 0.0, 0.0),
        Vec3::new(5.0, 0.0, 5.0),
        Vec3::new(0.0, 0.0, 5.0),
    ];
    
    let mut route = PatrolRoute::new(points);
    route.ping_pong = false; // Circular mode
    
    // Test circular progression
    let mut forward = true;
    
    // From index 0 should go to 1
    let (_, next_index) = route.get_next_target(0, &mut forward).unwrap();
    assert_eq!(next_index, 1);
    
    // From index 3 (last) should wrap to 0
    let (_, next_index) = route.get_next_target(3, &mut forward).unwrap();
    assert_eq!(next_index, 0);
    
    // Direction shouldn't change in circular mode during normal progression
    assert!(forward);
}

/// Test navigation obstacle detection
#[test]
fn test_navigation_obstacle_detection() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    // Spawn obstacles at strategic locations
    let _wall1 = app.world_mut().spawn((
        Position(Vec3::new(5.0, 0.0, 0.0)),
        NavigationObstacle,
    )).id();
    
    let _wall2 = app.world_mut().spawn((
        Position(Vec3::new(15.0, 0.0, 0.0)),
        NavigationObstacle,
    )).id();

    // Query obstacles
    let mut obstacles_query = app.world_mut().query_filtered::<&Position, With<NavigationObstacle>>();
    let obstacles: Vec<&Position> = obstacles_query.iter(app.world()).collect();
    
    assert_eq!(obstacles.len(), 2);
    
    // Verify obstacle positions
    let positions: Vec<Vec3> = obstacles.iter().map(|pos| pos.0).collect();
    assert!(positions.contains(&Vec3::new(5.0, 0.0, 0.0)));
    assert!(positions.contains(&Vec3::new(15.0, 0.0, 0.0)));
}