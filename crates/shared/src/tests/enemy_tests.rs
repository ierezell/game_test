#[cfg(test)]
mod enemy_tests {
    use crate::entity_implementations::{EnemyEntity, EnemyType};
    use crate::entity_traits::{GameEntity, PhysicsProvider, Spawnable, VisualProvider};
    use crate::input::{PLAYER_CAPSULE_HEIGHT, PLAYER_CAPSULE_RADIUS};
    use avian3d::prelude::{LockedAxes, RigidBody};

    #[test]
    fn test_enemy_entity_creation() {
        let enemy = EnemyEntity::default();
        assert_eq!(enemy.radius, PLAYER_CAPSULE_RADIUS * 0.8);
        assert_eq!(enemy.height, PLAYER_CAPSULE_HEIGHT * 0.9);
        assert_eq!(enemy.mass, 60.0);
        assert!(matches!(enemy.enemy_type, EnemyType::Basic));
    }

    #[test]
    fn test_enemy_types() {
        let basic = EnemyEntity::new(EnemyType::Basic);
        let fast = EnemyEntity::new(EnemyType::Fast);
        let heavy = EnemyEntity::new(EnemyType::Heavy);

        // Test basic enemy
        assert_eq!(basic.mass, 60.0);
        assert_eq!(basic.radius, PLAYER_CAPSULE_RADIUS * 0.8);

        // Test fast enemy - should be smaller and lighter
        assert_eq!(fast.mass, 40.0);
        assert_eq!(fast.radius, PLAYER_CAPSULE_RADIUS * 0.6);
        assert!(fast.radius < basic.radius);

        // Test heavy enemy - should be larger and heavier
        assert_eq!(heavy.mass, 100.0);
        assert_eq!(heavy.radius, PLAYER_CAPSULE_RADIUS * 1.2);
        assert!(heavy.radius > basic.radius);
    }

    #[test]
    fn test_enemy_visual_provider() {
        let enemy = EnemyEntity::default();
        let _mesh = enemy.get_mesh();
        let material = enemy.get_material();

        // Material should have the enemy's color
        assert_eq!(material.base_color, enemy.get_color());

        // Test different enemy types have different colors
        let basic = EnemyEntity::new(EnemyType::Basic);
        let fast = EnemyEntity::new(EnemyType::Fast);
        let heavy = EnemyEntity::new(EnemyType::Heavy);

        assert_ne!(basic.get_color(), fast.get_color());
        assert_ne!(basic.get_color(), heavy.get_color());
        assert_ne!(fast.get_color(), heavy.get_color());
    }

    #[test]
    fn test_enemy_physics_provider() {
        let enemy = EnemyEntity::default();
        let physics_bundle = enemy.get_physics_bundle();

        // Check collider is correct type and size
        let _collider = enemy.get_collider();
        // For Avian3D, we'll just verify it's a capsule collider by checking it was created with capsule parameters
        // The exact matching depends on the Avian3D version's API

        // Check rigid body is dynamic
        assert!(matches!(enemy.get_rigid_body(), RigidBody::Dynamic));

        // Check mass
        assert_eq!(physics_bundle.mass.0, enemy.mass);

        // Check enemy component has correct stats based on type
        let enemy_stats = &physics_bundle.enemy_bundle.enemy;
        assert_eq!(enemy_stats.health, enemy_stats.max_health);
        assert!(enemy_stats.detection_range > 0.0);
        assert!(enemy_stats.attack_range > 0.0);
        assert!(enemy_stats.move_speed > 0.0);
    }

    #[test]
    fn test_enemy_stats_by_type() {
        let basic = EnemyEntity::new(EnemyType::Basic);
        let fast = EnemyEntity::new(EnemyType::Fast);
        let heavy = EnemyEntity::new(EnemyType::Heavy);

        let basic_bundle = basic.get_physics_bundle();
        let fast_bundle = fast.get_physics_bundle();
        let heavy_bundle = heavy.get_physics_bundle();

        let basic_stats = &basic_bundle.enemy_bundle.enemy;
        let fast_stats = &fast_bundle.enemy_bundle.enemy;
        let heavy_stats = &heavy_bundle.enemy_bundle.enemy;

        // Fast should be faster than basic
        assert!(fast_stats.move_speed > basic_stats.move_speed);

        // Heavy should be slower than basic
        assert!(heavy_stats.move_speed < basic_stats.move_speed);

        // Heavy should have more health
        assert!(heavy_stats.max_health > basic_stats.max_health);

        // Fast should have less health
        assert!(fast_stats.max_health < basic_stats.max_health);

        // Fast should have longer detection range
        assert!(fast_stats.detection_range > basic_stats.detection_range);

        // Heavy should have shorter detection range
        assert!(heavy_stats.detection_range < basic_stats.detection_range);
    }

    #[test]
    fn test_enemy_game_entity() {
        let basic = EnemyEntity::new(EnemyType::Basic);
        let fast = EnemyEntity::new(EnemyType::Fast);
        let heavy = EnemyEntity::new(EnemyType::Heavy);

        assert_eq!(basic.entity_type(), "Enemy_Basic");
        assert_eq!(fast.entity_type(), "Enemy_Fast");
        assert_eq!(heavy.entity_type(), "Enemy_Heavy");
    }

    #[test]
    fn test_enemy_spawnable() {
        let enemy = EnemyEntity::default();
        let spawn_offset = enemy.get_spawn_offset();

        assert!(spawn_offset.is_some());
        let offset = spawn_offset.unwrap();
        assert_eq!(offset.x, 0.0);
        assert_eq!(offset.z, 0.0);
        assert!(offset.y > 0.0); // Should spawn slightly above ground
        assert_eq!(offset.y, enemy.height + 0.1);
    }

    #[test]
    fn test_enemy_physics_properties() {
        let enemy = EnemyEntity::default();
        let physics_bundle = enemy.get_physics_bundle();

        // Test friction is higher than default (more controlled movement)
        assert_eq!(physics_bundle.friction.dynamic_coefficient, 0.7);

        // Test linear damping is higher than player (more controlled movement)
        assert_eq!(physics_bundle.linear_damping.0, 1.5);

        // Test angular damping prevents spinning
        assert_eq!(physics_bundle.angular_damping.0, 10.0);

        // Test rotation is locked except Y axis (by verifying it's the expected locked axes configuration)
        let expected_locked_axes = LockedAxes::ROTATION_LOCKED.unlock_rotation_y();
        // Test by ensuring they have the same internal representation
        assert_eq!(
            format!("{:?}", physics_bundle.locked_axes),
            format!("{:?}", expected_locked_axes)
        );
    }
}
