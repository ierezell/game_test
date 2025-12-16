mod common;

#[cfg(test)]
mod test {
    use crate::common::test::{
        create_test_client, create_test_server, get_spawned_players, simulate_player_movement,
        wait_for_condition,
    };
    use avian3d::prelude::Position;
    use bevy::prelude::*;
    use client::ClientGameState;
    use lightyear::prelude::{MessageSender, MetadataChannel};
    use server::ServerGameState;
    use shared::protocol::HostStartGameEvent;

    #[test]
    fn test_movement_replication() {
        // 1. Setup server and client
        let mut server_app = create_test_server();
        let mut client_app = create_test_client(1, false, false, true, true); // Auto-join, headless

        // 2. Run updates to establish connection
        for _ in 0..100 {
            server_app.update();
            client_app.update();
        }

        // 3. Start game
        // In headless mode, we might need to manually trigger state change if networking isn't fully simulated
        // But let's try sending the event first
        let mut sender_query = client_app
            .world_mut()
            .query::<&mut MessageSender<HostStartGameEvent>>();
        let mut sender = sender_query.single_mut(client_app.world_mut()).unwrap();
        sender.send::<MetadataChannel>(HostStartGameEvent);

        for _ in 0..100 {
            server_app.update();
            client_app.update();
        }

        // Force state if needed (like in flow_test.rs)
        if *server_app
            .world()
            .resource::<State<ServerGameState>>()
            .get()
            == ServerGameState::Lobby
        {
            server_app
                .world_mut()
                .resource_mut::<NextState<ServerGameState>>()
                .set(ServerGameState::Playing);
            client_app
                .world_mut()
                .resource_mut::<NextState<ClientGameState>>()
                .set(ClientGameState::Playing);
        }

        for _ in 0..50 {
            server_app.update();
            client_app.update();
        }

        // 4. Get player entities
        let server_players = get_spawned_players(server_app.world_mut());
        let client_players = get_spawned_players(client_app.world_mut());

        assert!(
            !server_players.is_empty(),
            "Server should have spawned players"
        );
        assert!(
            !client_players.is_empty(),
            "Client should have spawned players"
        );

        let server_player = server_players[0];
        let client_player = client_players[0];

        // 5. Record initial positions
        let server_initial_pos = server_app.world().get::<Position>(server_player).unwrap().0;
        let client_initial_pos = client_app.world().get::<Position>(client_player).unwrap().0;

        println!("Server Initial Pos: {:?}", server_initial_pos);
        println!("Client Initial Pos: {:?}", client_initial_pos);

        // 6. Simulate movement on client
        // Move forward (negative Z)
        let move_input = Vec2::new(0.0, 1.0); // W key

        // Apply input for several frames
        for _ in 0..60 {
            simulate_player_movement(client_app.world_mut(), client_player, move_input);
            client_app.update();
            server_app.update();
        }

        // 7. Check final positions
        let server_final_pos = server_app.world().get::<Position>(server_player).unwrap().0;
        let client_final_pos = client_app.world().get::<Position>(client_player).unwrap().0;

        println!("Server Final Pos: {:?}", server_final_pos);
        println!("Client Final Pos: {:?}", client_final_pos);

        let server_moved = (server_final_pos - server_initial_pos).length();
        let client_moved = (client_final_pos - client_initial_pos).length();

        println!("Server Moved: {}", server_moved);
        println!("Client Moved: {}", client_moved);

        // Client should have moved (prediction)
        assert!(client_moved > 0.1, "Client should have moved locally");

        // Server MUST have moved (replication)
        // If this fails, it means the server didn't receive/process inputs
        assert!(
            server_moved > 0.1,
            "Server should have moved based on replicated inputs"
        );
    }
}
