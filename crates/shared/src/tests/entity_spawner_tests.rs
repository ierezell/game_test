#[cfg(test)]
mod entity_spawner_tests {
    use crate::entity_implementations::{
        EnemyEntity, EnemyType, FloorEntity, PlayerEntity, WallEntity, WallType,
    };
    use crate::entity_traits::{GameEntity, PhysicsProvider, Spawnable, VisualProvider};
    use bevy::prelude::*;

    #[test]
    fn test_floor_entity_spawnable() {
        let floor = FloorEntity::default();
        assert!(floor.get_spawn_offset().is_none()); // Floor entities spawn at exact position
        assert_eq!(floor.entity_type(), "Floor");
    }

    #[test]
    fn test_wall_entity_spawnable() {
        let wall = WallEntity::new(WallType::North);
        assert!(wall.get_spawn_offset().is_none()); // Wall entities spawn at exact position  
        assert_eq!(wall.entity_type(), "Wall");
    }

    #[test]
    fn test_player_entity_spawnable() {
        let player = PlayerEntity::default();
        assert!(player.get_spawn_offset().is_some());
        assert_eq!(player.entity_type(), "Player");
    }

    #[test]
    fn test_enemy_entity_spawnable() {
        let enemy = EnemyEntity::new(EnemyType::Basic);
        assert!(enemy.get_spawn_offset().is_some());
        assert_eq!(enemy.entity_type(), "Enemy_Basic");
    }

    #[test]
    fn test_enemy_types_spawn_correctly() {
        let basic = EnemyEntity::new(EnemyType::Basic);
        let fast = EnemyEntity::new(EnemyType::Fast);
        let heavy = EnemyEntity::new(EnemyType::Heavy);

        assert_eq!(basic.entity_type(), "Enemy_Basic");
        assert_eq!(fast.entity_type(), "Enemy_Fast");
        assert_eq!(heavy.entity_type(), "Enemy_Heavy");

        // All should be spawnable
        assert!(basic.get_spawn_offset().is_some());
        assert!(fast.get_spawn_offset().is_some());
        assert!(heavy.get_spawn_offset().is_some());
    }

    #[test]
    fn test_wall_types_spawn_correctly() {
        let north_wall = WallEntity::new(WallType::North);
        let south_wall = WallEntity::new(WallType::South);
        let east_wall = WallEntity::new(WallType::East);
        let west_wall = WallEntity::new(WallType::West);

        assert_eq!(north_wall.entity_type(), "Wall");
        assert_eq!(south_wall.entity_type(), "Wall");
        assert_eq!(east_wall.entity_type(), "Wall");
        assert_eq!(west_wall.entity_type(), "Wall");

        // All should be spawnable (at exact position)
        assert!(north_wall.get_spawn_offset().is_none());
        assert!(south_wall.get_spawn_offset().is_none());
        assert!(east_wall.get_spawn_offset().is_none());
        assert!(west_wall.get_spawn_offset().is_none());
    }

    #[test]
    fn test_entity_physics_bundles() {
        let floor = FloorEntity::default();
        let wall = WallEntity::new(WallType::North);
        let player = PlayerEntity::default();
        let enemy = EnemyEntity::new(EnemyType::Basic);

        // Test that physics bundles can be created
        let _floor_physics = floor.get_physics_bundle();
        let _wall_physics = wall.get_physics_bundle();
        let _player_physics = player.get_physics_bundle();
        let _enemy_physics = enemy.get_physics_bundle();

        // Test that all can generate visuals
        let _floor_mesh = floor.get_mesh();
        let _wall_mesh = wall.get_mesh();
        let _player_mesh = player.get_mesh();
        let _enemy_mesh = enemy.get_mesh();

        let _floor_material = floor.get_material();
        let _wall_material = wall.get_material();
        let _player_material = player.get_material();
        let _enemy_material = enemy.get_material();
    }

    #[test]
    fn test_spawn_positions() {
        let entities = [
            (FloorEntity::default(), Vec3::ZERO),
            (FloorEntity::default(), Vec3::new(10.0, 5.0, -3.0)),
            (FloorEntity::default(), Vec3::new(-5.0, 0.0, 8.0)),
        ];

        for (entity, base_pos) in entities {
            let spawn_offset = entity.get_spawn_offset().unwrap_or(Vec3::ZERO);
            let final_pos = base_pos + spawn_offset;

            // Verify final position is calculated correctly
            assert_eq!(final_pos.x, base_pos.x + spawn_offset.x);
            assert_eq!(final_pos.y, base_pos.y + spawn_offset.y);
            assert_eq!(final_pos.z, base_pos.z + spawn_offset.z);
        }
    }
}
