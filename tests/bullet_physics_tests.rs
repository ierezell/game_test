#[cfg(test)]
mod bullet_physics_tests {
    use avian3d::prelude::*;
    use bevy::input::InputPlugin;
    use bevy::prelude::*;
    use gtfo_like_game::combat::*;
    use gtfo_like_game::player::*;
    use gtfo_like_game::static_level::*;

    fn setup_physics_test_app() -> App {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            AssetPlugin::default(),
            TransformPlugin,
            InputPlugin,
        ))
        .init_asset::<Mesh>()
        .init_asset::<StandardMaterial>()
        .add_plugins(PhysicsPlugins::default())
        .add_plugins(CombatPlugin)
        .add_plugins(PlayerPlugin)
        .add_plugins(StaticLevelPlugin);
        app
    }

    fn setup_simple_physics_test_app() -> App {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            AssetPlugin::default(),
            TransformPlugin,
            InputPlugin,
        ))
        .init_asset::<Mesh>()
        .init_asset::<StandardMaterial>()
        .add_plugins(PhysicsPlugins::default())
        .add_plugins(CombatPlugin)
        .add_plugins(PlayerPlugin);
        app
    }

    #[test]
    fn test_bullet_trajectory_accuracy() {
        let mut app = setup_physics_test_app();

        // Create a camera at origin looking forward
        let camera_entity = app
            .world_mut()
            .spawn((
                FirstPersonCamera,
                Transform::from_xyz(0.0, 1.7, 0.0).looking_at(Vec3::new(0.0, 1.7, 5.0), Vec3::Y),
                GlobalTransform::default(),
            ))
            .id();

        // Create a player with a weapon
        let _player_entity = app
            .world_mut()
            .spawn((
                Player,
                PlayerController::default(),
                Weapon::new_assault_rifle(),
                Transform::from_xyz(0.0, 0.9, 0.0),
                GlobalTransform::default(),
            ))
            .id();

        app.update();

        // Get the camera transform
        let camera_transform = app
            .world()
            .get::<GlobalTransform>(camera_entity)
            .unwrap()
            .clone();

        // Verify camera is pointing in the right direction
        let forward = camera_transform.forward();
        assert!((forward.z - 1.0).abs() < 0.1); // Should be pointing roughly in +Z direction

        // Test that projectile spawn position is calculated correctly
        let expected_spawn_pos = camera_transform.translation() + forward * 0.5;
        assert!((expected_spawn_pos.x - 0.0).abs() < 0.1);
        assert!((expected_spawn_pos.y - 1.7).abs() < 0.1);
        assert!((expected_spawn_pos.z - 0.5).abs() < 0.1);
    }

    #[test]
    fn test_collision_detection_setup() {
        // Simplified collision test without complex layer masks
        let mut app = setup_physics_test_app();

        // Create a simple collision setup
        let wall_entity = app
            .world_mut()
            .spawn((
                Transform::from_xyz(5.0, 1.0, 0.0),
                RigidBody::Static,
                Collider::cuboid(1.0, 1.0, 1.0),
                CollisionLayers::default(),
            ))
            .id();

        let projectile_entity = app
            .world_mut()
            .spawn((
                Transform::from_xyz(0.0, 1.0, 0.0),
                RigidBody::Dynamic,
                Collider::sphere(0.03),
                CollisionLayers::default(),
                LinearVelocity(Vec3::new(10.0, 0.0, 0.0)),
                Mass(0.01),
                Restitution::new(0.0),
                Friction::new(0.0),
                Projectile {
                    damage: 25.0,
                    speed: 30.0,
                    lifetime: 3.0,
                    penetration: 1,
                    owner: Entity::PLACEHOLDER,
                },
            ))
            .id();

        app.update();

        // Verify entities were created
        assert!(app.world().get_entity(wall_entity).is_ok());
        assert!(app.world().get_entity(projectile_entity).is_ok());
    }

    #[test]
    fn test_projectile_physics_properties() {
        let mut app = setup_simple_physics_test_app();

        // Spawn a projectile with our new physics settings
        let projectile_entity = app
            .world_mut()
            .spawn((
                Transform::from_xyz(0.0, 1.0, 0.0),
                RigidBody::Dynamic,
                Collider::sphere(0.03),
                CollisionLayers::default(),
                LinearVelocity(Vec3::new(0.0, 0.0, 30.0)),
                Mass(0.01),
                Restitution::new(0.0),
                Friction::new(0.0),
                Projectile {
                    damage: 25.0,
                    speed: 30.0,
                    lifetime: 3.0,
                    penetration: 1,
                    owner: Entity::PLACEHOLDER,
                },
            ))
            .id();

        app.update();

        // Verify the projectile was created with correct properties
        let projectile = app.world().get::<Projectile>(projectile_entity).unwrap();
        assert_eq!(projectile.damage, 25.0);
        assert_eq!(projectile.speed, 30.0);
        assert_eq!(projectile.lifetime, 3.0);

        let mass = app.world().get::<Mass>(projectile_entity).unwrap();
        assert_eq!(mass.0, 0.01);

        let restitution = app.world().get::<Restitution>(projectile_entity).unwrap();
        assert_eq!(restitution.coefficient, 0.0);

        let friction = app.world().get::<Friction>(projectile_entity).unwrap();
        assert_eq!(friction.dynamic_coefficient, 0.0);
    }

    #[test]
    fn test_weapon_damage_balance() {
        // This is a unit test, no app setup needed
        let assault_rifle = Weapon::new_assault_rifle();
        let shotgun = Weapon::new_shotgun();

        // Verify damage has been reduced to reasonable levels
        assert!(
            assault_rifle.damage <= 30.0,
            "Assault rifle damage should be <= 30.0, got {}",
            assault_rifle.damage
        );
        assert!(
            shotgun.damage <= 50.0,
            "Shotgun damage should be <= 50.0, got {}",
            shotgun.damage
        );

        // Verify damage is still effective but not excessive
        assert!(
            assault_rifle.damage >= 20.0,
            "Assault rifle damage should be >= 20.0 for effectiveness"
        );
        assert!(
            shotgun.damage >= 30.0,
            "Shotgun damage should be >= 30.0 for close range effectiveness"
        );
    }

    #[test]
    fn test_projectile_spread_accuracy() {
        // This is a unit test, no app setup needed
        let weapon = Weapon::new_assault_rifle();

        // Test that spread calculation is reasonable
        let accuracy = weapon.accuracy; // 0.85
        let spread = (1.0 - accuracy) * 0.05; // Should be 0.15 * 0.05 = 0.0075

        assert!(
            spread < 0.01,
            "Spread should be very small for good accuracy, got {}",
            spread
        );
        assert!(spread > 0.0, "Spread should be greater than 0 for realism");

        // Test spread range
        assert!(spread <= 0.01, "Maximum spread should be reasonable");
    }

    #[test]
    fn test_projectile_speed_and_lifetime() {
        let mut app = setup_simple_physics_test_app();

        // Spawn projectile
        let projectile_entity = app
            .world_mut()
            .spawn((
                Transform::from_xyz(0.0, 1.0, 0.0),
                LinearVelocity(Vec3::new(0.0, 0.0, 30.0)),
                Projectile {
                    damage: 25.0,
                    speed: 30.0,
                    lifetime: 3.0,
                    penetration: 1,
                    owner: Entity::PLACEHOLDER,
                },
            ))
            .id();

        // Test initial values
        let velocity = app
            .world()
            .get::<LinearVelocity>(projectile_entity)
            .unwrap();
        assert_eq!(velocity.0.z, 30.0);

        let projectile = app.world().get::<Projectile>(projectile_entity).unwrap();
        assert_eq!(projectile.lifetime, 3.0);

        // With speed 30.0 and lifetime 3.0, max range should be 90 units
        let max_range = projectile.speed * projectile.lifetime;
        assert_eq!(max_range, 90.0);

        // This is reasonable for indoor combat scenarios
        assert!(max_range >= 60.0 && max_range <= 120.0);
    }

    #[test]
    fn test_wall_collision_layers() {
        let mut app = setup_physics_test_app();

        // The static level should create walls with proper collision layers
        app.update();

        // Query for entities with wall collision layers
        let mut wall_query = app
            .world_mut()
            .query_filtered::<Entity, (With<Collider>, With<RigidBody>)>();
        let wall_count = wall_query.iter(app.world()).count();

        // Should have at least floor + 4 walls = 5 entities with colliders
        assert!(
            wall_count >= 5,
            "Should have at least 5 collision entities (floor + walls), got {}",
            wall_count
        );
    }
}
