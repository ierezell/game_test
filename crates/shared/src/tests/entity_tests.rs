#[cfg(test)]
mod tests {
    use crate::entity_implementations::*;
    use crate::entity_traits::*;
    use bevy::prelude::*;

    #[test]
    fn test_floor_entity_visual_provider() {
        let floor = FloorEntity::default();

        // Test mesh creation
        let _mesh = floor.get_mesh();
        // Ensure we can create a mesh (basic validation)
        assert!(true); // Mesh creation should not panic

        // Test material creation
        let material = floor.get_material();
        assert_eq!(material.base_color, floor.get_color());

        // Test color
        let color = floor.get_color();
        assert_eq!(color, bevy::color::palettes::css::GREEN.into());
    }

    #[test]
    fn test_floor_entity_physics_provider() {
        let floor = FloorEntity::default();

        // Test physics bundle creation
        let _physics = floor.get_physics_bundle();
        // Should not panic

        // Test collider
        let _collider = floor.get_collider();
        // Should be a cuboid

        // Test rigid body
        let rigid_body = floor.get_rigid_body();
        assert!(matches!(rigid_body, avian3d::prelude::RigidBody::Static));
    }

    #[test]
    fn test_wall_entity_types() {
        let wall_types = [
            WallType::North,
            WallType::South,
            WallType::East,
            WallType::West,
        ];

        for wall_type in wall_types {
            let wall = WallEntity::new(wall_type.clone());

            // Test entity type
            assert_eq!(wall.entity_type(), "Wall");

            // Test visual provider
            let _mesh = wall.get_mesh();
            let _material = wall.get_material();
            let color = wall.get_color();
            assert_eq!(color, bevy::color::palettes::css::WHITE.into());

            // Test physics provider
            let _physics = wall.get_physics_bundle();
            let _collider = wall.get_collider();
            let rigid_body = wall.get_rigid_body();
            assert!(matches!(rigid_body, avian3d::prelude::RigidBody::Static));

            // Test dimensions
            let (width, height, depth) = wall_type.get_dimensions();
            assert!(width > 0.0);
            assert!(height > 0.0);
            assert!(depth > 0.0);
        }
    }

    #[test]
    fn test_wall_type_dimensions() {
        let north = WallType::North;
        let east = WallType::East;

        let (n_width, n_height, n_depth) = north.get_dimensions();
        let (e_width, e_height, e_depth) = east.get_dimensions();

        // North/South walls should be wide but thin
        assert_eq!(n_width, crate::scene::ROOM_SIZE);
        assert_eq!(n_depth, crate::scene::WALL_THICKNESS);

        // East/West walls should be thin but deep
        assert_eq!(e_width, crate::scene::WALL_THICKNESS);
        assert_eq!(e_depth, crate::scene::ROOM_SIZE);

        // Heights should be the same
        assert_eq!(n_height, e_height);
        assert_eq!(n_height, crate::scene::WALL_HEIGHT);
    }

    #[test]
    fn test_player_entity() {
        let default_player = PlayerEntity::default();
        let custom_player = PlayerEntity::with_color(Color::srgb(1.0, 0.0, 0.0));

        // Test default player
        assert_eq!(default_player.entity_type(), "Player");
        assert_eq!(
            default_player.get_color(),
            bevy::color::palettes::css::BLUE.into()
        );

        // Test custom player
        assert_eq!(custom_player.get_color(), Color::srgb(1.0, 0.0, 0.0));

        // Test physics
        let _physics = default_player.get_physics_bundle();
        let rigid_body = default_player.get_rigid_body();
        assert!(matches!(rigid_body, avian3d::prelude::RigidBody::Dynamic));

        // Test spawn offset
        let offset = default_player.get_spawn_offset();
        assert!(offset.is_some());
        let offset_vec = offset.unwrap();
        assert!(offset_vec.y > 0.0); // Should have positive Y offset
    }

    #[test]
    fn test_game_entity_trait() {
        let floor = FloorEntity::default();
        let wall = WallEntity::default();
        let player = PlayerEntity::default();

        // All entities should implement GameEntity
        assert_eq!(floor.entity_type(), "Floor");
        assert_eq!(wall.entity_type(), "Wall");
        assert_eq!(player.entity_type(), "Player");
    }

    #[test]
    fn test_spawnable_trait() {
        let floor = FloorEntity::default();
        let wall = WallEntity::default();
        let player = PlayerEntity::default();

        // Test spawn offsets
        assert!(floor.get_spawn_offset().is_none()); // Floor has no offset
        assert!(wall.get_spawn_offset().is_none()); // Wall has no offset
        assert!(player.get_spawn_offset().is_some()); // Player has offset
    }
}
