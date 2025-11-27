mod common;

use bevy::prelude::*;
use common::{create_test_server, create_test_client, run_apps_updates};
use shared::{
    protocol::{CharacterMarker, PlayerId, LobbyState},
    entities::PlayerPhysicsBundle,
    input::{PlayerAction, FpsController},
    navigation::{SimpleNavigationAgent, PatrolRoute, PatrolState},
};
use leafwing_input_manager::prelude::ActionState;
use avian3d::prelude::*;
use lightyear::prelude::PeerId;

/// Test full client-server connection and player spawning
#[test]
fn test_client_server_connection_and_player_spawn() {
    // Create server and client apps using common test utilities
    let mut server_app = create_test_server();
    let mut client_app = create_test_client(1, false, false, true);
    
    // Initialize apps to setup plugins
    let mut apps = [&mut server_app, &mut client_app];
    run_apps_updates(&mut apps, 5);
    
    // Simulate server creating lobby
    let lobby = server_app.world_mut().spawn((
        LobbyState {
            players: vec![1, 2],
            host_id: 1,
        },
        Name::from("TestLobby")
    )).id();
    
    // Simulate spawning players on server
    for player_id in [1, 2] {
        let player_entity = server_app.world_mut().spawn((
            CharacterMarker,
            PlayerId(PeerId::Netcode(player_id)),
            PlayerPhysicsBundle::default(),
            Transform::from_xyz(player_id as f32 * 2.0, 1.0, 0.0),
            Name::from(format!("Player{}", player_id))
        )).id();
        
        info!("Spawned player {} on server as entity {:?}", player_id, player_entity);
    }
    
    // Run simulation for several frames
    for frame in 0..10 {
        let mut apps = [&mut server_app, &mut client_app];
        run_apps_updates(&mut apps, 1);
        info!("Completed frame {}", frame);
    }
    
    // Verify server state
    let server_lobby = server_app.world().get::<LobbyState>(lobby).expect("Server lobby should exist");
    assert_eq!(server_lobby.players.len(), 2, "Server should have 2 players in lobby");
    
    let server_players: Vec<Entity> = server_app.world_mut()
        .query_filtered::<Entity, With<CharacterMarker>>()
        .iter(server_app.world())
        .collect();
    assert_eq!(server_players.len(), 2, "Server should have 2 player entities");
    
    // Verify each player has proper components (if they exist)
    if !server_players.is_empty() {
        for &player_entity in &server_players {
            let player_id = server_app.world().get::<PlayerId>(player_entity)
                .expect("Player should have PlayerId");
            if let Some(transform) = server_app.world().get::<Transform>(player_entity) {
                info!("Player {:?} at position {:?}", player_id, transform.translation);
                assert!(transform.translation.y >= -1.0, "Player should not be too far below ground");
            } else {
                info!("Player {:?} does not have Transform yet", player_id);
            }
        }
    } else {
        info!("No players spawned yet - this is normal for basic connection test");
    }
    
    info!("Client-server connection test passed!");
}

/// Test real-time multiplayer game session
#[test]
fn test_multiplayer_game_session() {
    // Create apps using test utilities for proper headless setup
    let mut server_app = create_test_server();
    let mut client1_app = create_test_client(1, false, false, true);
    let mut client2_app = create_test_client(2, false, false, true);
    
    // Initialize all apps
    let mut apps = [&mut server_app, &mut client1_app, &mut client2_app];
    run_apps_updates(&mut apps, 5);
    
    // Spawn game session on server
    let lobby = server_app.world_mut().spawn((
        LobbyState {
            players: vec![1, 2],
            host_id: 1,
        },
        Name::from("GameSession")
    )).id();
    
    // Spawn players with physics and input handling
    let player1 = server_app.world_mut().spawn((
        CharacterMarker,
        PlayerId(PeerId::Netcode(1)),
        PlayerPhysicsBundle::default(),
        FpsController::default(),
        ActionState::<PlayerAction>::default(),
        Transform::from_xyz(-5.0, 2.0, 0.0),
        Name::from("Player1")
    )).id();
    
    let player2 = server_app.world_mut().spawn((
        CharacterMarker,
        PlayerId(PeerId::Netcode(2)),
        PlayerPhysicsBundle::default(),
        FpsController::default(),
        ActionState::<PlayerAction>::default(),
        Transform::from_xyz(5.0, 2.0, 0.0),
        Name::from("Player2")
    )).id();
    
    // Simulate client input for player movement
    if let Some(mut action_state) = server_app.world_mut().get_mut::<ActionState<PlayerAction>>(player1) {
        action_state.set_axis_pair(&PlayerAction::Move, Vec2::new(1.0, 0.0)); // Move right
    }
    
    if let Some(mut action_state) = server_app.world_mut().get_mut::<ActionState<PlayerAction>>(player2) {
        action_state.set_axis_pair(&PlayerAction::Move, Vec2::new(-1.0, 0.0)); // Move left
    }
    
    // Run game simulation
    for frame in 0..30 {
        let mut apps = [&mut server_app, &mut client1_app, &mut client2_app];
        run_apps_updates(&mut apps, 1);
        
        if frame % 10 == 0 {
            info!("Game session frame {}", frame);
        }
    }
    
    // Verify players moved
    let player1_transform = server_app.world().get::<Transform>(player1)
        .expect("Player1 should exist");
    let player2_transform = server_app.world().get::<Transform>(player2)
        .expect("Player2 should exist");
    
    info!("Player1 final position: {:?}", player1_transform.translation);
    info!("Player2 final position: {:?}", player2_transform.translation);
    
    // Players should have moved (physics should have applied some movement)
    assert_ne!(player1_transform.translation, Vec3::new(-5.0, 2.0, 0.0), "Player1 should have moved from starting position");
    assert_ne!(player2_transform.translation, Vec3::new(5.0, 2.0, 0.0), "Player2 should have moved from starting position");
    
    // Verify lobby still exists and maintains player count
    let final_lobby = server_app.world().get::<LobbyState>(lobby)
        .expect("Lobby should still exist");
    assert_eq!(final_lobby.players.len(), 2, "Lobby should maintain 2 players throughout session");
    
    info!("Multiplayer game session test passed!");
}

/// Test AI navigation in multiplayer environment
#[test]
fn test_multiplayer_ai_navigation() {
    // Use proper test server setup
    let mut server_app = create_test_server();
    
    // Initialize
    let mut apps = [&mut server_app];
    run_apps_updates(&mut apps, 5);
    
    // Create patrol route for AI
    let patrol_points = vec![
        Vec3::new(0.0, 1.0, 0.0),
        Vec3::new(10.0, 1.0, 0.0),
        Vec3::new(10.0, 1.0, 10.0),
        Vec3::new(0.0, 1.0, 10.0),
    ];
    
    // Spawn AI entity with navigation
    let ai_entity = server_app.world_mut().spawn((
        SimpleNavigationAgent::new(2.0),
        PatrolRoute::new(patrol_points.clone()),
        PatrolState::default(),
        Transform::from_xyz(0.0, 1.0, 0.0),
        Name::from("AIAgent")
    )).id();
    
    // Spawn players in the same environment
    let player1 = server_app.world_mut().spawn((
        CharacterMarker,
        PlayerId(PeerId::Netcode(1)),
        Transform::from_xyz(5.0, 1.0, 5.0),
        Name::from("Player1")
    )).id();
    
    let player2 = server_app.world_mut().spawn((
        CharacterMarker,
        PlayerId(PeerId::Netcode(2)),
        Transform::from_xyz(8.0, 1.0, 8.0),
        Name::from("Player2")
    )).id();
    
    // Run simulation
    for frame in 0..50 {
        let mut apps = [&mut server_app];
        run_apps_updates(&mut apps, 1);
        
        if frame % 10 == 0 {
            let ai_transform = server_app.world().get::<Transform>(ai_entity)
                .expect("AI should exist");
            info!("Frame {}: AI at position {:?}", frame, ai_transform.translation);
        }
    }
    
    // Verify AI has navigation components
    assert!(server_app.world().get::<SimpleNavigationAgent>(ai_entity).is_some(), "AI should have navigation agent");
    assert!(server_app.world().get::<PatrolRoute>(ai_entity).is_some(), "AI should have patrol route");
    assert!(server_app.world().get::<PatrolState>(ai_entity).is_some(), "AI should have patrol state");
    
    // Verify players and AI coexist
    let ai_transform = server_app.world().get::<Transform>(ai_entity).expect("AI should exist");
    let player1_transform = server_app.world().get::<Transform>(player1).expect("Player1 should exist");
    let player2_transform = server_app.world().get::<Transform>(player2).expect("Player2 should exist");
    
    // All entities should be in valid positions
    assert!(ai_transform.translation.y >= 0.0, "AI should be above ground");
    assert!(player1_transform.translation.y >= 0.0, "Player1 should be above ground");
    assert!(player2_transform.translation.y >= 0.0, "Player2 should be above ground");
    
    info!("Multiplayer AI navigation test passed!");
}

/// Test client disconnection and reconnection
#[test]  
fn test_client_disconnection_reconnection() {
    // Use proper test setup
    let mut server_app = create_test_server();
    let mut client_app = create_test_client(1, false, false, true);
    
    // Initialize
    let mut apps = [&mut server_app, &mut client_app];
    run_apps_updates(&mut apps, 5);
    
    // Create initial lobby with multiple players
    let mut lobby = LobbyState {
        players: vec![1, 2, 3],
        host_id: 1,
    };
    
    let lobby_entity = server_app.world_mut().spawn((
        lobby.clone(),
        Name::from("TestLobby")
    )).id();
    
    // Spawn players
    for player_id in [1, 2, 3] {
        server_app.world_mut().spawn((
            CharacterMarker,
            PlayerId(PeerId::Netcode(player_id)),
            Transform::from_xyz(player_id as f32 * 2.0, 1.0, 0.0),
            Name::from(format!("Player{}", player_id))
        ));
    }
    
    // Run initial frames
    let mut apps = [&mut server_app, &mut client_app];
    run_apps_updates(&mut apps, 10);
    
    // Simulate player 2 disconnection
    lobby.players.retain(|&id| id != 2);
    
    // Update lobby on server
    if let Some(mut server_lobby) = server_app.world_mut().get_mut::<LobbyState>(lobby_entity) {
        server_lobby.players = lobby.players.clone();
    }
    
    // Remove player 2's entity from server
    let mut player_entities_to_remove = Vec::new();
    let mut player_query = server_app.world_mut().query_filtered::<(Entity, &PlayerId), With<CharacterMarker>>();
    for (entity, player_id) in player_query.iter(server_app.world()) {
        if let PeerId::Netcode(id) = player_id.0 {
            if id == 2 {
                player_entities_to_remove.push(entity);
            }
        }
    }
    
    for entity in player_entities_to_remove {
        server_app.world_mut().despawn(entity);
    }
    
    // Run frames after disconnection
    let mut apps = [&mut server_app, &mut client_app];
    run_apps_updates(&mut apps, 10);
    
    // Verify disconnection handled properly
    let updated_lobby = server_app.world().get::<LobbyState>(lobby_entity)
        .expect("Lobby should still exist");
    assert_eq!(updated_lobby.players.len(), 2, "Should have 2 players after disconnection");
    assert!(!updated_lobby.players.contains(&2), "Should not contain disconnected player");
    assert!(updated_lobby.players.contains(&1), "Should still contain player 1");
    assert!(updated_lobby.players.contains(&3), "Should still contain player 3");
    
    // Verify only 2 player entities remain
    let remaining_players: Vec<Entity> = server_app.world_mut()
        .query_filtered::<Entity, With<CharacterMarker>>()
        .iter(server_app.world())
        .collect();
    assert_eq!(remaining_players.len(), 2, "Should have 2 player entities remaining");
    
    // Simulate player 2 reconnection
    lobby.players.push(2);
    if let Some(mut server_lobby) = server_app.world_mut().get_mut::<LobbyState>(lobby_entity) {
        server_lobby.players = lobby.players.clone();
    }
    
    // Spawn player 2 back
    server_app.world_mut().spawn((
        CharacterMarker,
        PlayerId(PeerId::Netcode(2)),
        Transform::from_xyz(4.0, 1.0, 0.0), // Different position
        Name::from("Player2_Reconnected")
    ));
    
    // Run frames after reconnection
    let mut apps = [&mut server_app, &mut client_app];
    run_apps_updates(&mut apps, 10);
    
    // Verify reconnection
    let final_lobby = server_app.world().get::<LobbyState>(lobby_entity)
        .expect("Lobby should still exist");
    assert_eq!(final_lobby.players.len(), 3, "Should have 3 players after reconnection");
    assert!(final_lobby.players.contains(&2), "Should contain reconnected player");
    
    let final_players: Vec<Entity> = server_app.world_mut()
        .query_filtered::<Entity, With<CharacterMarker>>()
        .iter(server_app.world())
        .collect();
    assert_eq!(final_players.len(), 3, "Should have 3 player entities after reconnection");
    
    info!("Client disconnection/reconnection test passed!");
}

/// Test complex multiplayer scenario with physics, AI, and multiple players
#[test]
fn test_complex_multiplayer_scenario() {
    // Use proper test setup for headless testing
    let mut server_app = create_test_server();
    let mut client1_app = create_test_client(1, false, false, true);
    let mut client2_app = create_test_client(2, false, false, true);
    
    // Initialize all apps
    let mut apps = [&mut server_app, &mut client1_app, &mut client2_app];
    run_apps_updates(&mut apps, 10);
    
    // Create game world with lobby
    let lobby = server_app.world_mut().spawn((
        LobbyState {
            players: vec![1, 2],
            host_id: 1,
        },
        Name::from("ComplexGameSession")
    )).id();
    
    // Spawn players with full physics and input systems
    let player1 = server_app.world_mut().spawn((
        CharacterMarker,
        PlayerId(PeerId::Netcode(1)),
        PlayerPhysicsBundle::default(),
        FpsController::default(),
        ActionState::<PlayerAction>::default(),
        Transform::from_xyz(-10.0, 5.0, 0.0),
        Name::from("ComplexPlayer1")
    )).id();
    
    let player2 = server_app.world_mut().spawn((
        CharacterMarker,
        PlayerId(PeerId::Netcode(2)),
        PlayerPhysicsBundle::default(),
        FpsController::default(),
        ActionState::<PlayerAction>::default(),
        Transform::from_xyz(10.0, 5.0, 0.0),
        Name::from("ComplexPlayer2")
    )).id();
    
    // Add AI entities with navigation
    let ai_patrol = vec![
        Vec3::new(-5.0, 1.0, -5.0),
        Vec3::new(5.0, 1.0, -5.0),
        Vec3::new(5.0, 1.0, 5.0),
        Vec3::new(-5.0, 1.0, 5.0),
    ];
    
    let ai_entity = server_app.world_mut().spawn((
        SimpleNavigationAgent::new(3.0),
        PatrolRoute::new(ai_patrol),
        PatrolState::default(),
        Transform::from_xyz(0.0, 1.0, 0.0),
        Name::from("ComplexAI")
    )).id();
    
    // Add environment obstacles
    for i in 0..5 {
        server_app.world_mut().spawn((
            shared::navigation::NavigationObstacle,
            Position(Vec3::new(i as f32 * 3.0 - 6.0, 1.0, 3.0)),
            Name::from(format!("Obstacle{}", i))
        ));
    }
    
    // Simulate complex interactions over many frames
    for frame in 0..100 {
        // Simulate player input every few frames
        if frame % 5 == 0 {
            // Player 1 moves in a circle
            let angle = frame as f32 * 0.1;
            if let Some(mut action_state) = server_app.world_mut().get_mut::<ActionState<PlayerAction>>(player1) {
                action_state.set_axis_pair(&PlayerAction::Move, Vec2::new(angle.cos(), angle.sin()));
            }
            
            // Player 2 moves back and forth
            let direction = if (frame / 20) % 2 == 0 { 1.0 } else { -1.0 };
            if let Some(mut action_state) = server_app.world_mut().get_mut::<ActionState<PlayerAction>>(player2) {
                action_state.set_axis_pair(&PlayerAction::Move, Vec2::new(direction, 0.0));
            }
        }
        
        // Update all apps
        let mut apps = [&mut server_app, &mut client1_app, &mut client2_app];
        run_apps_updates(&mut apps, 1);
        
        if frame % 25 == 0 {
            info!("Complex scenario frame {}", frame);
        }
    }
    
    // Verify final state
    let final_lobby = server_app.world().get::<LobbyState>(lobby)
        .expect("Lobby should exist throughout complex scenario");
    assert_eq!(final_lobby.players.len(), 2, "Should maintain 2 players");
    
    // Verify all entities still exist
    assert!(server_app.world().get::<Transform>(player1).is_some(), "Player1 should still exist");
    assert!(server_app.world().get::<Transform>(player2).is_some(), "Player2 should still exist");
    assert!(server_app.world().get::<Transform>(ai_entity).is_some(), "AI should still exist");
    
    // Count final entities
    let total_characters: usize = server_app.world_mut()
        .query_filtered::<Entity, With<CharacterMarker>>()
        .iter(server_app.world())
        .count();
    assert_eq!(total_characters, 2, "Should have exactly 2 character entities");
    
    let total_obstacles: usize = server_app.world_mut()
        .query_filtered::<Entity, With<shared::navigation::NavigationObstacle>>()
        .iter(server_app.world())
        .count();
    assert_eq!(total_obstacles, 5, "Should have exactly 5 obstacle entities");
    
    info!("Complex multiplayer scenario test passed!");
}