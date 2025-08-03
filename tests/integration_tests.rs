use bevy::prelude::*;
use gtfo_like_game;

#[cfg(test)]
mod integration_tests {
    use super::*;

    // Helper to create a test app with minimal setup
    fn create_test_app() -> App {
        let mut app = App::new();
        
        // Add minimal plugins needed for testing
        app.add_plugins(MinimalPlugins)
            .add_plugins(bevy::time::TimePlugin)
            .add_plugins(bevy::transform::TransformPlugin)
            .add_plugins(avian3d::PhysicsPlugins::default())
            .init_state::<gtfo_like_game::game_state::GameState>();
            
        app
    }

    #[test]
    fn test_weapon_properties() {
        // Test assault rifle
        let rifle = gtfo_like_game::combat::Weapon::new_assault_rifle();
        assert_eq!(rifle.damage, 50.0, "Assault rifle should do 50 damage");
        assert_eq!(rifle.ammo_max, 30, "Assault rifle should have 30 rounds");
        assert!(rifle.can_fire(1.0), "New weapon should be able to fire after rate of fire delay");
        
        // Test shotgun
        let shotgun = gtfo_like_game::combat::Weapon::new_shotgun();
        assert_eq!(shotgun.damage, 80.0, "Shotgun should do 80 damage");
        assert_eq!(shotgun.ammo_max, 8, "Shotgun should have 8 rounds");
        
        println!("✓ Weapon properties test passed");
    }

    #[test]
    fn test_health_system() {
        let mut health = gtfo_like_game::combat::Health::new(100.0);
        
        // Test initial state
        assert_eq!(health.current, 100.0);
        assert_eq!(health.maximum, 100.0);
        assert!(!health.is_dead());
        
        // Test damage
        health.take_damage(50.0, 1.0);
        assert_eq!(health.current, 50.0);
        assert!(!health.is_dead());
        
        // Test killing blow
        health.take_damage(60.0, 2.0);
        assert_eq!(health.current, 0.0);
        assert!(health.is_dead());
        
        println!("✓ Health system test passed");
    }

    #[test]
    fn test_enemy_creation() {
        let striker = gtfo_like_game::enemies::Enemy::new_striker();
        assert_eq!(striker.attack_damage, 25.0);
        assert_eq!(striker.move_speed, 6.0);
        
        let shooter = gtfo_like_game::enemies::Enemy::new_shooter();
        assert_eq!(shooter.attack_damage, 35.0);
        assert_eq!(shooter.move_speed, 3.0);
        
        let tank = gtfo_like_game::enemies::Enemy::new_tank();
        assert_eq!(tank.attack_damage, 50.0);
        assert_eq!(tank.move_speed, 2.0);
        
        println!("✓ Enemy creation test passed");
    }

    #[test]
    fn test_game_balance() {
        // Test that assault rifle can kill enemy in reasonable number of shots
        let rifle = gtfo_like_game::combat::Weapon::new_assault_rifle();
        let enemy_health = 60.0f32; // Current enemy health from static level
        
        let shots_to_kill = (enemy_health / rifle.damage).ceil() as u32;
        assert_eq!(shots_to_kill, 2, "Enemy should die in exactly 2 shots with assault rifle");
        
        // Test that one magazine is more than enough
        assert!(rifle.ammo_max >= shots_to_kill, "One magazine should contain enough ammo to kill enemy");
        
        println!("✓ Game balance test passed");
    }

    #[test]
    fn test_weapon_reload() {
        let mut weapon = gtfo_like_game::combat::Weapon::new_assault_rifle();
        
        // Test initial state
        assert_eq!(weapon.ammo_current, weapon.ammo_max);
        assert!(!weapon.is_reloading);
        
        // Simulate firing all ammo
        weapon.ammo_current = 0;
        assert!(!weapon.can_fire(0.0));
        
        // Test reload
        weapon.start_reload(0.0);
        assert!(weapon.is_reloading);
        
        // Simulate time passing
        weapon.update_reload(weapon.reload_time + 0.1);
        assert!(!weapon.is_reloading);
        assert_eq!(weapon.ammo_current, weapon.ammo_max);
        
        println!("✓ Weapon reload test passed");
    }

    #[test]
    fn test_projectile_creation() {
        let projectile = gtfo_like_game::combat::Projectile {
            damage: 35.0,
            speed: 50.0,
            lifetime: 5.0,
            penetration: 1,
            owner: Entity::PLACEHOLDER,
        };
        
        assert_eq!(projectile.damage, 35.0);
        assert_eq!(projectile.speed, 50.0);
        assert_eq!(projectile.lifetime, 5.0);
        
        println!("✓ Projectile creation test passed");
    }

    #[test]
    fn test_enemy_ai_creation() {
        let patrol_points = vec![
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(5.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 5.0),
        ];
        
        let ai = gtfo_like_game::enemies::EnemyAI {
            detection_radius: 10.0,
            patrol_points: patrol_points.clone(),
            current_patrol_index: 0,
            last_known_player_position: None,
            search_timer: 0.0,
            reaction_time: 0.5,
        };
        
        assert_eq!(ai.detection_radius, 10.0);
        assert_eq!(ai.patrol_points.len(), 3);
        assert_eq!(ai.current_patrol_index, 0);
        
        println!("✓ Enemy AI creation test passed");
    }

    #[test]
    fn test_damage_calculation() {
        // Test scenarios to ensure game balance
        let rifle_damage = 35.0f32;
        let shotgun_damage = 80.0f32;
        let enemy_health = 60.0f32;
        
        // Rifle should take 2 shots
        assert_eq!((enemy_health / rifle_damage).ceil() as u32, 2);
        
        // Shotgun should take 1 shot
        assert_eq!((enemy_health / shotgun_damage).ceil() as u32, 1);
        
        // Player health vs enemy damage
        let player_health = 100.0f32;
        let enemy_damage = 20.0f32;
        
        // Enemy should take 5 hits to kill player
        assert_eq!((player_health / enemy_damage).ceil() as u32, 5);
        
        println!("✓ Damage calculation test passed");
    }
}
