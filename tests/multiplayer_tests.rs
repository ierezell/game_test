use bevy::prelude::*;
use gtfo_like_game::{
    protocol::*,
    shared::SharedPlugin,
};
use avian3d::prelude::Position;

/// Test basic client-server connection
#[test]
fn test_client_server_connection() {
    // This is a placeholder test - we'll need to set up a proper test environment
    // with Lightyear's test infrastructure once the basic networking is working
    
    // For now, just test that we can create the main components
    let mut app = App::new();
    app.add_plugins(SharedPlugin);
    
    // Test that we can spawn networked entities
    let player_entity = app.world_mut().spawn(NetworkedPlayerBundle::new(1, Vec3::ZERO)).id();
    
    // Verify the entity has the expected components
    assert!(app.world().get_entity(player_entity).is_ok());
    assert!(app.world().get::<PlayerId>(player_entity).is_some());
    assert!(app.world().get::<PlayerHealth>(player_entity).is_some());
    assert!(app.world().get::<PlayerStamina>(player_entity).is_some());
}

/// Test message serialization and deserialization
#[test]
fn test_message_serialization() {
    use serde_json;
    
    // Test WeaponFireMessage serialization
    let message = WeaponFireMessage {
        player_id: 12345,
        origin: Vec3::new(1.0, 2.0, 3.0),
        direction: Vec3::new(0.0, 0.0, 1.0),
        damage: 25.0,
    };
    
    // Serialize to JSON (this tests that Serialize is implemented correctly)
    let serialized = serde_json::to_string(&message).expect("Should serialize");
    
    // Deserialize back
    let deserialized: WeaponFireMessage = serde_json::from_str(&serialized).expect("Should deserialize");
    
    assert_eq!(message.player_id, deserialized.player_id);
    assert_eq!(message.origin, deserialized.origin);
    assert_eq!(message.direction, deserialized.direction);
    assert_eq!(message.damage, deserialized.damage);
}

/// Test player component updates
#[test]  
fn test_player_component_updates() {
    let mut app = App::new();
    app.add_plugins(SharedPlugin);
    
    // Spawn a networked player
    let player_entity = app.world_mut().spawn(NetworkedPlayerBundle::new(1, Vec3::ZERO)).id();
    
    // Test health updates
    let mut health = app.world_mut().get_mut::<PlayerHealth>(player_entity).unwrap();
    assert_eq!(health.current, 100.0);
    health.current = 75.0;
    drop(health); // Release the borrow
    
    let health = app.world().get::<PlayerHealth>(player_entity).unwrap();
    assert_eq!(health.current, 75.0);
    
    // Test stamina updates
    let mut stamina = app.world_mut().get_mut::<PlayerStamina>(player_entity).unwrap();
    assert_eq!(stamina.current, 100.0);
    stamina.current = 50.0;
    drop(stamina);
    
    let stamina = app.world().get::<PlayerStamina>(player_entity).unwrap();
    assert_eq!(stamina.current, 50.0);
}

/// Test NetworkedEnemyType enum
#[test]
fn test_enemy_types() {
    use gtfo_like_game::protocol::{EnemyType, NetworkedEnemyType};
    
    let striker = NetworkedEnemyType(EnemyType::Striker);
    let _charger = NetworkedEnemyType(EnemyType::Charger);
    let _shooter = NetworkedEnemyType(EnemyType::Shooter);
    
    // Test serialization
    let serialized = serde_json::to_string(&striker).expect("Should serialize");
    let deserialized: NetworkedEnemyType = serde_json::from_str(&serialized).expect("Should deserialize");
    
    assert_eq!(striker.0 as u8, deserialized.0 as u8);
}

/// Test player position and rotation components
#[test]
fn test_player_transform_components() {
    let pos = PlayerPosition(Vec3::new(10.0, 5.0, 3.0));
    let rot = PlayerRotation(Quat::from_rotation_y(0.5));
    
    // Test that we can serialize and deserialize positions/rotations
    let pos_serialized = serde_json::to_string(&pos).expect("Should serialize position");
    let rot_serialized = serde_json::to_string(&rot).expect("Should serialize rotation");
    
    let pos_deserialized: PlayerPosition = serde_json::from_str(&pos_serialized).expect("Should deserialize position");
    let rot_deserialized: PlayerRotation = serde_json::from_str(&rot_serialized).expect("Should deserialize rotation");
    
    assert_eq!(pos.0, pos_deserialized.0);
    assert_eq!(rot.0, rot_deserialized.0);
}

// Integration tests for the full multiplayer setup would go here, but they require
// setting up actual client-server networking which is more complex.
// For now, these component and serialization tests verify the basic building blocks work.

#[cfg(test)]
mod integration_tests {
    use super::*;
    
    /// Test basic multiplayer component setup without full networking
    #[test]  
    fn test_basic_multiplayer_setup() {
        // Test that we can set up the basic components without running a full frame
        let mut app = App::new();
        app.add_plugins(SharedPlugin);
        
        // Spawn multiple players to test multi-entity scenarios
        let player1 = app.world_mut().spawn(NetworkedPlayerBundle::new(1, Vec3::new(0.0, 0.0, 0.0))).id();
        let player2 = app.world_mut().spawn(NetworkedPlayerBundle::new(2, Vec3::new(10.0, 0.0, 0.0))).id();
        
        // Verify both entities exist and have the expected components
        assert!(app.world().get_entity(player1).is_ok());
        assert!(app.world().get_entity(player2).is_ok());
        
        // Verify they have different IDs and positions
        let pos1 = app.world().get::<Position>(player1).unwrap();
        let pos2 = app.world().get::<Position>(player2).unwrap();
        
        assert_ne!(pos1.0, pos2.0); // Different positions
        
        // Test that we can modify component state
        {
            let mut health1 = app.world_mut().get_mut::<PlayerHealth>(player1).unwrap();
            health1.current = 50.0;
        }
        
        let health1 = app.world().get::<PlayerHealth>(player1).unwrap();
        assert_eq!(health1.current, 50.0);
        
        // If we get here without panicking, basic multiplayer setup works
        assert!(true);
    }
}
