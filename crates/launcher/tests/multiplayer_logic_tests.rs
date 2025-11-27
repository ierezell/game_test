mod common;

use bevy::prelude::*;
use bevy::winit::WinitPlugin;
use shared::{
    protocol::{CharacterMarker, PlayerId, LobbyState},
    entities::{PlayerPhysicsBundle, color_from_id},
    input::{PlayerAction, FpsController},
};
use leafwing_input_manager::prelude::ActionState;
use server::ServerGameState;
use avian3d::prelude::*;
use lightyear::prelude::PeerId;

/// Test player ID generation and uniqueness
#[test]
fn test_player_id_generation_and_uniqueness() {
    let player1 = PlayerId(PeerId::Netcode(1));
    let player2 = PlayerId(PeerId::Netcode(2));
    let player3 = PlayerId(PeerId::Netcode(1)); // Same as player1
    
    assert_ne!(player1, player2, "Different player IDs should not be equal");
    assert_eq!(player1, player3, "Same player IDs should be equal");
    
    // Test color generation for different players
    let id1 = match player1.0 { PeerId::Netcode(id) => id, _ => 0 };
    let id2 = match player2.0 { PeerId::Netcode(id) => id, _ => 0 };
    let id3 = match player3.0 { PeerId::Netcode(id) => id, _ => 0 };
    let color1 = color_from_id(id1);
    let color2 = color_from_id(id2);
    let color3 = color_from_id(id3);
    
    assert_ne!(color1, color2, "Different players should have different colors");
    assert_eq!(color1, color3, "Same player ID should generate same color");
}

/// Test character marker component
#[test]
fn test_character_marker_component() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    
    // Spawn player entity with character marker
    let player_entity = app.world_mut().spawn((
        CharacterMarker,
        PlayerId(PeerId::Netcode(1)),
        PlayerPhysicsBundle::default(),
        FpsController::default(),
        Transform::from_xyz(0.0, 1.0, 0.0),
    )).id();
    
    // Spawn NPC entity with character marker but no player ID
    let npc_entity = app.world_mut().spawn((
        CharacterMarker,
        Transform::from_xyz(5.0, 1.0, 5.0),
    )).id();
    
    // Query for all characters
    let mut character_query = app.world_mut().query_filtered::<Entity, With<CharacterMarker>>();
    let characters: Vec<Entity> = character_query.iter(app.world()).collect();
    
    assert_eq!(characters.len(), 2, "Should have 2 character entities");
    assert!(characters.contains(&player_entity), "Should include player entity");
    assert!(characters.contains(&npc_entity), "Should include NPC entity");
    
    // Query for players specifically (has both CharacterMarker and PlayerId)
    let mut player_query = app.world_mut().query_filtered::<(Entity, &PlayerId), With<CharacterMarker>>();
    let players: Vec<(Entity, &PlayerId)> = player_query.iter(app.world()).collect();
    
    assert_eq!(players.len(), 1, "Should have 1 player entity");
    assert_eq!(players[0].0, player_entity, "Player entity should match");
    assert_eq!(players[0].1, &PlayerId(PeerId::Netcode(1)), "Player ID should be 1");
}

/// Test lobby state structure and management
#[test]
fn test_lobby_state_structure() {
    let mut lobby = LobbyState {
        players: Vec::new(),
        host_id: 0,
    };
    
    // Test initial state
    assert_eq!(lobby.players.len(), 0, "Lobby should start empty");
    assert_eq!(lobby.host_id, 0, "Host ID should be 0 initially");
    
    // Test adding players
    lobby.players.push(1);
    lobby.players.push(2);
    assert_eq!(lobby.players.len(), 2, "Should have 2 players");
    
    // Test host assignment
    lobby.host_id = 1;
    assert_eq!(lobby.host_id, 1, "Host should be player 1");
    
    // Test lobby state manipulation
    assert!(lobby.players.contains(&1), "Should contain player 1");
    assert!(lobby.players.contains(&2), "Should contain player 2");
}

/// Test multiplayer entity spawning patterns
#[test]
fn test_multiplayer_entity_spawning() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    
    // Simulate spawning multiple players
    let spawn_positions = [
        Vec3::new(0.0, 1.0, 0.0),
        Vec3::new(5.0, 1.0, 0.0), 
        Vec3::new(0.0, 1.0, 5.0),
        Vec3::new(5.0, 1.0, 5.0),
    ];
    
    let mut player_entities = Vec::new();
    
    for (i, &pos) in spawn_positions.iter().enumerate() {
        let player_id = (i + 1) as u64;
        let entity = app.world_mut().spawn((
            CharacterMarker,
            PlayerId(PeerId::Netcode(player_id)),
            PlayerPhysicsBundle::default(),
            FpsController::default(),
            ActionState::<PlayerAction>::default(),
            Transform::from_xyz(pos.x, pos.y, pos.z),
        )).id();
        
        player_entities.push(entity);
    }
    
    assert_eq!(player_entities.len(), 4, "Should have spawned 4 players");
    
    // Verify all players have unique positions
    for (i, &entity) in player_entities.iter().enumerate() {
        let transform = app.world().get::<Transform>(entity).unwrap();
        let expected_pos = spawn_positions[i];
        assert_eq!(transform.translation, expected_pos, "Player should be at expected position");
        
        // Verify player has required components
        assert!(app.world().get::<CharacterMarker>(entity).is_some());
        assert!(app.world().get::<PlayerId>(entity).is_some());
        assert!(app.world().get::<FpsController>(entity).is_some());
        assert!(app.world().get::<ActionState<PlayerAction>>(entity).is_some());
    }
    
    // Verify player IDs are unique
    let mut player_ids = Vec::new();
    for &entity in &player_entities {
        let player_id = app.world().get::<PlayerId>(entity).unwrap();
        if let PeerId::Netcode(id) = player_id.0 {
            player_ids.push(id);
        }
    }
    player_ids.sort();
    assert_eq!(player_ids, vec![1, 2, 3, 4], "Player IDs should be unique and sequential");
}

/// Test server game state management
#[test] 
fn test_server_game_state_management() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::state::app::StatesPlugin);
    app.init_state::<ServerGameState>();
    
    // Test initial state
    let initial_state = app.world().resource::<State<ServerGameState>>();
    assert_eq!(**initial_state, ServerGameState::Lobby, "Should start in lobby state");
    
    // Test state transitions (simulate)
    app.world_mut().resource_mut::<NextState<ServerGameState>>().set(ServerGameState::Playing);
    app.update(); // Process state change
    
    let current_state = app.world().resource::<State<ServerGameState>>();
    assert_eq!(**current_state, ServerGameState::Playing, "Should transition to playing state");
}

/// Test player color assignment consistency
#[test]
fn test_player_color_assignment() {
    // Test that color assignment is deterministic and consistent
    let player_colors: Vec<_> = (1..=8).map(|id| color_from_id(id)).collect();
    
    // All colors should be different for different IDs
    for i in 0..player_colors.len() {
        for j in (i+1)..player_colors.len() {
            assert_ne!(
                player_colors[i], player_colors[j],
                "Players {} and {} should have different colors", i+1, j+1
            );
        }
    }
    
    // Same ID should always produce the same color
    for id in 1..=8 {
        let color1 = color_from_id(id);
        let color2 = color_from_id(id);
        assert_eq!(color1, color2, "Same player ID should always produce same color");
    }
    
    // Test edge cases
    let color_zero = color_from_id(0);
    let color_large = color_from_id(999999);
    
    // All colors should have proper alpha
    assert_eq!(color_zero.alpha(), 1.0, "Color should have full alpha");
    assert_eq!(color_large.alpha(), 1.0, "Color should have full alpha");
}

/// Test multiplayer physics isolation (component validation)
#[test]
fn test_multiplayer_physics_isolation() {
    let mut app = App::new();
    // Use minimal setup to avoid rendering system conflicts
    app.add_plugins((
        MinimalPlugins,
        AssetPlugin::default(),
    ));
    
    // Spawn multiple players with physics components
    let player1 = app.world_mut().spawn((
        CharacterMarker,
        PlayerId(PeerId::Netcode(1)),
        PlayerPhysicsBundle::default(),
        Transform::from_xyz(0.0, 5.0, 0.0),
    )).id();
    
    let player2 = app.world_mut().spawn((
        CharacterMarker,
        PlayerId(PeerId::Netcode(2)),
        PlayerPhysicsBundle::default(), 
        Transform::from_xyz(10.0, 5.0, 0.0),
    )).id();
    
    // Update to initialize components
    app.update();
    
    // Test that players have separate physics states
    let p1_transform = app.world().get::<Transform>(player1).unwrap();
    let p2_transform = app.world().get::<Transform>(player2).unwrap();
    
    // Verify players have different positions and maintain their state
    assert_ne!(p1_transform.translation, p2_transform.translation, "Players should have different positions");
    assert_eq!(p1_transform.translation, Vec3::new(0.0, 5.0, 0.0), "Player 1 position should be correct");
    assert_eq!(p2_transform.translation, Vec3::new(10.0, 5.0, 0.0), "Player 2 position should be correct");
    
    // Players should still exist and have their components
    assert!(app.world().get::<PlayerId>(player1).is_some(), "Player 1 should still exist");
    assert!(app.world().get::<PlayerId>(player2).is_some(), "Player 2 should still exist");
}

/// Test lobby capacity management
#[test]
fn test_lobby_capacity_management() {
    let mut lobby = LobbyState {
        players: Vec::new(),
        host_id: 0,
    };
    
    // Test adding players up to capacity
    for i in 1..=4 {
        lobby.players.push(i);
        assert!(lobby.players.len() <= 4, "Should not exceed max players");
    }
    
    assert_eq!(lobby.players.len(), 4, "Should have exactly 4 players");
    
    // Test lobby full condition
    let is_full = lobby.players.len() >= 4;
    assert!(is_full, "Lobby should be considered full");
    
    // Test player removal
    lobby.players.retain(|&id| id != 2); // Remove player 2
    assert_eq!(lobby.players.len(), 3, "Should have 3 players after removal");
    assert!(!lobby.players.contains(&2), "Player 2 should be removed");
    assert_eq!(lobby.players, vec![1, 3, 4], "Remaining players should be correct");
}

/// Test component synchronization setup
#[test]
fn test_component_synchronization_setup() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    
    // Spawn a networked player entity with all sync components
    let networked_player = app.world_mut().spawn((
        // Core identity components
        CharacterMarker,
        PlayerId(PeerId::Netcode(42)),
        
        // Physics components that should sync
        Transform::from_xyz(1.0, 2.0, 3.0),
        LinearVelocity(Vec3::new(0.5, 0.0, -0.2)),
        
        // Game logic components
        FpsController::default(),
        ActionState::<PlayerAction>::default(),
        
        // Physics components 
        PlayerPhysicsBundle::default(),
    )).id();
    
    // Verify all expected components are present
    let world = app.world();
    
    // Core components
    assert!(world.get::<CharacterMarker>(networked_player).is_some());
    assert!(world.get::<PlayerId>(networked_player).is_some());
    
    // Transform and physics
    let transform = world.get::<Transform>(networked_player).unwrap();
    assert_eq!(transform.translation, Vec3::new(1.0, 2.0, 3.0));
    
    let velocity = world.get::<LinearVelocity>(networked_player).unwrap();
    assert_eq!(velocity.0, Vec3::new(0.5, 0.0, -0.2));
    
    // Game logic
    assert!(world.get::<FpsController>(networked_player).is_some());
    assert!(world.get::<ActionState<PlayerAction>>(networked_player).is_some());
    
    // Physics bundle components
    assert!(world.get::<RigidBody>(networked_player).is_some());
    assert!(world.get::<Collider>(networked_player).is_some());
    assert!(world.get::<Mass>(networked_player).is_some());
}

/// Test player disconnection handling logic
#[test]
fn test_player_disconnection_handling() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    
    // Create a lobby with multiple players
    let mut lobby = LobbyState {
        players: vec![1, 2, 3, 4],
        host_id: 1,

    };
    
    // Simulate player 2 disconnecting
    lobby.players.retain(|&id| id != 2);
    assert_eq!(lobby.players, vec![1, 3, 4], "Player 2 should be removed");
    
    // If host disconnects, need to reassign host
    if lobby.host_id == 2 {
        lobby.host_id = *lobby.players.first().unwrap_or(&0);
    }
    assert_eq!(lobby.host_id, 1, "Host should remain player 1");
    
    // If the actual host (player 1) disconnects
    lobby.players.retain(|&id| id != 1);
    assert_eq!(lobby.players, vec![3, 4], "Player 1 (host) should be removed");
    
    // Reassign host to first remaining player
    lobby.host_id = *lobby.players.first().unwrap_or(&0);
    assert_eq!(lobby.host_id, 3, "Host should be reassigned to player 3");
    
    // Test final lobby state
    assert_eq!(lobby.players.len(), 2, "Should have 2 players after removal");
    assert!(!lobby.players.contains(&1), "Should not contain removed player 1 (former host)");
    assert!(lobby.players.contains(&3), "Should still contain player 3 (new host)");
    assert!(lobby.players.contains(&4), "Should still contain player 4");
}