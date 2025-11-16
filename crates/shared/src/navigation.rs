use avian3d::prelude::*;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use vleue_navigator::prelude::*;

/// Plugin that handles all navigation and pathfinding functionality
pub struct NavigationPlugin;

impl Plugin for NavigationPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(VleueNavigatorPlugin)
            .add_plugins(NavmeshUpdaterPlugin::<Collider, NavigationObstacle>::default())
            .add_systems(
                Update,
                (
                    process_navigation_requests,
                    update_pathfinding,
                    move_navigation_agents,
                    refresh_paths_on_navmesh_change,
                    handle_path_completion,
                )
                    .chain(),
            )
            .add_systems(
                FixedUpdate,
                (obstacle_avoidance_system, formation_movement_system),
            );
    }
}

/// Component that makes an entity capable of navigation and pathfinding
/// Attach this to any player or bot to enable AI movement
#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct NavigationAgent {
    /// Movement speed in units per second
    pub speed: f32,
    /// Radius for collision avoidance
    pub radius: f32,
    /// How close the agent needs to get to a waypoint before moving to the next one
    pub arrival_threshold: f32,
    /// Maximum acceleration for smooth movement
    pub max_acceleration: f32,
    /// Current velocity for smooth movement
    pub velocity: Vec3,
    /// Whether the agent should stop at the final destination
    pub stop_at_destination: bool,
    /// Agent behavior type
    pub behavior: NavigationBehavior,
    /// Priority for path planning (higher = more important)
    pub priority: u8,
}

impl NavigationAgent {
    pub fn new(speed: f32) -> Self {
        Self {
            speed,
            radius: 0.5,
            arrival_threshold: 0.1,
            max_acceleration: 10.0,
            velocity: Vec3::ZERO,
            stop_at_destination: true,
            behavior: NavigationBehavior::Direct,
            priority: 1,
        }
    }

    pub fn player(speed: f32) -> Self {
        Self {
            speed,
            radius: 0.4,
            arrival_threshold: 0.2,
            max_acceleration: 15.0,
            velocity: Vec3::ZERO,
            stop_at_destination: true,
            behavior: NavigationBehavior::Direct,
            priority: 10, // Players have high priority
        }
    }

    pub fn bot(speed: f32) -> Self {
        Self {
            speed,
            radius: 0.5,
            arrival_threshold: 0.15,
            max_acceleration: 8.0,
            velocity: Vec3::ZERO,
            stop_at_destination: true,
            behavior: NavigationBehavior::Patrol,
            priority: 5,
        }
    }

    pub fn with_radius(mut self, radius: f32) -> Self {
        self.radius = radius;
        self
    }

    pub fn with_behavior(mut self, behavior: NavigationBehavior) -> Self {
        self.behavior = behavior;
        self
    }
}

/// Different navigation behaviors
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum NavigationBehavior {
    /// Direct movement to target
    Direct,
    /// Patrol between multiple points
    Patrol,
    /// Follow another entity
    Follow { target: Entity, distance: f32 },
    /// Flee from a target
    Flee { target: Entity, distance: f32 },
    /// Formation movement (stay in formation with group)
    Formation { leader: Entity, offset: Vec3 },
}

/// Component for the current navigation target and path
#[derive(Component, Clone, Debug)]
pub struct NavigationTarget {
    /// Current destination
    pub destination: Vec3,
    /// Path to follow (queue of waypoints)
    pub path: VecDeque<Vec3>,
    /// Current waypoint index
    pub current_waypoint: usize,
    /// Whether pathfinding is in progress
    pub pathfinding_in_progress: bool,
    /// Time when pathfinding was requested
    pub pathfind_request_time: f32,
    /// Optional entity to follow
    pub follow_target: Option<Entity>,
}

impl NavigationTarget {
    pub fn new(destination: Vec3) -> Self {
        Self {
            destination,
            path: VecDeque::new(),
            current_waypoint: 0,
            pathfinding_in_progress: false,
            pathfind_request_time: 0.0,
            follow_target: None,
        }
    }

    pub fn with_path(destination: Vec3, path: Vec<Vec3>) -> Self {
        Self {
            destination,
            path: path.into(),
            current_waypoint: 0,
            pathfinding_in_progress: false,
            pathfind_request_time: 0.0,
            follow_target: None,
        }
    }

    pub fn get_next_waypoint(&self) -> Option<Vec3> {
        self.path.front().copied()
    }

    pub fn advance_waypoint(&mut self) {
        if !self.path.is_empty() {
            self.path.pop_front();
            self.current_waypoint += 1;
        }
    }

    pub fn is_path_complete(&self) -> bool {
        self.path.is_empty()
    }

    pub fn clear_path(&mut self) {
        self.path.clear();
        self.current_waypoint = 0;
        self.pathfinding_in_progress = false;
    }
}

/// Component for entities that should block navigation (obstacles)
#[derive(Component, Clone, Debug)]
pub struct NavigationObstacle;

/// Component for patrol routes
#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct PatrolRoute {
    /// List of patrol points
    pub points: Vec<Vec3>,
    /// Current patrol index
    pub current_index: usize,
    /// Whether to reverse direction at the end (ping-pong) or loop
    pub ping_pong: bool,
    /// Direction for ping-pong movement
    pub forward: bool,
    /// Time to wait at each patrol point
    pub wait_time: f32,
    /// Current wait timer
    pub current_wait: f32,
}

impl PatrolRoute {
    pub fn new(points: Vec<Vec3>) -> Self {
        Self {
            points,
            current_index: 0,
            ping_pong: false,
            forward: true,
            wait_time: 2.0,
            current_wait: 0.0,
        }
    }

    pub fn ping_pong(mut self, wait_time: f32) -> Self {
        self.ping_pong = true;
        self.wait_time = wait_time;
        self
    }

    pub fn get_current_target(&self) -> Option<Vec3> {
        self.points.get(self.current_index).copied()
    }

    pub fn advance(&mut self) {
        if self.points.is_empty() {
            return;
        }

        if self.ping_pong {
            if self.forward {
                self.current_index += 1;
                if self.current_index >= self.points.len() - 1 {
                    self.forward = false;
                }
            } else {
                if self.current_index > 0 {
                    self.current_index -= 1;
                }
                if self.current_index == 0 {
                    self.forward = true;
                }
            }
        } else {
            self.current_index = (self.current_index + 1) % self.points.len();
        }
        self.current_wait = 0.0;
    }
}

/// Component for formation movement
#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct FormationMember {
    pub leader: Entity,
    pub offset: Vec3,
    pub formation_type: FormationType,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum FormationType {
    Line,
    Column,
    Wedge,
    Box,
}

/// System to process navigation requests and start pathfinding
fn process_navigation_requests(
    mut agents: Query<
        (Entity, &Transform, &mut NavigationTarget, &NavigationAgent),
        Changed<NavigationTarget>,
    >,
    navmesh_query: Query<&ManagedNavMesh>,
    navmeshes: Res<Assets<NavMesh>>,
    time: Res<Time>,
) {
    let current_time = time.elapsed_secs();

    for (entity, transform, mut target, _agent) in agents.iter_mut() {
        if target.pathfinding_in_progress {
            continue;
        }

        // Check if we need to find a new path
        let needs_new_path =
            target.path.is_empty() || (current_time - target.pathfind_request_time) > 1.0; // Recompute path every second

        if needs_new_path {
            if let Ok(navmesh_handle) = navmesh_query.single() {
                if let Some(navmesh) = navmeshes.get(navmesh_handle) {
                    let start_pos = transform.translation;
                    let end_pos = target.destination;

                    // Check if both positions are valid
                    if navmesh.transformed_is_in_mesh(start_pos)
                        && navmesh.transformed_is_in_mesh(end_pos)
                    {
                        if let Some(path_result) = navmesh.transformed_path(start_pos, end_pos) {
                            let mut new_path = VecDeque::new();

                            // Convert path to waypoints, skipping the first point (current position)
                            for point in path_result.path.iter().skip(1) {
                                new_path.push_back(*point);
                            }

                            target.path = new_path;
                            target.current_waypoint = 0;
                            target.pathfind_request_time = current_time;

                            info!(
                                "Found path for entity {:?} with {} waypoints",
                                entity,
                                target.path.len()
                            );
                        } else {
                            warn!(
                                "No path found for entity {:?} from {:?} to {:?}",
                                entity, start_pos, end_pos
                            );
                        }
                    } else {
                        warn!("Invalid start or end position for entity {:?}", entity);
                    }
                }
            }
        }
    }
}

/// System to update pathfinding for agents with targets
fn update_pathfinding(
    mut agents: Query<(Entity, &Transform, &mut NavigationTarget, &NavigationAgent)>,
    follow_targets: Query<&Transform, Without<NavigationTarget>>,
    _time: Res<Time>,
) {
    for (_entity, transform, mut target, agent) in agents.iter_mut() {
        // Handle follow behavior
        if let Some(follow_entity) = target.follow_target {
            if let Ok(follow_transform) = follow_targets.get(follow_entity) {
                let distance = transform.translation.distance(follow_transform.translation);

                // Update destination if target moved significantly
                if distance > 2.0 {
                    target.destination = follow_transform.translation;
                    target.clear_path(); // Force re-pathfinding
                }
            }
        }

        // Handle patrol behavior
        if matches!(agent.behavior, NavigationBehavior::Patrol) {
            // Patrol logic will be handled by patrol_system if PatrolRoute component exists
        }
    }
}

/// System to move navigation agents along their paths
fn move_navigation_agents(
    mut agents: Query<(&mut Transform, &mut NavigationTarget, &mut NavigationAgent)>,
    time: Res<Time>,
) {
    let delta_time = time.delta_secs();

    for (mut transform, mut target, mut agent) in agents.iter_mut() {
        if let Some(next_waypoint) = target.get_next_waypoint() {
            let current_pos = transform.translation;
            let direction = (next_waypoint - current_pos).normalize();

            if direction.is_finite() {
                // Smooth acceleration/deceleration
                let desired_velocity = direction * agent.speed;
                let velocity_change = desired_velocity - agent.velocity;
                let max_velocity_change = agent.max_acceleration * delta_time;

                let velocity_change_clamped = velocity_change.clamp_length_max(max_velocity_change);
                agent.velocity += velocity_change_clamped;

                // Apply movement
                let movement = agent.velocity * delta_time;
                transform.translation += movement;

                // Rotate to face movement direction
                if agent.velocity.length() > 0.1 {
                    let forward = agent.velocity.normalize();
                    if forward.y.abs() < 0.9 {
                        // Avoid gimbal lock
                        transform.look_to(forward, Vec3::Y);
                    }
                }

                // Check if we've reached the waypoint
                let distance_to_waypoint = current_pos.distance(next_waypoint);
                if distance_to_waypoint < agent.arrival_threshold {
                    target.advance_waypoint();

                    // If no more waypoints and we should stop at destination
                    if target.is_path_complete() && agent.stop_at_destination {
                        agent.velocity = Vec3::ZERO;
                    }
                }
            }
        } else if agent.stop_at_destination {
            // No target, gradually stop
            agent.velocity *= 0.9;
            if agent.velocity.length() < 0.01 {
                agent.velocity = Vec3::ZERO;
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

/// System to handle path completion and behavior-specific actions
fn handle_path_completion(
    mut agents: Query<(Entity, &Transform, &mut NavigationTarget, &NavigationAgent)>,
    mut patrol_query: Query<&mut PatrolRoute>,
    time: Res<Time>,
) {
    for (entity, _transform, mut target, agent) in agents.iter_mut() {
        if target.is_path_complete() {
            match &agent.behavior {
                NavigationBehavior::Patrol => {
                    if let Ok(mut patrol) = patrol_query.get_mut(entity) {
                        patrol.current_wait += time.delta_secs();

                        if patrol.current_wait >= patrol.wait_time {
                            patrol.advance();
                            if let Some(next_target) = patrol.get_current_target() {
                                target.destination = next_target;
                                target.clear_path();
                            }
                        }
                    }
                }
                NavigationBehavior::Follow {
                    target: _follow_entity,
                    distance: _,
                } => {
                    // Following behavior is handled in update_pathfinding
                }
                NavigationBehavior::Flee {
                    target: _flee_entity,
                    distance: _,
                } => {
                    // TODO: Implement flee behavior
                }
                _ => {}
            }
        }
    }
}

/// System for basic obstacle avoidance
fn obstacle_avoidance_system(mut agents: Query<(&Transform, &mut NavigationAgent)>) {
    // Simple obstacle avoidance using agent positions
    let agent_positions: Vec<(Vec3, f32)> = agents
        .iter()
        .map(|(transform, agent)| (transform.translation, agent.radius))
        .collect();

    for (transform, mut agent) in agents.iter_mut() {
        let mut avoidance_force = Vec3::ZERO;

        for (other_pos, other_radius) in &agent_positions {
            if *other_pos == transform.translation {
                continue; // Skip self
            }

            let distance = transform.translation.distance(*other_pos);
            let min_distance = agent.radius + other_radius + 0.1;

            if distance < min_distance && distance > 0.0 {
                let avoid_direction = (transform.translation - *other_pos).normalize();
                let avoid_strength = (min_distance - distance) / min_distance;
                avoidance_force += avoid_direction * avoid_strength * agent.speed * 0.5;
            }
        }

        // Apply avoidance force
        if avoidance_force.length() > 0.1 {
            agent.velocity += avoidance_force * 0.1; // Gentle avoidance
        }
    }
}

/// System for formation movement
fn formation_movement_system(
    mut members: Query<(&mut NavigationTarget, &FormationMember)>,
    leaders: Query<&Transform, Without<FormationMember>>,
) {
    for (mut target, formation) in members.iter_mut() {
        if let Ok(leader_transform) = leaders.get(formation.leader) {
            let formation_position = leader_transform.translation + formation.offset;

            // Update target to maintain formation
            let distance_to_formation = target.destination.distance(formation_position);
            if distance_to_formation > 1.0 {
                target.destination = formation_position;
                target.clear_path();
            }
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
    commands.entity(follower).insert((
        NavigationAgent::new(3.0).with_behavior(NavigationBehavior::Follow { target, distance }),
        NavigationTarget {
            destination: Vec3::ZERO,
            path: VecDeque::new(),
            current_waypoint: 0,
            pathfinding_in_progress: false,
            pathfind_request_time: 0.0,
            follow_target: Some(target),
        },
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

/// Helper function to create a formation
pub fn setup_formation(
    commands: &mut Commands,
    leader: Entity,
    members: Vec<Entity>,
    formation_type: FormationType,
) {
    let offsets = match formation_type {
        FormationType::Line => {
            let spacing = 2.0;
            let mut offsets = Vec::new();
            for i in 0..members.len() {
                let offset = Vec3::new(
                    (i as f32 - (members.len() as f32 - 1.0) / 2.0) * spacing,
                    0.0,
                    -2.0,
                );
                offsets.push(offset);
            }
            offsets
        }
        FormationType::Column => {
            let spacing = 1.5;
            members
                .iter()
                .enumerate()
                .map(|(i, _)| Vec3::new(0.0, 0.0, -(i as f32 + 1.0) * spacing))
                .collect()
        }
        FormationType::Wedge => {
            let mut offsets = Vec::new();
            for i in 0..members.len() {
                let side = if i % 2 == 0 { 1.0 } else { -1.0 };
                let depth = (i / 2 + 1) as f32;
                offsets.push(Vec3::new(side * depth, 0.0, -depth));
            }
            offsets
        }
        FormationType::Box => {
            let mut offsets = Vec::new();
            let side_length = (members.len() as f32).sqrt().ceil() as usize;
            for i in 0..members.len() {
                let x = (i % side_length) as f32 - (side_length as f32 - 1.0) / 2.0;
                let z = -((i / side_length) as f32) - 1.0;
                offsets.push(Vec3::new(x * 1.5, 0.0, z * 1.5));
            }
            offsets
        }
    };

    for (member, offset) in members.iter().zip(offsets.iter()) {
        commands.entity(*member).insert((
            NavigationAgent::new(3.0).with_behavior(NavigationBehavior::Formation {
                leader,
                offset: *offset,
            }),
            NavigationTarget::new(Vec3::ZERO),
            FormationMember {
                leader,
                offset: *offset,
                formation_type: formation_type.clone(),
            },
        ));
    }
}
