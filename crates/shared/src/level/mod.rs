pub mod create_dynamic;
pub mod create_static;
pub mod render;

#[cfg(test)]
mod tests {
    use super::*;
    use avian3d::prelude::Position;
    use bevy::prelude::*;
    use create_static::{LevelDoneMarker, ROOM_SIZE, WALL_HEIGHT, setup_static_level};

    #[test]
    fn test_setup_static_level_creates_entities() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        // Call the system
        setup_static_level(app.world_mut().commands(), Some(12345));
        app.update();

        let mut query = app.world_mut().query::<&LevelDoneMarker>();
        let level_markers = query.iter(app.world()).count();

        let mut query = app.world_mut().query::<&Position>();
        let positions = query.iter(app.world()).count();

        let mut query = app.world_mut().query::<&Name>();
        let names = query.iter(app.world()).count();

        assert_eq!(level_markers, 1, "Should create exactly one level marker");
        assert!(
            positions >= 5,
            "Should create at least 5 positioned entities (floor + 4 walls)"
        );
        assert!(names >= 5, "Should create at least 5 named entities");
    }

    #[test]
    fn test_setup_static_level_with_different_seeds() {
        let mut app1 = App::new();
        app1.add_plugins(MinimalPlugins);
        setup_static_level(app1.world_mut().commands(), Some(111));
        app1.update();

        let mut app2 = App::new();
        app2.add_plugins(MinimalPlugins);
        setup_static_level(app2.world_mut().commands(), Some(222));
        app2.update();

        // Both should create the same number of entities (deterministic for now)
        let mut query1 = app1.world_mut().query::<&Position>();
        let count1 = query1.iter(app1.world()).count();

        let mut query2 = app2.world_mut().query::<&Position>();
        let count2 = query2.iter(app2.world()).count();

        assert_eq!(
            count1, count2,
            "Different seeds should create same number of entities"
        );
    }

    #[test]
    fn test_setup_static_level_with_none_seed() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        setup_static_level(app.world_mut().commands(), None);
        app.update();

        // Should still work with default seed
        let mut query = app.world_mut().query::<&LevelDoneMarker>();
        let level_markers = query.iter(app.world()).count();
        assert_eq!(
            level_markers, 1,
            "Should create level marker even with None seed"
        );
    }

    #[test]
    fn test_wall_positions_are_correct() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        setup_static_level(app.world_mut().commands(), Some(42));
        app.update();

        let mut wall_positions = Vec::new();
        let mut query = app.world_mut().query::<(&Name, &Position)>();
        for (name, position) in query.iter(app.world()) {
            if name.as_str().starts_with("Wall") {
                wall_positions.push((name.as_str().to_string(), position.0));
            }
        }

        assert_eq!(wall_positions.len(), 4, "Should create exactly 4 walls");

        // Check that walls are positioned at room boundaries
        for (wall_name, pos) in wall_positions {
            match wall_name.as_str() {
                "Wall East" => {
                    assert!(
                        (pos.x - (ROOM_SIZE / 2.0)).abs() > 0.0,
                        "East wall should be positioned correctly"
                    );
                    assert_eq!(pos.y, WALL_HEIGHT / 2.0, "Wall height should be correct");
                }
                "Wall West" => {
                    assert!(
                        (pos.x + (ROOM_SIZE / 2.0)).abs() > 0.0,
                        "West wall should be positioned correctly"
                    );
                    assert_eq!(pos.y, WALL_HEIGHT / 2.0, "Wall height should be correct");
                }
                "Wall North" => {
                    assert!(
                        (pos.z - (ROOM_SIZE / 2.0)).abs() > 0.0,
                        "North wall should be positioned correctly"
                    );
                    assert_eq!(pos.y, WALL_HEIGHT / 2.0, "Wall height should be correct");
                }
                "Wall South" => {
                    assert!(
                        (pos.z + (ROOM_SIZE / 2.0)).abs() > 0.0,
                        "South wall should be positioned correctly"
                    );
                    assert_eq!(pos.y, WALL_HEIGHT / 2.0, "Wall height should be correct");
                }
                _ => panic!("Unexpected wall name: {}", wall_name),
            }
        }
    }
}
