use avian3d::prelude::{LinearVelocity, Position};
use bevy::prelude::*;

/// Marker component for navigation mesh entity
#[derive(Component, Debug, Clone)]
pub struct NavigationMeshMarker;

/// Navigation agent component that stores pathfinding state
#[derive(Component, Debug, Clone)]
pub struct NavigationAgent {
    /// Current path the agent is following
    pub current_path: Vec<Vec3>,
    /// Current target index in the path
    pub current_target_index: usize,
    /// Movement speed of the agent
    pub speed: f32,
    /// Minimum distance to consider a waypoint reached
    pub waypoint_threshold: f32,
    /// Current destination
    pub destination: Option<Vec3>,
    /// Whether the agent should continuously seek new paths
    pub auto_repath: bool,
    /// Time since last path calculation
    pub last_repath_time: f32,
    /// How often to recalculate path (in seconds)
    pub repath_interval: f32,
}

impl Default for NavigationAgent {
    fn default() -> Self {
        Self {
            current_path: Vec::new(),
            current_target_index: 0,
            speed: 3.0,
            waypoint_threshold: 0.5,
            destination: None,
            auto_repath: false,
            last_repath_time: 0.0,
            repath_interval: 1.0,
        }
    }
}

impl NavigationAgent {
    pub fn new(speed: f32) -> Self {
        Self {
            speed,
            ..Default::default()
        }
    }

    pub fn with_auto_repath(mut self, interval: f32) -> Self {
        self.auto_repath = true;
        self.repath_interval = interval;
        self
    }

    /// Set a new destination for the agent
    pub fn set_destination(&mut self, destination: Vec3) {
        self.destination = Some(destination);
        self.current_path.clear();
        self.current_target_index = 0;
    }

    /// Get the current target position
    pub fn current_target(&self) -> Option<Vec3> {
        self.current_path.get(self.current_target_index).copied()
    }

    /// Check if the agent has reached its destination
    pub fn has_reached_destination(&self) -> bool {
        self.current_path.is_empty() || self.current_target_index >= self.current_path.len()
    }

    /// Advance to the next waypoint in the path
    pub fn advance_waypoint(&mut self) {
        if self.current_target_index < self.current_path.len().saturating_sub(1) {
            self.current_target_index += 1;
        }
    }
}

/// Component for entities that should seek and follow target entities
#[derive(Component, Debug, Clone)]
pub struct TargetSeeker {
    /// The entity to follow/seek
    pub target: Option<Entity>,
    /// How often to update the target destination
    pub update_interval: f32,
    /// Time since last target update
    pub last_update: f32,
    /// Offset from target position
    pub offset: Vec3,
}

impl Default for TargetSeeker {
    fn default() -> Self {
        Self {
            target: None,
            update_interval: 0.5,
            last_update: 0.0,
            offset: Vec3::ZERO,
        }
    }
}

impl TargetSeeker {
    pub fn new(target: Entity) -> Self {
        Self {
            target: Some(target),
            ..Default::default()
        }
    }

    pub fn with_offset(mut self, offset: Vec3) -> Self {
        self.offset = offset;
        self
    }

    pub fn with_update_interval(mut self, interval: f32) -> Self {
        self.update_interval = interval;
        self
    }
}

/// System to update target seeking behavior
pub fn update_target_seekers(
    mut seeker_query: Query<(&mut NavigationAgent, &mut TargetSeeker)>,
    target_query: Query<&Position>,
    time: Res<Time>,
) {
    for (mut agent, mut seeker) in seeker_query.iter_mut() {
        seeker.last_update += time.delta_secs();

        if seeker.last_update < seeker.update_interval {
            continue;
        }

        let Some(target_entity) = seeker.target else {
            continue;
        };

        if let Ok(target_position) = target_query.get(target_entity) {
            let destination = target_position.0 + seeker.offset;
            agent.set_destination(destination);
            seeker.last_update = 0.0;
        }
    }
}

/// System to move navigation agents along their calculated paths
pub fn move_navigation_agents(
    mut agent_query: Query<(&Position, &mut LinearVelocity, &mut NavigationAgent)>,
) {
    for (position, mut velocity, mut agent) in agent_query.iter_mut() {
        if agent.has_reached_destination() {
            velocity.0 = Vec3::ZERO;
            continue;
        }

        let Some(target) = agent.current_target() else {
            velocity.0 = Vec3::ZERO;
            continue;
        };

        let current_pos = position.0;
        let direction = target - current_pos;
        let distance = direction.length();

        // Check if we've reached the current waypoint
        if distance <= agent.waypoint_threshold {
            agent.advance_waypoint();

            // If we've reached the final waypoint, stop
            if agent.has_reached_destination() {
                velocity.0 = Vec3::ZERO;
                debug!("Agent reached final destination");
                continue;
            }

            // Get the next target
            let Some(next_target) = agent.current_target() else {
                velocity.0 = Vec3::ZERO;
                continue;
            };

            let next_direction = next_target - current_pos;
            velocity.0 = next_direction.normalize() * agent.speed;
        } else {
            // Move towards current waypoint
            velocity.0 = direction.normalize() * agent.speed;
        }
    }
}

/// Simple pathfinding system that moves agents in straight lines to their destination
/// This is a fallback when navmesh is not available
pub fn simple_pathfinding(
    mut agent_query: Query<(&Position, &mut NavigationAgent)>,
    time: Res<Time>,
) {
    for (_position, mut agent) in agent_query.iter_mut() {
        agent.last_repath_time += time.delta_secs();

        // Check if we need to calculate a new path
        let needs_new_path = agent.destination.is_some()
            && (agent.current_path.is_empty()
                || (agent.auto_repath && agent.last_repath_time >= agent.repath_interval));

        if !needs_new_path {
            continue;
        }

        let Some(destination) = agent.destination else {
            continue;
        };

        // Simple straight-line path
        agent.current_path = vec![destination];
        agent.current_target_index = 0;
        agent.last_repath_time = 0.0;
    }
}

/// Helper function to add a navigation agent with specified speed to an entity
pub fn add_navigation_agent_with_speed(commands: &mut Commands, entity: Entity, speed: f32) {
    commands.entity(entity).insert((
        NavigationAgent::new(speed).with_auto_repath(2.0),
        LinearVelocity::default(),
    ));

    info!(
        "Added navigation agent with speed {} to entity {:?}",
        speed, entity
    );
}
