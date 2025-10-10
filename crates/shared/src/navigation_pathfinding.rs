use crate::scene::WALL_THICKNESS;
use avian3d::prelude::Position;
use bevy::prelude::*;
use pathfinding::prelude::*;
use std::collections::HashMap;

pub struct NavigationPlugin;

impl Plugin for NavigationPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(NavigationGrid::default())
            .add_systems(Startup, setup_navigation_grid)
            .add_systems(
                FixedUpdate,
                (update_enemy_navigation, move_agents_along_path),
            );
    }
}

/// Resource that holds the navigation grid and obstacle data
#[derive(Resource)]
pub struct NavigationGrid {
    pub size: UVec2,
    pub cell_size: f32,
    pub obstacles: HashMap<IVec2, bool>,
    pub is_built: bool,
}

impl Default for NavigationGrid {
    fn default() -> Self {
        Self {
            size: UVec2::new(64, 64), // 64x64 grid
            cell_size: 0.5,
            obstacles: HashMap::new(),
            is_built: false,
        }
    }
}

impl NavigationGrid {
    /// Build the navigation grid by marking obstacles
    pub fn build(&mut self) {
        info!("Building navigation grid with pathfinding algorithms...");

        // Mark outer walls as obstacles
        let wall_thickness_cells = ((WALL_THICKNESS / self.cell_size) as u32).max(1);

        for x in 0..self.size.x {
            for y in 0..self.size.y {
                let is_outer_wall = x < wall_thickness_cells
                    || x >= self.size.x - wall_thickness_cells
                    || y < wall_thickness_cells
                    || y >= self.size.y - wall_thickness_cells;

                if is_outer_wall {
                    self.obstacles.insert(IVec2::new(x as i32, y as i32), true);
                }
            }
        }

        self.is_built = true;
        info!(
            "Navigation grid built with {} obstacles",
            self.obstacles.len()
        );
    }

    /// Convert world position to grid coordinates
    pub fn world_to_grid(&self, world_pos: Vec3) -> IVec2 {
        let half_grid_x = self.size.x as f32 / 2.0;
        let half_grid_y = self.size.y as f32 / 2.0;

        let grid_x = ((world_pos.x / self.cell_size) + half_grid_x)
            .clamp(0.0, self.size.x as f32 - 1.0) as i32;
        let grid_y = ((world_pos.z / self.cell_size) + half_grid_y)
            .clamp(0.0, self.size.y as f32 - 1.0) as i32;

        IVec2::new(grid_x, grid_y)
    }

    /// Convert grid coordinates to world position
    pub fn grid_to_world(&self, grid_pos: IVec2) -> Vec3 {
        let half_grid_x = self.size.x as f32 / 2.0;
        let half_grid_y = self.size.y as f32 / 2.0;

        let world_x = (grid_pos.x as f32 - half_grid_x) * self.cell_size;
        let world_z = (grid_pos.y as f32 - half_grid_y) * self.cell_size;

        Vec3::new(world_x, 0.0, world_z)
    }

    /// Check if a grid position is walkable
    pub fn is_walkable(&self, pos: IVec2) -> bool {
        if pos.x < 0 || pos.y < 0 || pos.x >= self.size.x as i32 || pos.y >= self.size.y as i32 {
            return false;
        }
        !self.obstacles.contains_key(&pos)
    }

    /// Find path using A* algorithm from pathfinding crate
    pub fn find_path(&self, start: Vec3, end: Vec3) -> Option<Vec<Vec3>> {
        if !self.is_built {
            return None;
        }

        let start_grid = self.world_to_grid(start);
        let end_grid = self.world_to_grid(end);

        if !self.is_walkable(start_grid) || !self.is_walkable(end_grid) {
            return None;
        }

        #[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        struct GridPos(i32, i32);

        impl GridPos {
            fn successors(&self, grid: &NavigationGrid) -> Vec<(GridPos, i32)> {
                let &GridPos(x, y) = self;
                vec![
                    GridPos(x + 1, y),
                    GridPos(x - 1, y),
                    GridPos(x, y + 1),
                    GridPos(x, y - 1),
                    GridPos(x + 1, y + 1),
                    GridPos(x + 1, y - 1),
                    GridPos(x - 1, y + 1),
                    GridPos(x - 1, y - 1),
                ]
                .into_iter()
                .filter(|pos| grid.is_walkable(IVec2::new(pos.0, pos.1)))
                .map(|pos| {
                    let cost = if pos.0.abs_diff(x) + pos.1.abs_diff(y) == 1 {
                        1
                    } else {
                        2
                    };
                    (pos, cost)
                })
                .collect()
            }

            fn distance(&self, other: &GridPos) -> i32 {
                (self.0.abs_diff(other.0) + self.1.abs_diff(other.1)) as i32
            }
        }

        let start_pos = GridPos(start_grid.x, start_grid.y);
        let end_pos = GridPos(end_grid.x, end_grid.y);

        if let Some(path) = astar(
            &start_pos,
            |p| p.successors(self),
            |p| p.distance(&end_pos),
            |p| *p == end_pos,
        ) {
            // Convert grid path to world coordinates
            let world_path = path
                .0
                .iter()
                .map(|GridPos(x, y)| self.grid_to_world(IVec2::new(*x, *y)))
                .collect();

            Some(world_path)
        } else {
            None
        }
    }

    /// Add a dynamic obstacle
    pub fn add_obstacle(&mut self, world_pos: Vec3) {
        let grid_pos = self.world_to_grid(world_pos);
        self.obstacles.insert(grid_pos, true);
    }

    /// Remove a dynamic obstacle
    pub fn remove_obstacle(&mut self, world_pos: Vec3) {
        let grid_pos = self.world_to_grid(world_pos);
        self.obstacles.remove(&grid_pos);
    }
}

/// Component to mark entities that should use navigation
#[derive(Component, Clone, Debug)]
pub struct NavigationAgent {
    pub current_path: Vec<Vec3>,
    pub path_index: usize,
    pub target_position: Option<Vec3>,
    pub movement_speed: f32,
    pub pathfinding_timer: Timer,
}

impl Default for NavigationAgent {
    fn default() -> Self {
        Self {
            current_path: Vec::default(),
            path_index: 0,
            target_position: None,
            movement_speed: 3.0,
            pathfinding_timer: Timer::from_seconds(0.5, TimerMode::Repeating),
        }
    }
}

/// Component to mark the navigation grid entity
#[derive(Component)]
pub struct NavigationMeshMarker;

/// Setup the navigation grid
fn setup_navigation_grid(mut nav_grid: ResMut<NavigationGrid>) {
    info!("Setting up pathfinding navigation grid...");
    nav_grid.build();
}

/// System to update enemy navigation using pathfinding algorithms
fn update_enemy_navigation(
    mut agent_query: Query<(Entity, &mut NavigationAgent, &Position), With<crate::enemy::Enemy>>,
    player_query: Query<
        &Position,
        (
            With<crate::protocol::PlayerId>,
            Without<crate::enemy::Enemy>,
        ),
    >,
    nav_grid: Res<NavigationGrid>,
    time: Res<Time>,
) {
    if !nav_grid.is_built {
        return; // Navigation grid not ready
    }

    for (entity, mut agent, position) in agent_query.iter_mut() {
        agent.pathfinding_timer.tick(time.delta());

        // Only update pathfinding periodically to avoid performance issues
        if !agent.pathfinding_timer.just_finished() {
            continue;
        }

        // Find the closest player as target
        let closest_player = player_query
            .iter()
            .map(|player_pos| (player_pos.0, position.0.distance(player_pos.0)))
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

        if let Some((target_world_pos, distance)) = closest_player {
            // Only pathfind if player is within reasonable range
            if distance < 25.0 && distance > 1.0 {
                agent.target_position = Some(target_world_pos);

                // Calculate path using pathfinding crate's A* algorithm
                if let Some(path) = nav_grid.find_path(position.0, target_world_pos) {
                    agent.current_path = path;
                    agent.path_index = 0;

                    debug!(
                        "Entity {:?} found path with {} waypoints to target using pathfinding crate",
                        entity,
                        agent.current_path.len()
                    );
                } else {
                    // If pathfinding fails, clear the path
                    agent.current_path.clear();
                    agent.path_index = 0;
                    debug!("Entity {:?} failed to find path to target", entity);
                }
            } else {
                // Target is too far or too close, clear path
                if distance > 25.0 {
                    agent.current_path.clear();
                    agent.path_index = 0;
                    agent.target_position = None;
                }
            }
        }
    }
}

/// System to move agents along their calculated paths
fn move_agents_along_path(
    mut agent_query: Query<(Entity, &mut NavigationAgent, &mut Position)>,
    time: Res<Time>,
) {
    for (entity, mut agent, mut position) in agent_query.iter_mut() {
        if agent.current_path.is_empty() || agent.path_index >= agent.current_path.len() {
            continue;
        }

        let target_waypoint = agent.current_path[agent.path_index];
        let current_pos = position.0;
        let direction = (target_waypoint - current_pos).normalize_or_zero();
        let distance_to_waypoint = current_pos.distance(target_waypoint);

        let movement_distance = agent.movement_speed * time.delta().as_secs_f32();

        if distance_to_waypoint <= movement_distance {
            // Reached current waypoint, move to next
            position.0 = target_waypoint;
            agent.path_index += 1;

            if agent.path_index >= agent.current_path.len() {
                // Reached the end of the path
                debug!("Entity {:?} reached end of path", entity);
                agent.current_path.clear();
                agent.path_index = 0;
            }
        } else {
            // Move towards current waypoint
            position.0 += direction * movement_distance;
        }
    }
}

/// Helper function to add navigation agent to an entity
pub fn add_navigation_agent(commands: &mut Commands, entity: Entity) {
    commands.entity(entity).insert(NavigationAgent::default());
}

/// Helper function to add navigation agent with custom speed
pub fn add_navigation_agent_with_speed(commands: &mut Commands, entity: Entity, speed: f32) {
    commands.entity(entity).insert(NavigationAgent {
        movement_speed: speed,
        ..default()
    });
}

/// Helper function to create dynamic obstacle
pub fn create_dynamic_obstacle(nav_grid: &mut ResMut<NavigationGrid>, position: Vec3) {
    nav_grid.add_obstacle(position);
}

/// Helper function to remove dynamic obstacle
pub fn remove_dynamic_obstacle(nav_grid: &mut ResMut<NavigationGrid>, position: Vec3) {
    nav_grid.remove_obstacle(position);
}
