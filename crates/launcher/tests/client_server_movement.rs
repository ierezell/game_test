mod common;

#[cfg(test)]
mod test {
    use crate::common::test::{
        get_entity_position, run_apps_updates, setup_two_player_game, simulate_player_movement,
    };
    use bevy::prelude::*;
    use lightyear::prelude::PeerId;
    use shared::protocol::PlayerId;

    // Force LobbyState update to trigger replication
    fn check_fixed_update(time: Res<Time>) {
       println!("FixedUpdate Running! Elapsed: {:?}", time.elapsed());
    }

    fn force_lobby_update(mut query: Query<&mut shared::protocol::LobbyState>) {
        for mut lobby in query.iter_mut() {
             // Just deref mut to trigger change detection
             let _ = &mut *lobby; 
        }
    }

    #[test]
    fn test_client_server_movement_sync() {
        // 1. Setup server and clients
        // This connects them and runs enough ticks to establish the session
        let (mut server, mut client1, mut client2) = setup_two_player_game();
        
        server.add_systems(Update, force_lobby_update);
        // server.add_systems(FixedUpdate, check_fixed_update);
        // client1.add_systems(FixedUpdate, check_fixed_update);
        // client2.add_systems(FixedUpdate, check_fixed_update);
        
        
        // Manual Force Start: Send HostStartGameEvent from both being sure one is connected
        use lightyear::prelude::MessageSender;
        use shared::protocol::HostStartGameEvent;
        use lightyear::prelude::MetadataChannel;
        
        println!("Forcing Game Start...");

        // Send from Client 1
        let mut sender1 = client1.world_mut().query::<&mut MessageSender<HostStartGameEvent>>().single_mut(client1.world_mut()).expect("Client 1 sender not found");
        sender1.send::<MetadataChannel>(HostStartGameEvent);

        // Send from Client 2
        let mut sender2 = client2.world_mut().query::<&mut MessageSender<HostStartGameEvent>>().single_mut(client2.world_mut()).expect("Client 2 sender not found");
        sender2.send::<MetadataChannel>(HostStartGameEvent);
        
        // Run updates to process the message
        {
            let mut apps = [&mut server, &mut client1, &mut client2];
            run_apps_updates(&mut apps, 20); // short wait for message
        }

        // Debug Server LobbyState
        let server_world = server.world_mut();
        // use shared::protocol::LobbyState; // Already imported at top if fixed, but let's just use full path or ensure single import
        if let Ok(lobby) = server_world.query::<&shared::protocol::LobbyState>().single(server_world) {
             println!("Server LobbyState Players: {:?}", lobby.players);
        }
        
        // Manual Force State Change on Server if not Playing
        use server::ServerGameState;
        let current_state = server.world().resource::<State<ServerGameState>>().get().clone();
        if current_state != ServerGameState::Playing {
            println!("Force switching Server to Playing state via NextState/State");
            server.world_mut().insert_resource(NextState::Pending(ServerGameState::Playing));
        }

        // Run updates to process State Transition and Spawning
        {
            let mut apps = [&mut server, &mut client1, &mut client2];
            run_apps_updates(&mut apps, 200);
        }

        let client1_id = PlayerId(PeerId::Netcode(1));

        // 2. Identify Player 1 on Client 1
        // We expect player entities to be spawned now.
        // 2. Identify Player 1 on Client 1
        use client::ClientGameState;
        let client_state = client1.world().resource::<State<ClientGameState>>().get();
        println!("Client 1 State: {:?}", client_state);

        println!("Listing all entities on Client 1:");
        let world = client1.world_mut();
        let mut count = 0;
        let mut query = world.query::<Entity>();
        for e in query.iter(world) {
            count += 1;
            // Print first 5 entities components if possible? 
            // Too hard to reflect all components easily in one line without Archetypes inspection.
            // Just count them.
        }
        println!("Total Entities on Client 1: {}", count);

        println!("Listing all entities with PlayerId on Client 1:");
        let mut query = world.query::<(Entity, &PlayerId)>();
        for (e, pid) in query.iter(world) {
           println!("Entity: {:?}, PlayerId: {:?}", e, pid);
        }
        
        // Debug Server entities too
        println!("Listing all entities with PlayerId on Server:");
        let server_world = server.world_mut();
        
        // Check for LobbyState on server
        use shared::protocol::LobbyState;
        let lobby_count = server_world.query::<&LobbyState>().iter(server_world).count();
        println!("Server LobbyState Count: {}", lobby_count);
        
        // List server entities with LobbyState and their components? 
        // Can't easily print components dynamic names, but we can check if it has Replicate.
        // Need to import Replicate? lightyear::prelude::Replicate is generic? 
        // No, Replicate is a component struct Replicate<C> or Replicate bundle?
        // In lightyear 0.18 it's a component `Replicate`.
        
        let mut server_query = server_world.query::<(Entity, &PlayerId)>();
        for (e, pid) in server_query.iter(server_world) {
            println!("Server Entity: {:?}, PlayerId: {:?}", e, pid);
        }
        
        println!("Checking Server for Client Connections and ReplicationSender:");
        use lightyear::prelude::server::ClientOf;
        use lightyear::prelude::ReplicationSender;
        // use bevy::core::Name; // Name is in prelude usually
        // Check for Client entities on Server
        let mut client_query = server_world.query::<(Entity, Option<&bevy::prelude::Name>, Option<&ReplicationSender>, Option<&ClientOf>)>();
        for (e, name, sender, client_of) in client_query.iter(server_world) {
             let has_repl = sender.is_some();
             let is_client_of = client_of.is_some();
             println!("Server Entity: {:?}, Name: {:?}, HasReplSender: {}, IsClientOf: {}", e, name, has_repl, is_client_of);
        }

        println!("Listing all entities on Client 1:");
        let world = client1.world_mut();
        let mut name_query = world.query::<(Entity, Option<&bevy::prelude::Name>)>();
        let count = name_query.iter(world).count();
        println!("Total Entities on Client 1: {}", count);
        for (e, name) in name_query.iter(world) {
            println!("Client Entity: {:?}, Name: {:?}", e, name);
        }

        let p1_on_client1 = world
            .query::<(Entity, &PlayerId)>()
            .iter(world)
            .find(|(_, pid)| **pid == client1_id)
            .map(|(e, _)| e)
            .expect("Player 1 should exist on Client 1");

        // 3. Identify Player 1 on Server
        let p1_on_server = server
            .world_mut()
            .query::<(Entity, &PlayerId)>()
            .iter(server.world())
            .find(|(_, pid)| **pid == client1_id)
            .map(|(e, _)| e)
            .expect("Player 1 should exist on Server");

        // 4. Record initial positions
        let initial_pos_client = get_entity_position(client1.world(), p1_on_client1)
            .expect("Player 1 on client should have position");
        let initial_pos_server = get_entity_position(server.world(), p1_on_server)
            .expect("Player 1 on server should have position");

        // Assert they start roughly at the same place (likely 0,0,0 or spawn point)
        assert!(
            (initial_pos_client - initial_pos_server).length() < 0.1,
            "Client and Server positions should match initially. Client: {:?}, Server: {:?}",
            initial_pos_client,
            initial_pos_server
        );

        println!("Initial Position: {:?}", initial_pos_server);

        // 5. Apply Movement Input
        // "1 acceleration" -> We'll assume this means full positive input on one axis.
        // We apply this input on the CLIENT. The client's InputPlugin should handle replication.
        // "1 second" -> We'll assume 60 ticks if fixed timestep is 60hz, or just iterate enough times.
        // common::run_apps_updates runs one update per app per cycle.
        
        let move_direction = Vec2::new(1.0, 0.0);
        let ticks = 60; // Approx 1 second

        let mut apps = [&mut server, &mut client1, &mut client2];
        
        for _ in 0..ticks {
            // Apply input every tick (simulate holding down key)
            simulate_player_movement(apps[1].world_mut(), p1_on_client1, move_direction);
            
            // Run updates
            run_apps_updates(&mut apps, 1);
        }

        // 6. Stop Input and let it settle
        // "do not move afterwards"
        // We stop applying input.
        // We run a few more ticks to let friction slow it down or just to verify it doesn't keep moving indefinitely.
        
        let stop_ticks = 30; // 0.5 seconds to settle
        for _ in 0..stop_ticks {
             // Explicitly set zero input or just don't set it (if it resets every frame?)
             // Leafwing Input Manager actions usually persist if pressed, but simulate_player_movement uses set_axis_pair.
             // If we don't call it, it might retain value depending on config.
             // Let's set it to zero to be sure "he releases the key".
             simulate_player_movement(apps[1].world_mut(), p1_on_client1, Vec2::ZERO);
             run_apps_updates(&mut apps, 1);
        }

        // 7. Verify Positions
        let final_pos_client = get_entity_position(apps[1].world(), p1_on_client1)
            .expect("Player 1 on client should have position");
        let final_pos_server = get_entity_position(apps[0].world(), p1_on_server)
            .expect("Player 1 on server should have position");

        println!("Final Client Pos: {:?}", final_pos_client);
        println!("Final Server Pos: {:?}", final_pos_server);

        // Assert moved significantly
        let distance_moved = (final_pos_server - initial_pos_server).length();
        assert!(
            distance_moved > 1.0, 
            "Player should have moved > 1.0 units. Moved: {}", distance_moved
        );

        // Assert Client and Server agree
        assert!(
            (final_pos_client - final_pos_server).length() < 0.5,
            "Client and Server positions should match. Client: {:?}, Server: {:?}, Diff: {}",
            final_pos_client,
            final_pos_server,
            (final_pos_client - final_pos_server).length()
        );

        // 8. Verify "do not move afterwards"
        // We'll run a few more ticks and ensure position doesn't change much more.
        let post_stop_pos = final_pos_server;
        
        let idle_ticks = 20;
        simulate_player_movement(apps[1].world_mut(), p1_on_client1, Vec2::ZERO);
        run_apps_updates(&mut apps, idle_ticks);

        let final_idle_pos = get_entity_position(apps[0].world(), p1_on_server)
            .expect("Player 1 on server should have position");

        assert!(
            (final_idle_pos - post_stop_pos).length() < 0.1,
            "Player should have stopped moving. Position changed from {:?} to {:?} after idle ticks.",
            post_stop_pos,
            final_idle_pos
        );
    }
}
