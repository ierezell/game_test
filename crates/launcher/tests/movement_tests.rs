mod common;
use bevy::prelude::*;
use common::{setup_two_player_game, get_spawned_players, simulate_player_movement, simulate_player_look, get_entity_position, assert_entity_moved};

#[test]
fn test_player_movement() {
    let (mut server, mut client1, mut client2) = setup_two_player_game();
    
    // Get the players
    let client1_players = get_spawned_players(client1.world_mut());
    let server_players = get_spawned_players(server.world_mut());
    
    assert!(!client1_players.is_empty(), "Should have at least one player on client1");
    assert!(!server_players.is_empty(), "Should have at least one player on server");
    
    let client1_player = client1_players[0];
    let server_player = server_players[0];
    
    // Record initial positions
    let initial_client_pos = get_entity_position(client1.world(), client1_player)
        .expect("Player should have position");
    let initial_server_pos = get_entity_position(server.world(), server_player)
        .expect("Player should have position");
    
    // Simulate player movement
    simulate_player_movement(client1.world_mut(), client1_player, Vec2::new(1.0, 0.0));
    simulate_player_look(client1.world_mut(), client1_player, Vec2::new(0.1, 0.0));
    
    // Run updates to process movement
    for _ in 0..100 {
        server.update();
        client1.update();
        client2.update();
    }
    
    // Assert player was able to move and it was reflected on server and other clients
    assert_entity_moved(client1.world(), client1_player, initial_client_pos, 0.1);
    assert_entity_moved(server.world(), server_player, initial_server_pos, 0.1);
}
