use avian3d::prelude::*;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use vleue_navigator::prelude::*;

/// Component for entities that should block navigation (obstacles)
#[derive(Component, Clone, Debug)]
pub struct NavigationObstacle;

pub struct NavigationPlugin;

impl Plugin for NavigationPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(VleueNavigatorPlugin)
            .add_plugins(NavmeshUpdaterPlugin::<Collider, NavigationObstacle>::default())
            .add_systems(Update, (patrol_system,));
    }
}

/// Simplified navigation behavior - let vleue_navigator handle pathfinding
#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct SimpleNavigationAgent {
    pub speed: f32,
    pub arrival_threshold: f32,
}

impl SimpleNavigationAgent {
    pub fn new(speed: f32) -> Self {
        Self {
            speed,
            arrival_threshold: 1.0,
        }
    }

    pub fn bot() -> Self {
        Self {
            speed: 3.0,
            arrival_threshold: 1.5,
        }
    }
}

/// Simple patrol state
#[derive(Component, Clone, Debug)]
pub struct PatrolState {
    pub current_target_index: usize,
    pub wait_timer: f32,
    pub wait_duration: f32,
}

impl PatrolState {
    pub fn new() -> Self {
        Self {
            current_target_index: 0,
            wait_timer: 0.0,
            wait_duration: 2.0,
        }
    }
}

/// Simple patrol route component
#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct PatrolRoute {
    pub points: Vec<Vec3>,
    pub ping_pong: bool,
    pub forward: bool,
}

impl PatrolRoute {
    pub fn new(points: Vec<Vec3>) -> Self {
        Self {
            points,
            ping_pong: true,
            forward: true,
        }
    }

    pub fn get_next_target(&self, current_index: usize) -> Option<(Vec3, usize)> {
        if self.points.is_empty() {
            return None;
        }

        let mut next_index = current_index;
        
        if self.ping_pong {
            if self.forward {
                next_index += 1;
                if next_index >= self.points.len() {
                    next_index = self.points.len().saturating_sub(2);
                }
            } else {
                if next_index > 0 {
                    next_index -= 1;
                } else {
                    next_index = 1.min(self.points.len() - 1);
                }
            }
        } else {
            next_index = (next_index + 1) % self.points.len();
        }

        self.points.get(next_index).map(|&pos| (pos, next_index))
    }
}

/// Simple patrol system that uses vleue_navigator's NavMeshAgent for pathfinding
fn patrol_system(
    mut agents: Query<(
        Entity,
        &mut NavMeshAgent,
        &mut PatrolState,
        &PatrolRoute,
        &SimpleNavigationAgent,
        &Position,
    )>,
    time: Res<Time>,
) {
    for (entity, mut nav_agent, mut patrol_state, patrol_route, simple_agent, position) in agents.iter_mut() {
        // Update wait timer
        patrol_state.wait_timer += time.delta_secs();
        
        // Check if we've reached the current target
        let current_target = patrol_route.points.get(patrol_state.current_target_index);
        let reached_target = if let Some(target_pos) = current_target {
            let distance = Vec2::new(position.0.x, position.0.z)
                .distance(Vec2::new(target_pos.x, target_pos.z));
            distance < simple_agent.arrival_threshold
        } else {
            false
        };

        // If we've reached the target and waited long enough, move to next target
        if reached_target && patrol_state.wait_timer >= patrol_state.wait_duration {
            if let Some((next_target, next_index)) = patrol_route.get_next_target(patrol_state.current_target_index) {
                debug!("Entity {:?}: Moving to next patrol target: {:?} (index {})", 
                    entity, next_target, next_index);
                
                // Set new target using vleue_navigator's NavMeshAgent
                nav_agent.target = next_target;
                patrol_state.current_target_index = next_index;
                patrol_state.wait_timer = 0.0;
                
                // Update direction for ping-pong
                if patrol_route.ping_pong {
                    if next_index == patrol_route.points.len() - 1 {
                        // Reached end, reverse direction next time
                    } else if next_index == 0 {
                        // Reached start, go forward next time  
                    }
                }
            }
        } else if !reached_target {
            // Still moving towards target, reset wait timer
            patrol_state.wait_timer = 0.0;
        }
    }
}

/// Helper function to set up patrol behavior using vleue_navigator components
pub fn setup_patrol(
    commands: &mut Commands,
    entity: Entity,
    patrol_points: Vec<Vec3>,
    speed: f32,
) {
    let patrol_route = PatrolRoute::new(patrol_points.clone());
    let initial_target = patrol_route.points.first().copied().unwrap_or(Vec3::ZERO);

    commands.entity(entity).insert((
        SimpleNavigationAgent::new(speed),
        PatrolState::new(),
        patrol_route,
        NavMeshAgent::new(initial_target, speed),
    ));

    info!("Set up patrol for entity {:?} with {} points, initial target: {:?}", 
        entity, patrol_points.len(), initial_target);
}

/// Helper function to add navigation obstacle
pub fn add_navigation_obstacle(commands: &mut Commands, entity: Entity) {
    commands.entity(entity).insert(NavigationObstacle);
}

/// Validate spawn position to avoid obstacles
pub fn validate_spawn_position(
    position: Vec3,
    obstacles: &Query<&Position, With<NavigationObstacle>>,
    agent_radius: f32,
) -> Vec3 {
    let mut adjusted_position = position;
    let min_distance = agent_radius + 2.0;
    
    // Check if current position is too close to any obstacle
    for obstacle_pos in obstacles.iter() {
        let distance = adjusted_position.distance(obstacle_pos.0);
        if distance < min_distance {
            // Find a safe position by moving away from the obstacle
            let away_direction = (adjusted_position - obstacle_pos.0).normalize();
            if away_direction.is_finite() {
                adjusted_position = obstacle_pos.0 + away_direction * min_distance;
                info!("Adjusted spawn position from {:?} to {:?} to avoid obstacle at {:?}", 
                    position, adjusted_position, obstacle_pos.0);
            }
        }
    }
    
    // Ensure the position is at the correct height level
    adjusted_position.y = 1.0;
    adjusted_position
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_navigation_agent() {
        let agent = SimpleNavigationAgent::new(5.0);
        assert_eq!(agent.speed, 5.0);
        assert_eq!(agent.arrival_threshold, 1.0);

        let bot = SimpleNavigationAgent::bot();
        assert_eq!(bot.speed, 3.0);
        assert_eq!(bot.arrival_threshold, 1.5);
    }

    #[test]
    fn test_patrol_route() {
        let points = vec![
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(10.0, 0.0, 0.0),
            Vec3::new(10.0, 0.0, 10.0),
        ];

        let patrol_route = PatrolRoute::new(points.clone());
        assert_eq!(patrol_route.points, points);
        assert!(patrol_route.ping_pong);
        assert!(patrol_route.forward);

        // Test getting next target
        if let Some((next_target, next_index)) = patrol_route.get_next_target(0) {
            assert_eq!(next_target, points[1]);
            assert_eq!(next_index, 1);
        }
    }

    #[test]
    fn test_patrol_state() {
        let state = PatrolState::new();
        assert_eq!(state.current_target_index, 0);
        assert_eq!(state.wait_timer, 0.0);
        assert_eq!(state.wait_duration, 2.0);
    }
}

        // Handle follow behavior
        if let Some(follow_entity) = target.follow_target {
            if let Ok(follow_pos) = follow_targets.get(follow_entity) {
                let distance = position.0.distance(follow_pos.0);
                if distance > 2.0 {
                    new_destination = follow_pos.0;
                    needs_new_path = true;
                    info!(
                        "Entity {:?}: Follow behavior - new destination {:?}, distance: {:.2}",
                        entity, new_destination, distance
                    );
                }
            }
        }

        // Check if we need a new path - be more conservative about recalculation
        let destination_changed = new_destination.distance(target.destination) > 0.5;
        let path_empty = target.path.is_empty();
        let path_too_old = (current_time - target.pathfind_request_time) > 10.0; // Increased from 2.0 to 10.0
        let not_patrol_behavior = !matches!(agent.behavior, NavigationBehavior::Patrol);

        needs_new_path = needs_new_path
            || (path_empty && not_patrol_behavior)
            || (destination_changed && not_patrol_behavior)
            || path_too_old;

        // Removed excessive trace logging for patrol entities

        if needs_new_path && !target.pathfinding_in_progress {
            info!(
                "Entity {:?}: Calculating new path from {:?} to {:?} (reasons: dest_changed={}, path_empty={}, path_old={}, not_patrol={})",
                entity,
                position.0,
                new_destination,
                destination_changed,
                path_empty,
                path_too_old,
                not_patrol_behavior
            );

            target.pathfinding_in_progress = true;
            target.destination = new_destination;

            info!("Entity {:?}: TRIGGERING PATH CALCULATION from {:?} to {:?}", entity, position.0, target.destination);
            if let Some(new_path) = compute_path(navmesh, position.0, target.destination) {
                info!(
                    "Entity {:?}: Path calculated with {} waypoints: {:?}",
                    entity,
                    new_path.len(),
                    new_path.iter().take(3).collect::<Vec<_>>()
                );
                target.path = new_path;
                target.pathfind_request_time = current_time;
                target.last_pathfind_success = true;
            } else {
                warn!(
                    "Entity {:?}: Failed to calculate path from {:?} to {:?}",
                    entity, position.0, target.destination
                );
                target.last_pathfind_success = false;
            }

            target.pathfinding_in_progress = false;
        }
    }
}

/// System to move navigation agents along their paths
fn move_navigation_agents(
    mut agents: Query<(
        Entity,
        &mut Position,
        Option<&mut LinearVelocity>,
        &mut NavigationTarget,
        &mut NavigationAgent,
    )>,
    obstacles: Query<&Position, (With<NavigationObstacle>, Without<NavigationAgent>)>,
    time: Res<Time>,
) {
    let delta_time = time.delta_secs();
    let current_time = time.elapsed_secs();

    for (entity, mut position, mut maybe_linear_velocity, mut target, mut agent) in
        agents.iter_mut()
    {
        if let Some(next_waypoint) = target.get_next_waypoint() {
            let current_pos = position.0;
            let to_waypoint = next_waypoint - current_pos;
            let distance_to_waypoint = to_waypoint.length();
            
            // Reduced logging for smoother output
            if distance_to_waypoint > agent.arrival_threshold * 2.0 {
                trace!("Entity {:?}: Moving to waypoint {:?}, distance: {:.2}", 
                    entity, next_waypoint, distance_to_waypoint);
            }

            // Removed excessive debug logging

            // Only move if we're not already at the waypoint
            if distance_to_waypoint > agent.arrival_threshold {
                let direction = to_waypoint.normalize();

                if direction.is_finite() {
                    // Smooth acceleration/deceleration with lookahead for smoother movement
                    let desired_velocity = direction * agent.speed;

                    // Slow down when approaching waypoint to prevent overshoot
                    let slowdown_distance = agent.arrival_threshold * 3.0;
                    let speed_multiplier = if distance_to_waypoint < slowdown_distance {
                        (distance_to_waypoint / slowdown_distance).max(0.3) // Min 30% speed
                    } else {
                        1.0
                    };

                    let adjusted_desired_velocity = desired_velocity * speed_multiplier;
                    let velocity_change = adjusted_desired_velocity - agent.velocity;
                    let max_velocity_change = agent.max_acceleration * delta_time;

                    let velocity_change_clamped =
                        velocity_change.clamp_length_max(max_velocity_change);
                    agent.velocity += velocity_change_clamped;

                    // Apply movement with collision checking
                    let movement = agent.velocity * delta_time;
                    let new_position = position.0 + movement;
                    
                    // Improved obstacle avoidance with stuck detection
                    let mut collision_detected = false;
                    let mut nearest_obstacle_distance = f32::MAX;
                    
                    for obstacle_pos in obstacles.iter() {
                        let distance = new_position.distance(obstacle_pos.0);
                        let current_distance = position.0.distance(obstacle_pos.0);
                        nearest_obstacle_distance = nearest_obstacle_distance.min(distance);
                        
                        // More generous collision detection with obstacle size
                        let min_distance = agent.radius + 1.8; // More space around obstacles
                        
                        if distance < min_distance {
                            // Check if we're getting closer to obstacle - if so, avoid
                            if distance < current_distance {
                                collision_detected = true;
                                trace!("Entity {:?}: Avoiding obstacle at {:?} (distance: {:.2})", entity, obstacle_pos.0, distance);
                                break;
                            }
                        }
                    }
                    
                    // Only apply movement if no collision or if moving away from obstacles
                    if !collision_detected {
                        position.0 = new_position;
                        
                        // Update linear velocity component if present
                        if let Some(ref mut lin_vel) = maybe_linear_velocity {
                            lin_vel.0 = agent.velocity;
                        }
                    } else {
                        // Try alternative movement - slide along obstacles instead of stopping
                        let mut slide_direction = Vec3::ZERO;
                        
                        // Calculate slide direction perpendicular to obstacle
                        for obstacle_pos in obstacles.iter() {
                            let to_obstacle = obstacle_pos.0 - position.0;
                            if to_obstacle.length() < agent.radius + 2.0 {
                                let perpendicular = Vec3::new(-to_obstacle.z, to_obstacle.y, to_obstacle.x).normalize();
                                slide_direction += perpendicular;
                            }
                        }
                        
                        if slide_direction.length() > 0.1 {
                            // Try sliding movement
                            let slide_movement = slide_direction.normalize() * agent.speed * delta_time * 0.5;
                            let slide_position = position.0 + slide_movement;
                            
                            // Check if slide movement is safe
                            let mut slide_safe = true;
                            for obstacle_pos in obstacles.iter() {
                                if slide_position.distance(obstacle_pos.0) < agent.radius + 1.5 {
                                    slide_safe = false;
                                    break;
                                }
                            }
                            
                            if slide_safe {
                                position.0 = slide_position;
                                agent.velocity = slide_direction.normalize() * agent.speed * 0.3;
                                if let Some(ref mut lin_vel) = maybe_linear_velocity {
                                    lin_vel.0 = agent.velocity;
                                }
                            } else {
                                // Completely stuck - request new path but don't clear immediately
                                agent.velocity = Vec3::ZERO;
                                if let Some(ref mut lin_vel) = maybe_linear_velocity {
                                    lin_vel.0 = Vec3::ZERO;
                                }
                                
                                // Only clear path after being stuck for a while
                                if (current_time - target.pathfind_request_time) > 3.0 {
                                    info!("Entity {:?}: Stuck for too long, requesting new path", entity);
                                    target.clear_path();
                                }
                            }
                        } else {
                            // No slide direction found, stop and wait
                            agent.velocity = Vec3::ZERO;
                            if let Some(ref mut lin_vel) = maybe_linear_velocity {
                                lin_vel.0 = Vec3::ZERO;
                            }
                        }
                    }
                }
            } else {
                // We're at the waypoint, advance to next
                info!(
                    "Entity {:?}: Reached waypoint {:?}, advancing to next (remaining: {})",
                    entity,
                    next_waypoint,
                    target.path.len() - 1
                );
                target.advance_waypoint();

                // For patrol behavior, maintain velocity towards next waypoint
                if matches!(agent.behavior, NavigationBehavior::Patrol) {
                    if let Some(next_waypoint) = target.get_next_waypoint() {
                        // Immediately orient towards next waypoint to maintain smooth movement
                        let to_next = next_waypoint - position.0;
                        if to_next.length() > 0.1 {
                            let direction = to_next.normalize();
                            if direction.is_finite() {
                                agent.velocity = direction * agent.speed * 0.8; // Maintain some velocity
                            }
                        }
                        info!(
                            "Entity {:?}: Advanced to next waypoint: {:?}",
                            entity, next_waypoint
                        );
                    }
                } else if target.is_path_complete() {
                    info!(
                        "Entity {:?}: Path complete, behavior: {:?}, stop_at_dest: {}",
                        entity, agent.behavior, agent.stop_at_destination
                    );
                    if agent.stop_at_destination {
                        agent.velocity *= 0.8; // Gradually slow down
                        if let Some(ref mut lin_vel) = maybe_linear_velocity {
                            lin_vel.0 = agent.velocity;
                        }
                    }
                }
            }
        } else {
            // No target, gradually stop
            agent.velocity *= 0.9;
            if agent.velocity.length() < 0.01 {
                agent.velocity = Vec3::ZERO;
                if let Some(mut lin_vel) = maybe_linear_velocity {
                    lin_vel.0 = Vec3::ZERO;
                }
            } else if let Some(ref mut lin_vel) = maybe_linear_velocity {
                lin_vel.0 = agent.velocity;
            }
        }
    }
}

/// System to refresh paths when navmesh changes
fn refresh_paths_on_navmesh_change(
    mut agents: Query<&mut NavigationTarget, With<NavigationAgent>>,
    navmesh_query: Query<&NavMeshStatus, Changed<NavMeshStatus>>,
) {
    // Check if navmesh was rebuilt
    for status in navmesh_query.iter() {
        if *status == NavMeshStatus::Built {
            // Clear all paths to force re-pathfinding
            for mut target in agents.iter_mut() {
                target.clear_path();
            }
            break;
        }
    }
}

/// System to handle patrol behavior when agents reach their destination
fn handle_patrol_behavior(
    mut agents: Query<(
        Entity,
        &Position,
        &mut NavigationTarget,
        &NavigationAgent,
    )>,
    mut patrol_query: Query<&mut PatrolRoute>,
    navmesh_query: Query<&ManagedNavMesh>,
    navmeshes: Res<Assets<NavMesh>>,
    time: Res<Time>,
) {
    let navmesh = navmesh_query
        .single()
        .ok()
        .and_then(|handle| navmeshes.get(handle));

    if navmesh.is_none() {
        return; // No spam
    }

    let navmesh = navmesh.unwrap();

    for (entity, position, mut target, agent) in agents.iter_mut() {
        if matches!(agent.behavior, NavigationBehavior::Patrol) {
            if let Ok(mut patrol) = patrol_query.get_mut(entity) {
                let current_patrol_target = patrol.get_current_target().unwrap_or(position.0);
                // Use horizontal distance only (X and Z) to avoid height differences from physics
                let horizontal_distance = Vec2::new(position.0.x, position.0.z)
                    .distance(Vec2::new(current_patrol_target.x, current_patrol_target.z));
                let path_complete = target.is_path_complete();

                info!("Entity {:?}: Patrol check - pos: {:?}, target: {:?}, h_distance: {:.2}, threshold: {:.2}, path_complete: {}",
                    entity, position.0, current_patrol_target, horizontal_distance, agent.arrival_threshold, path_complete);
                
                // Simple patrol logic: if close to target and path complete, wait then advance
                if horizontal_distance < agent.arrival_threshold && path_complete && target.last_pathfind_success {
                    patrol.current_wait += time.delta_secs();
                    
                    if patrol.current_wait >= patrol.wait_time {
                        info!("Entity {:?}: Advancing patrol from waypoint {} after {:.1}s wait", entity, patrol.current_index, patrol.current_wait);
                        patrol.advance();
                        patrol.current_wait = 0.0;
                        
                        if let Some(next_target) = patrol.get_current_target() {
                            info!("Entity {:?}: Setting new patrol target: {:?}", entity, next_target);
                            target.destination = next_target;
                            target.clear_path(); // Clear path to trigger new pathfinding
                            info!("Entity {:?}: Cleared path, will compute new path to {:?}", entity, next_target);
                        }
                    }
                } else if path_complete && !target.last_pathfind_success {
                    // Path failed - don't spam pathfinding requests, wait a bit
                    patrol.current_wait += time.delta_secs();
                    if patrol.current_wait >= 2.0 {  // Wait 2 seconds before retrying
                        info!("Entity {:?}: Retrying pathfinding to patrol target {:?} after pathfinding failure", entity, current_patrol_target);
                        patrol.current_wait = 0.0;
                        target.destination = current_patrol_target;
                        target.clear_path(); // Clear path to trigger new pathfinding
                    }
                } else if path_complete && target.last_pathfind_success {
                    // Need path to current target - let normal pathfinding handle it
                    info!("Entity {:?}: Need path to patrol target {:?}, h_distance: {:.2}", entity, current_patrol_target, horizontal_distance);
                    patrol.current_wait = 0.0;
                    target.destination = current_patrol_target;
                    target.clear_path(); // Clear path to trigger new pathfinding
                    info!("Entity {:?}: Cleared path, will compute new path to patrol target {:?}", entity, current_patrol_target);
                }
            }
        }
    }
}

/// System for basic obstacle avoidance and stuck detection
fn obstacle_avoidance_system(
    mut agents: Query<(Entity, &Position, &mut NavigationAgent, &mut NavigationTarget)>,
    obstacles: Query<&Position, (With<NavigationObstacle>, Without<NavigationAgent>)>,
    time: Res<Time>,
) {
    let current_time = time.elapsed_secs();
    
    // Collect agent positions for inter-agent avoidance
    let agent_positions: Vec<(Vec3, f32)> = agents
        .iter()
        .map(|(_, pos, agent, _)| (pos.0, agent.radius))
        .collect();

    for (entity, position, mut agent, mut target) in agents.iter_mut() {
        // Skip obstacle avoidance for patrol entities to prevent movement interference
        if matches!(agent.behavior, NavigationBehavior::Patrol) {
            // For patrol entities, check if waypoint is blocked by obstacle
            if let Some(waypoint) = target.get_next_waypoint() {
                let mut waypoint_blocked = false;
                for obstacle_pos in obstacles.iter() {
                    let distance = waypoint.distance(obstacle_pos.0);
                    if distance < agent.radius + 1.8 {
                        waypoint_blocked = true;
                        break;
                    }
                }
                
                if waypoint_blocked {
                    info!("Entity {:?}: Waypoint {:?} is blocked, skipping to next", entity, waypoint);
                    target.advance_waypoint(); // Skip blocked waypoint
                }
            }
            continue;
        }

        let mut avoidance_force = Vec3::ZERO;

        // Inter-agent avoidance
        for (other_pos, other_radius) in &agent_positions {
            if *other_pos == position.0 {
                continue; // Skip self
            }

            let distance = position.0.distance(*other_pos);
            let min_distance = agent.radius + other_radius + 0.1;

            if distance < min_distance && distance > 0.0 {
                let avoid_direction = (position.0 - *other_pos).normalize();
                let avoid_strength = (min_distance - distance) / min_distance;
                avoidance_force += avoid_direction * avoid_strength * agent.speed * 0.3;
            }
        }

        // Obstacle avoidance force
        for obstacle_pos in obstacles.iter() {
            let distance = position.0.distance(obstacle_pos.0);
            let min_distance = agent.radius + 2.5; // Larger avoidance radius
            
            if distance < min_distance && distance > 0.0 {
                let avoid_direction = (position.0 - obstacle_pos.0).normalize();
                let avoid_strength = (min_distance - distance) / min_distance;
                avoidance_force += avoid_direction * avoid_strength * agent.speed * 0.4;
            }
        }

        // Apply avoidance force
        if avoidance_force.length() > 0.1 {
            agent.velocity += avoidance_force * 0.1; // Gentle avoidance
        }
    }
}

/// Helper function to set a navigation target for an entity
pub fn set_navigation_target(commands: &mut Commands, entity: Entity, destination: Vec3) {
    commands
        .entity(entity)
        .insert(NavigationTarget::new(destination));
}

/// Helper function to make an entity follow another entity
pub fn set_follow_target(commands: &mut Commands, follower: Entity, target: Entity, distance: f32) {
    let mut nav_target = NavigationTarget::new(Vec3::ZERO);
    nav_target.follow_target = Some(target);

    commands.entity(follower).insert((
        NavigationAgent::new(3.0).with_behavior(NavigationBehavior::Follow { target, distance }),
        nav_target,
    ));
}

/// Helper function to set up patrol behavior
pub fn setup_patrol(
    commands: &mut Commands,
    entity: Entity,
    patrol_points: Vec<Vec3>,
    speed: f32,
    ping_pong: bool,
) {
    let patrol_route = if ping_pong {
        PatrolRoute::new(patrol_points.clone()).ping_pong(2.0)
    } else {
        PatrolRoute::new(patrol_points.clone())
    };

    let initial_target = patrol_route.get_current_target().unwrap_or(Vec3::ZERO);

    commands.entity(entity).insert((
        NavigationAgent::bot(speed).with_behavior(NavigationBehavior::Patrol),
        NavigationTarget::new(initial_target),
        patrol_route,
    ));
}

/// Helper function to add navigation obstacle
pub fn add_navigation_obstacle(commands: &mut Commands, entity: Entity) {
    commands.entity(entity).insert(NavigationObstacle);
}

/// Validate and potentially adjust spawn position to avoid obstacles
pub fn validate_spawn_position(
    position: Vec3,
    obstacles: &Query<&Position, With<NavigationObstacle>>,
    agent_radius: f32,
) -> Vec3 {
    let mut adjusted_position = position;
    let min_distance = agent_radius + 2.0; // Safe distance from obstacles
    
    // Check if current position is too close to any obstacle
    for obstacle_pos in obstacles.iter() {
        let distance = adjusted_position.distance(obstacle_pos.0);
        if distance < min_distance {
            // Find a safe position by moving away from the obstacle
            let away_direction = (adjusted_position - obstacle_pos.0).normalize();
            if away_direction.is_finite() {
                adjusted_position = obstacle_pos.0 + away_direction * min_distance;
                info!("Adjusted spawn position from {:?} to {:?} to avoid obstacle at {:?}", 
                    position, adjusted_position, obstacle_pos.0);
            }
        }
    }
    
    // Ensure the position is at the correct height level
    adjusted_position.y = 1.0;
    adjusted_position
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::app::App;

    #[test]
    fn test_navigation_agent_creation() {
        let agent = NavigationAgent::new(5.0);
        assert_eq!(agent.speed, 5.0);
        assert_eq!(agent.behavior, NavigationBehavior::Direct);

        let bot = NavigationAgent::bot(3.0);
        assert_eq!(bot.speed, 3.0);
        assert!(matches!(bot.behavior, NavigationBehavior::Patrol));
        assert_eq!(bot.arrival_threshold, 1.0);
        assert!(!bot.stop_at_destination);
    }

    #[test]
    fn test_navigation_target() {
        let destination = Vec3::new(10.0, 0.0, 5.0);
        let target = NavigationTarget::new(destination);

        assert_eq!(target.destination, destination);
        assert!(target.is_path_complete());
        assert!(target.get_next_waypoint().is_none());

        // Test with path
        let waypoints = vec![
            Vec3::new(1.0, 0.0, 1.0),
            Vec3::new(2.0, 0.0, 2.0),
            Vec3::new(3.0, 0.0, 3.0),
        ];
        let mut target_with_path = NavigationTarget::with_path(destination, waypoints);

        assert!(!target_with_path.is_path_complete());
        assert_eq!(
            target_with_path.get_next_waypoint(),
            Some(Vec3::new(1.0, 0.0, 1.0))
        );

        target_with_path.advance_waypoint();
        assert_eq!(
            target_with_path.get_next_waypoint(),
            Some(Vec3::new(2.0, 0.0, 2.0))
        );

        target_with_path.advance_waypoint();
        target_with_path.advance_waypoint();
        assert!(target_with_path.is_path_complete());
    }

    #[test]
    fn test_patrol_route() {
        let points = vec![
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(10.0, 0.0, 0.0),
            Vec3::new(10.0, 0.0, 10.0),
        ];

        // Test loop patrol
        let mut loop_patrol = PatrolRoute::new(points.clone());
        assert_eq!(loop_patrol.get_current_target(), Some(points[0]));

        loop_patrol.advance();
        assert_eq!(loop_patrol.get_current_target(), Some(points[1]));

        loop_patrol.advance();
        assert_eq!(loop_patrol.get_current_target(), Some(points[2]));

        loop_patrol.advance(); // Should wrap around
        assert_eq!(loop_patrol.get_current_target(), Some(points[0]));

        // Test ping-pong patrol
        let mut ping_pong_patrol = PatrolRoute::new(points.clone()).ping_pong(1.0);
        assert!(ping_pong_patrol.ping_pong);
        assert!(ping_pong_patrol.forward);

        ping_pong_patrol.advance();
        assert_eq!(ping_pong_patrol.current_index, 1);
        assert!(ping_pong_patrol.forward);

        ping_pong_patrol.advance();
        assert_eq!(ping_pong_patrol.current_index, 2);
        assert!(!ping_pong_patrol.forward); // Should reverse at end

        ping_pong_patrol.advance();
        assert_eq!(ping_pong_patrol.current_index, 1);
        assert!(!ping_pong_patrol.forward);

        ping_pong_patrol.advance();
        assert_eq!(ping_pong_patrol.current_index, 0);
        assert!(ping_pong_patrol.forward); // Should reverse at start
    }

    #[test]
    fn test_navigation_system_integration() {
        let mut app = App::new();
        app.add_plugins((
            bevy::time::TimePlugin,
            bevy::asset::AssetPlugin::default(),
            NavigationPlugin,
        ));

        // Create a test entity with navigation components
        let entity = app
            .world_mut()
            .spawn((
                Position::new(Vec3::ZERO),
                NavigationAgent::bot(2.0),
                NavigationTarget::new(Vec3::new(5.0, 0.0, 0.0)),
            ))
            .id();

        // Verify components exist
        assert!(app.world().get::<NavigationAgent>(entity).is_some());
        assert!(app.world().get::<NavigationTarget>(entity).is_some());

        // Test that systems are registered
        // Note: More detailed system testing would require running the app
    }

    #[test]
    fn test_patrol_setup_helper() {
        let mut app = App::new();
        app.add_plugins((
            bevy::time::TimePlugin,
            bevy::asset::AssetPlugin::default(),
            NavigationPlugin,
        ));

        let entity = app.world_mut().spawn((Position::new(Vec3::ZERO),)).id();

        let patrol_points = vec![
            Vec3::new(0.0, 1.0, 0.0),
            Vec3::new(10.0, 1.0, 0.0),
            Vec3::new(10.0, 1.0, 10.0),
        ];

        setup_patrol(
            &mut app.world_mut().commands(),
            entity,
            patrol_points.clone(),
            3.0,
            false,
        );

        // Apply commands
        app.update();

        // Verify components were added
        let agent = app.world().get::<NavigationAgent>(entity);
        assert!(agent.is_some());
        assert_eq!(agent.unwrap().speed, 3.0);

        let patrol = app.world().get::<PatrolRoute>(entity);
        assert!(patrol.is_some());
        assert_eq!(patrol.unwrap().points, patrol_points);

        let target = app.world().get::<NavigationTarget>(entity);
        assert!(target.is_some());
        assert_eq!(target.unwrap().destination, patrol_points[0]);
    }

    #[test]
    fn test_find_nearest_in_mesh_fallback() {
        // This test doesn't use a real navmesh, but tests the sampling logic structure
        let center = Vec3::new(5.0, 0.0, 5.0);
        let max_radius = 3.0;
        let step = 0.5;

        // Test that the function generates reasonable sample points
        // (We can't test the actual navmesh functionality without a real mesh)
        let mut sample_count = 0;
        let mut radius = step;
        while radius <= max_radius {
            let count = ((radius as f32 / step as f32).ceil() as i32) * 8;
            for i in 0..count {
                let t = (i as f32) / (count as f32);
                let angle = t * std::f32::consts::TAU;
                let dx = angle.cos() * radius;
                let dz = angle.sin() * radius;
                let sample = Vec3::new(center.x + dx, center.y, center.z + dz);

                // Verify samples are within expected radius
                let distance = center.distance(sample);
                assert!(
                    (distance - radius).abs() < 0.1,
                    "Sample should be at radius {}, got distance {}",
                    radius,
                    distance
                );
                sample_count += 1;
            }
            radius += step;
        }

        assert!(sample_count > 0, "Should generate sample points");
    }

    #[test]
    fn test_navigation_behavior_variants() {
        let direct = NavigationBehavior::Direct;
        let patrol = NavigationBehavior::Patrol;
        let follow = NavigationBehavior::Follow {
            target: Entity::from_bits(42),
            distance: 2.0,
        };

        // Test pattern matching
        match direct {
            NavigationBehavior::Direct => (),
            _ => panic!("Should match Direct"),
        }

        match patrol {
            NavigationBehavior::Patrol => (),
            _ => panic!("Should match Patrol"),
        }

        match follow {
            NavigationBehavior::Follow { target, distance } => {
                assert_eq!(target, Entity::from_bits(42));
                assert_eq!(distance, 2.0);
            }
            _ => panic!("Should match Follow"),
        }
    }
}
