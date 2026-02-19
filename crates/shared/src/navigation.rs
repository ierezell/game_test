use avian3d::prelude::*;
use bevy::prelude::*;
use lightyear::prelude::{InterpolationTarget, NetworkTarget, Replicate};
use serde::{Deserialize, Serialize};

#[derive(Component, Clone, Debug)]
pub struct NavigationObstacle;

pub struct NavigationPlugin;

impl Plugin for NavigationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (patrol_system, movement_system));
    }
}

#[derive(Component, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SimpleNavigationAgent {
    pub speed: f32,
    pub arrival_threshold: f32,
    pub current_target: Option<Vec3>,
}

impl SimpleNavigationAgent {
    pub fn new(speed: f32) -> Self {
        Self {
            speed,
            arrival_threshold: 1.0,
            current_target: None,
        }
    }

    pub fn bot() -> Self {
        Self {
            speed: 3.0,
            arrival_threshold: 1.5,
            current_target: None,
        }
    }
}

/// Simple patrol state
#[derive(Component, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PatrolState {
    pub current_target_index: usize,
    pub wait_timer: f32,
    pub wait_duration: f32,
    pub forward: bool,
}

impl Default for PatrolState {
    fn default() -> Self {
        Self {
            current_target_index: 0,
            wait_timer: 0.0,
            wait_duration: 2.0,
            forward: true,
        }
    }
}

/// Simple patrol route component
#[derive(Component, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PatrolRoute {
    pub points: Vec<Vec3>,
    pub ping_pong: bool,
}

impl PatrolRoute {
    pub fn new(points: Vec<Vec3>) -> Self {
        Self {
            points,
            ping_pong: true,
        }
    }

    pub fn get_next_target(
        &self,
        current_index: usize,
        forward: &mut bool,
    ) -> Option<(Vec3, usize)> {
        if self.points.is_empty() || self.points.len() == 1 {
            return None; // No movement possible with 0 or 1 points
        }

        let mut next_index = current_index;

        if self.ping_pong {
            if *forward {
                next_index += 1;
                if next_index >= self.points.len() {
                    next_index = self.points.len().saturating_sub(2);
                    *forward = false;
                }
            } else if next_index > 0 {
                next_index -= 1;
            } else {
                next_index = 1.min(self.points.len() - 1);
                *forward = true;
            }
        } else {
            next_index = (next_index + 1) % self.points.len();
        }

        self.points.get(next_index).map(|&pos| (pos, next_index))
    }
}

/// Simple patrol system that manages patrol behavior
fn patrol_system(
    mut agents: Query<(
        Entity,
        &mut SimpleNavigationAgent,
        &mut PatrolState,
        &PatrolRoute,
        &Position,
    )>,
    time: Res<Time>,
) {
    for (entity, mut nav_agent, mut patrol_state, patrol_route, position) in agents.iter_mut() {
        // Update wait timer
        patrol_state.wait_timer += time.delta_secs();

        // Check if we've reached the current target
        let current_target = patrol_route.points.get(patrol_state.current_target_index);
        let reached_target = if let Some(target_pos) = current_target {
            let distance = Vec2::new(position.0.x, position.0.z)
                .distance(Vec2::new(target_pos.x, target_pos.z));
            distance < nav_agent.arrival_threshold
        } else {
            false
        };

        // If we've reached the target and waited long enough, move to next target
        if reached_target && patrol_state.wait_timer >= patrol_state.wait_duration {
            if let Some((next_target, next_index)) = patrol_route
                .get_next_target(patrol_state.current_target_index, &mut patrol_state.forward)
            {
                debug!(
                    "Entity {:?}: Moving to next patrol target: {:?} (index {})",
                    entity, next_target, next_index
                );

                nav_agent.current_target = Some(next_target);
                patrol_state.current_target_index = next_index;
                patrol_state.wait_timer = 0.0;
            }
        } else if !reached_target {
            // Still moving towards target, reset wait timer
            patrol_state.wait_timer = 0.0;
        }
    }
}

/// Simple movement system for navigation agents using Avian3D physics
fn movement_system(
    mut agents: Query<(&mut Position, &mut Rotation, &SimpleNavigationAgent)>,
    time: Res<Time>,
) {
    for (mut position, mut rotation, nav_agent) in agents.iter_mut() {
        if let Some(target) = nav_agent.current_target {
            let current_pos = position.0;
            let direction = (target - current_pos).normalize();

            if direction.is_finite() {
                let movement = direction * nav_agent.speed * time.delta_secs();
                let new_position = current_pos + movement;

                // Update position (keep Y stable)
                position.0.x = new_position.x;
                position.0.z = new_position.z;

                // Rotate to face movement direction
                if direction.length() > 0.01 {
                    let look_direction = Vec3::new(direction.x, 0.0, direction.z);
                    if look_direction.length() > 0.01 {
                        let target_rotation =
                            Quat::from_rotation_y(look_direction.x.atan2(look_direction.z));
                        rotation.0 = target_rotation;
                    }
                }
            }
        }
    }
}

/// Helper function to set up patrol behavior
pub fn setup_patrol(commands: &mut Commands, entity: Entity, patrol_points: Vec<Vec3>, speed: f32) {
    let patrol_route = PatrolRoute::new(patrol_points.clone());
    let initial_target = patrol_route.points.first().copied();

    let mut nav_agent = SimpleNavigationAgent::new(speed);
    nav_agent.current_target = initial_target;

    commands.entity(entity).insert((
        nav_agent,
        PatrolState::default(),
        patrol_route,
        Replicate::to_clients(NetworkTarget::All),
        // Add interpolation for smooth NPC movement on clients
        InterpolationTarget::to_clients(NetworkTarget::All),
    ));

    info!(
        "Set up patrol for entity {:?} with {} points, initial target: {:?}",
        entity,
        patrol_points.len(),
        initial_target
    );
}

pub fn validate_spawn_position(
    position: Vec3,
    obstacles: &Query<&Position, With<NavigationObstacle>>,
    agent_radius: f32,
) -> Vec3 {
    let mut adjusted_position = position;
    let min_distance = agent_radius + 2.0;

    for obstacle_pos in obstacles.iter() {
        let distance = adjusted_position.distance(obstacle_pos.0);
        if distance < min_distance {
            let away_direction = (adjusted_position - obstacle_pos.0).normalize();
            if away_direction.is_finite() {
                adjusted_position = obstacle_pos.0 + away_direction * min_distance;
            }
        }
    }

    adjusted_position.y = 1.0;
    adjusted_position
}

#[cfg(test)]
mod tests {
    use super::{NavigationObstacle, PatrolRoute, validate_spawn_position};
    use avian3d::prelude::Position;
    use bevy::prelude::{App, Query, Resource, Vec3, With};

    #[derive(Resource, Default)]
    struct AdjustedSpawn(pub Option<Vec3>);

    fn compute_adjusted_spawn(
        obstacles: Query<&Position, With<NavigationObstacle>>,
        mut adjusted: bevy::prelude::ResMut<AdjustedSpawn>,
    ) {
        adjusted.0 = Some(validate_spawn_position(
            Vec3::new(0.1, 0.0, 0.1),
            &obstacles,
            0.5,
        ));
    }

    #[test]
    fn patrol_route_ping_pong_advances_and_reverses() {
        let route = PatrolRoute::new(vec![
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(2.0, 0.0, 0.0),
        ]);

        let mut forward = true;
        let (_, idx1) = route
            .get_next_target(0, &mut forward)
            .expect("target should advance to index 1");
        assert_eq!(idx1, 1);
        assert!(forward);

        let (_, idx2) = route
            .get_next_target(idx1, &mut forward)
            .expect("target should advance to index 2");
        assert_eq!(idx2, 2);

        let (_, idx3) = route
            .get_next_target(idx2, &mut forward)
            .expect("target should reverse toward index 1");
        assert_eq!(idx3, 1);
        assert!(!forward);
    }

    #[test]
    fn validate_spawn_pushes_away_from_obstacles() {
        let mut app = App::new();
        app.insert_resource(AdjustedSpawn::default());
        app.world_mut()
            .spawn((Position::new(Vec3::ZERO), NavigationObstacle));
        app.add_systems(bevy::prelude::Update, compute_adjusted_spawn);

        app.update();

        let adjusted = app
            .world()
            .resource::<AdjustedSpawn>()
            .0
            .expect("Adjusted spawn should be computed");

        assert!(
            adjusted.distance(Vec3::ZERO) >= 2.4,
            "Adjusted position should be moved away from obstacle, got {:?}",
            adjusted
        );
        assert_eq!(adjusted.y, 1.0, "Adjusted spawn Y should be normalized to 1.0");
    }
}
