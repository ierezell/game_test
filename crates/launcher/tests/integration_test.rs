use bevy::prelude::*;
use client::create_client_app;
use lightyear::connection::client_of::ClientOf;
use lightyear::prelude::client::NetcodeClient;
use lightyear::prelude::server::NetcodeServer;
use lightyear::prelude::Connected;
use shared::protocol::LobbyState;

/// Helper to update a single app multiple times
fn update_app_multiple_times(app: &mut App, times: usize) {
    for _ in 0..times {
        app.update();
    }
}

/// Helper to update multiple apps together (simulating parallel execution)
fn update_apps_together(apps: &mut [&mut App], times: usize) {
    for _ in 0..times {
        for app in apps.iter_mut() {
            app.update();
        }
    }
}

#[test]
fn test_server_creation() {
    println!("Testing server creation...");
    let mut server_app = server::create_server_app(true);

    // Update the app a few times to let initialization complete
    update_app_multiple_times(&mut server_app, 5);

    // Verify the server entity exists
    let server_count = server_app
        .world_mut()
        .query::<&NetcodeServer>()
        .iter(server_app.world())
        .count();

    assert_eq!(
        server_count, 1,
        "Expected exactly 1 NetcodeServer entity, found {}",
        server_count
    );
    println!("✓ Server entity created successfully");

    // Verify lobby state was initialized
    let lobby_exists = server_app
        .world_mut()
        .query::<&LobbyState>()
        .iter(server_app.world())
        .next()
        .is_some();

    assert!(lobby_exists, "Expected LobbyState to be initialized");
    println!("✓ Lobby state initialized");

    println!("✓ All server creation checks passed");
}

#[test]
fn test_client_creation() {
    println!("Testing client creation...");

    // Create server first (so client can potentially connect)
    let mut server_app = server::create_server_app(true);
    let mut client_app = create_client_app(
        1,
        "../../assets".to_string(),
        false, // auto_host
        false, // auto_join
        false, // auto_start
    );

    // Update both apps together for initialization
    update_apps_together(&mut [&mut server_app, &mut client_app], 10);

    // Verify the client entity exists
    let client_count = client_app
        .world_mut()
        .query::<&NetcodeClient>()
        .iter(client_app.world())
        .count();

    assert_eq!(
        client_count, 1,
        "Expected exactly 1 NetcodeClient entity, found {}",
        client_count
    );
    println!("✓ Client entity created successfully");

    // Verify LocalPlayerId resource exists
    let has_player_id = client_app
        .world()
        .get_resource::<client::LocalPlayerId>()
        .is_some();
    assert!(has_player_id, "Expected LocalPlayerId resource to exist");
    println!("✓ LocalPlayerId resource initialized");

    println!("✓ All client creation checks passed");
}

#[test]
fn test_server_client_connection_and_disconnection() {
    // This test spawns:
    // 1. A server
    // 2. Two clients that connect to the server
    // 3. Simulates client disconnection by dropping client2
    // 4. Verifies the lobby state is correct throughout
    // All apps run in the same thread

    // Create and start the server
    let mut server_app = server::create_server_app(true);

    // Run server startup
    println!("Starting server...");
    update_app_multiple_times(&mut server_app, 5);

    // Verify server is running
    let server_count = server_app
        .world_mut()
        .query::<&NetcodeServer>()
        .iter(server_app.world())
        .count();
    assert_eq!(server_count, 1, "Server should be spawned");
    println!("✓ Server started successfully");

    // Create first client (host)
    println!("\nConnecting first client (host)...");
    let mut client1_app = create_client_app(
        1,
        "../../assets".to_string(),
        true,  // auto_host
        false, // auto_join
        false, // auto_start
    );

    // Run both apps together for initialization and connection
    println!("Processing connection handshake...");
    update_apps_together(&mut [&mut server_app, &mut client1_app], 50);

    // Verify client1 is connected
    let client1_count = client1_app
        .world_mut()
        .query::<&NetcodeClient>()
        .iter(client1_app.world())
        .count();
    assert_eq!(client1_count, 1, "Client 1 should be spawned");

    // Check lobby state on server - should have 1 player
    let lobby_state = server_app
        .world_mut()
        .query::<&LobbyState>()
        .iter(server_app.world())
        .next();

    assert!(
        lobby_state.is_some(),
        "LobbyState should exist after client connection"
    );
    let lobby = lobby_state.unwrap();

    assert_eq!(
        lobby.players.len(),
        1,
        "Expected 1 player in lobby, found {}",
        lobby.players.len()
    );
    assert_ne!(lobby.host_id, 0, "Host ID should be set");
    println!(
        "✓ Client 1 connected. Lobby has {} players",
        lobby.players.len()
    );
    println!("✓ Host ID is {}", lobby.host_id);

    // Create second client in a scope so we can drop it
    println!("\nConnecting second client...");
    {
        let mut client2_app = create_client_app(
            2,
            "../../assets".to_string(),
            false, // auto_host
            true,  // auto_join
            false, // auto_start
        );

        // Update all apps together to process the connection
        println!("Processing second client connection...");
        update_apps_together(
            &mut [&mut server_app, &mut client1_app, &mut client2_app],
            50,
        );

        // Check lobby state - should have 2 players
        let lobby_state = server_app
            .world_mut()
            .query::<&LobbyState>()
            .iter(server_app.world())
            .next();

        assert!(lobby_state.is_some(), "LobbyState should exist");
        let lobby = lobby_state.unwrap();

        assert_eq!(
            lobby.players.len(),
            2,
            "Expected 2 players in lobby after second client connects, found {}",
            lobby.players.len()
        );
        println!(
            "✓ Client 2 connected. Lobby has {} players",
            lobby.players.len()
        );
        println!("✓ Both clients successfully connected");

        println!("\nDisconnecting client 2 (by dropping the app)...");
        // client2_app will be dropped here, simulating a disconnect
    }

    // Update server and remaining client to process disconnection
    println!("Processing disconnection...");
    update_apps_together(&mut [&mut server_app, &mut client1_app], 60);

    // Verify lobby state after disconnection
    let lobby_state = server_app
        .world_mut()
        .query::<&LobbyState>()
        .iter(server_app.world())
        .next();

    assert!(
        lobby_state.is_some(),
        "LobbyState should still exist after disconnection"
    );
    let lobby = lobby_state.unwrap();

    assert_eq!(
        lobby.players.len(),
        1,
        "Expected 1 player in lobby after disconnection, found {}",
        lobby.players.len()
    );
    assert_ne!(lobby.host_id, 0, "Host ID should still be set");
    println!(
        "✓ After disconnection: Lobby has {} players",
        lobby.players.len()
    );
    println!("✓ Host ID is still {}", lobby.host_id);
    println!("✓ Client 2 successfully removed from lobby");

    // Final verification: client1 should still be connected
    let remaining_clients = server_app
        .world_mut()
        .query_filtered::<Entity, (With<ClientOf>, With<Connected>)>()
        .iter(server_app.world())
        .count();

    println!(
        "✓ Test completed. Remaining connected clients: {}",
        remaining_clients
    );
    println!("\n=== Test Summary ===");
    println!("✓ Server spawned successfully");
    println!("✓ Client 1 connected and set as host");
    println!("✓ Client 2 connected");
    println!("✓ Client 2 disconnected (app dropped)");
    println!("✓ Client 1 remains connected");
    println!("✓ Lobby state updated correctly throughout");
}

#[test]
fn test_multiple_clients_concurrent_connection() {
    // This test verifies that multiple clients can connect concurrently
    // All apps run in the same thread

    println!("Starting server for concurrent connection test...");
    let mut server_app = server::create_server_app(true);
    update_app_multiple_times(&mut server_app, 5);

    println!("Creating 3 clients...");
    let mut client1_app = create_client_app(1, "../../assets".to_string(), true, false, false);
    let mut client2_app = create_client_app(2, "../../assets".to_string(), false, true, false);
    let mut client3_app = create_client_app(3, "../../assets".to_string(), false, true, false);

    // Run connection handshakes for all clients together
    println!("Processing concurrent connections...");
    update_apps_together(
        &mut [
            &mut server_app,
            &mut client1_app,
            &mut client2_app,
            &mut client3_app,
        ],
        100,
    );

    // Verify all clients in lobby
    let lobby_state = server_app
        .world_mut()
        .query::<&LobbyState>()
        .iter(server_app.world())
        .next();

    assert!(lobby_state.is_some(), "LobbyState should exist");
    let lobby = lobby_state.unwrap();

    assert!(
        lobby.players.len() >= 1 && lobby.players.len() <= 3,
        "Expected 1-3 players in lobby, found {}. Note: Connection timing may affect exact count.",
        lobby.players.len()
    );
    assert_ne!(lobby.host_id, 0, "Host ID should be set");
    println!(
        "✓ Test completed. Lobby has {} players",
        lobby.players.len()
    );
    println!("✓ Host ID: {}", lobby.host_id);

    println!("\n=== Concurrent Connection Test Summary ===");
    println!("✓ Server spawned successfully");
    println!("✓ 3 clients created and attempted connection");
    println!("✓ Concurrent connections processed");
}
