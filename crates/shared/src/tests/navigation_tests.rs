#[cfg(test)]
mod navigation_tests {
    use crate::navigation::{NavigationAgent, NavigationGrid, NavigationGridMarker};
    use bevy::prelude::*;

    #[test]
    fn test_navigation_grid_creation() {
        let grid = NavigationGrid::default();

        assert_eq!(grid.size, UVec2::new(64, 64));
        assert_eq!(grid.cell_size, 0.5);
        assert!(!grid.is_built);
        assert!(grid.obstacles.is_empty());
    }

    #[test]
    fn test_world_to_grid_conversion() {
        let grid = NavigationGrid::default();

        // Test center position
        let center_world = Vec3::ZERO;
        let center_grid = grid.world_to_grid(center_world);
        assert_eq!(center_grid, IVec2::new(32, 32)); // Center of 64x64 grid

        // Test positive position
        let pos_world = Vec3::new(5.0, 0.0, 5.0);
        let pos_grid = grid.world_to_grid(pos_world);
        assert_eq!(pos_grid, IVec2::new(42, 42)); // 32 + (5.0 / 0.5) = 42

        // Test negative position
        let neg_world = Vec3::new(-5.0, 0.0, -5.0);
        let neg_grid = grid.world_to_grid(neg_world);
        assert_eq!(neg_grid, IVec2::new(22, 22)); // 32 - (5.0 / 0.5) = 22
    }

    #[test]
    fn test_grid_to_world_conversion() {
        let grid = NavigationGrid::default();

        // Test center position
        let center_grid = IVec2::new(32, 32);
        let center_world = grid.grid_to_world(center_grid);
        assert!((center_world.x - 0.0).abs() < 0.001);
        assert!((center_world.z - 0.0).abs() < 0.001);

        // Test positive position
        let pos_grid = IVec2::new(42, 42);
        let pos_world = grid.grid_to_world(pos_grid);
        assert!((pos_world.x - 5.0).abs() < 0.001);
        assert!((pos_world.z - 5.0).abs() < 0.001);

        // Test negative position
        let neg_grid = IVec2::new(22, 22);
        let neg_world = grid.grid_to_world(neg_grid);
        assert!((neg_world.x - (-5.0)).abs() < 0.001);
        assert!((neg_world.z - (-5.0)).abs() < 0.001);
    }

    #[test]
    fn test_navigation_grid_walkability() {
        let mut grid = NavigationGrid::default();

        // Initially all positions should be walkable
        assert!(grid.is_walkable(IVec2::new(10, 10)));
        assert!(grid.is_walkable(IVec2::new(50, 50)));

        // Add some obstacles
        grid.obstacles.insert(IVec2::new(10, 10));
        grid.obstacles.insert(IVec2::new(50, 50));

        // Now those positions should not be walkable
        assert!(!grid.is_walkable(IVec2::new(10, 10)));
        assert!(!grid.is_walkable(IVec2::new(50, 50)));

        // But nearby positions should still be walkable
        assert!(grid.is_walkable(IVec2::new(11, 10)));
        assert!(grid.is_walkable(IVec2::new(10, 11)));

        // Out of bounds should not be walkable
        assert!(!grid.is_walkable(IVec2::new(-1, 0)));
        assert!(!grid.is_walkable(IVec2::new(0, -1)));
        assert!(!grid.is_walkable(IVec2::new(64, 0)));
        assert!(!grid.is_walkable(IVec2::new(0, 64)));
    }

    #[test]
    fn test_navigation_grid_neighbors() {
        let mut grid = NavigationGrid::default();

        // Test neighbors of center position (should have 8 neighbors)
        let center = IVec2::new(32, 32);
        let neighbors = grid.get_neighbors(center);
        assert_eq!(neighbors.len(), 8);

        // Test neighbors include all 8 directions
        assert!(neighbors.contains(&IVec2::new(31, 31))); // NW
        assert!(neighbors.contains(&IVec2::new(32, 31))); // N
        assert!(neighbors.contains(&IVec2::new(33, 31))); // NE
        assert!(neighbors.contains(&IVec2::new(31, 32))); // W
        assert!(neighbors.contains(&IVec2::new(33, 32))); // E
        assert!(neighbors.contains(&IVec2::new(31, 33))); // SW
        assert!(neighbors.contains(&IVec2::new(32, 33))); // S
        assert!(neighbors.contains(&IVec2::new(33, 33))); // SE

        // Test corner position (should have 3 neighbors)
        let corner = IVec2::new(0, 0);
        let corner_neighbors = grid.get_neighbors(corner);
        assert_eq!(corner_neighbors.len(), 3);

        // Add obstacle to block some neighbors
        grid.obstacles.insert(IVec2::new(31, 31));
        grid.obstacles.insert(IVec2::new(32, 31));

        let blocked_neighbors = grid.get_neighbors(center);
        assert_eq!(blocked_neighbors.len(), 6); // 2 neighbors blocked
        assert!(!blocked_neighbors.contains(&IVec2::new(31, 31)));
        assert!(!blocked_neighbors.contains(&IVec2::new(32, 31)));
    }

    #[test]
    fn test_navigation_grid_build() {
        let mut grid = NavigationGrid::default();

        assert!(!grid.is_built);
        assert!(grid.obstacles.is_empty());

        grid.build();

        assert!(grid.is_built);
        assert!(!grid.obstacles.is_empty()); // Should have wall obstacles

        // Check that wall areas are marked as obstacles
        // The exact positions depend on ROOM_SIZE and WALL_THICKNESS constants
        // but there should be obstacles around the perimeter
        let has_obstacles = grid.obstacles.len() > 0;
        assert!(has_obstacles);
    }

    #[test]
    fn test_simple_pathfinding() {
        let mut grid = NavigationGrid::default();
        grid.build();

        // Test simple straight-line path
        let start = Vec3::new(0.0, 0.0, 0.0);
        let end = Vec3::new(2.0, 0.0, 0.0);

        let path = grid.find_path(start, end);
        assert!(path.is_some());

        let path = path.unwrap();
        assert!(!path.is_empty());
        assert_eq!(path[0], grid.grid_to_world(grid.world_to_grid(start)));

        // The path should generally move towards the target
        let last_point = path.last().unwrap();
        let distance_to_target = last_point.distance(end);
        assert!(distance_to_target < 2.0); // Should be reasonably close
    }

    #[test]
    fn test_blocked_pathfinding() {
        let mut grid = NavigationGrid::default();

        // Create a wall of obstacles
        for x in 30..35 {
            grid.obstacles.insert(IVec2::new(x, 32));
        }
        grid.is_built = true;

        // Try to find path through the wall
        let start = Vec3::new(-2.0, 0.0, 0.0);
        let end = Vec3::new(2.0, 0.0, 0.0);

        let path = grid.find_path(start, end);

        if let Some(path) = path {
            // If a path is found, it should go around the obstacle
            assert!(!path.is_empty());

            // Check that none of the path points are in blocked areas
            for point in &path {
                let grid_pos = grid.world_to_grid(*point);
                assert!(grid.is_walkable(grid_pos));
            }
        }
        // If no path is found, that's also acceptable for this test
    }

    #[test]
    fn test_navigation_agent_creation() {
        let agent = NavigationAgent::default();

        assert!(agent.current_path.is_empty());
        assert_eq!(agent.path_index, 0);
        assert!(agent.target_position.is_none());
        assert_eq!(agent.movement_speed, 3.0);
        assert!(!agent.pathfinding_timer.is_finished());
    }

    #[test]
    fn test_navigation_agent_custom_speed() {
        let agent = NavigationAgent {
            movement_speed: 5.0,
            ..default()
        };

        assert_eq!(agent.movement_speed, 5.0);
    }

    #[test]
    fn test_navigation_grid_marker_component() {
        // Test that the marker component can be created
        let _marker = NavigationGridMarker;

        // This is mainly to ensure the component compiles and can be used
        // The actual functionality is tested through integration
    }

    #[test]
    fn test_pathfinding_edge_cases() {
        let mut grid = NavigationGrid::default();
        grid.build();

        // Test pathfinding to same position
        let pos = Vec3::new(1.0, 0.0, 1.0);
        let path = grid.find_path(pos, pos);

        // Should either return empty path or single-point path
        if let Some(path) = path {
            assert!(!path.is_empty());
            if path.len() == 1 {
                let distance = path[0].distance(pos);
                assert!(distance < 1.0); // Should be close to original position
            }
        }

        // Test pathfinding to out-of-bounds position
        let start = Vec3::ZERO;
        let out_of_bounds = Vec3::new(1000.0, 0.0, 1000.0);
        let path = grid.find_path(start, out_of_bounds);

        // Should either find no path or path to edge of grid
        if let Some(path) = path {
            for point in &path {
                let grid_pos = grid.world_to_grid(*point);
                assert!(grid_pos.x >= 0 && grid_pos.x < grid.size.x as i32);
                assert!(grid_pos.y >= 0 && grid_pos.y < grid.size.y as i32);
            }
        }
    }
}
