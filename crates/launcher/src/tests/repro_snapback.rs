#[cfg(test)]
mod test {
    use crate::tests::common::test::{ClientServerStepper, simulate_player_movement};
    use avian3d::prelude::Position;
    use bevy::prelude::*;
    use lightyear::prelude::{MessageSender, MetadataChannel};
    use server::ServerGameState;
    use shared::protocol::HostStartGameEvent;

    #[test]
    fn test_movement_replication() {
        // 1. Create Stepper with 1 client
        let mut stepper = ClientServerStepper::new(1, true);
        stepper.init();

        // 2. Start game (trigger transition to Playing)
        let client_idx = 0;
        let client_app = &mut stepper.client_apps[client_idx];
        let mut sender_query = client_app
            .world_mut()
            .query::<&mut MessageSender<HostStartGameEvent>>();

        // Use iter_mut().next() to get the single component safely
        if let Some(mut sender) = sender_query.iter_mut(client_app.world_mut()).next() {
            sender.send::<MetadataChannel>(HostStartGameEvent);
        }

        // Run updates to propagate event
        for _ in 0..100 {
            stepper.frame_step();
        }

        // Force state if needed (stepper usually handles this if auto-start is irrelevant)
        // But let's check current state
        if *stepper
            .server_app
            .world()
            .resource::<State<ServerGameState>>()
            .get()
            == ServerGameState::Lobby
        {
            stepper
                .server_app
                .world_mut()
                .resource_mut::<NextState<ServerGameState>>()
                .set(ServerGameState::Playing);
        }

        // Wait for replication of spawn
        for _ in 0..100 {
            stepper.frame_step();
        }

        let client_world = stepper.client_world_mut(client_idx);
        let client_players: Vec<Entity> = client_world
            .query_filtered::<Entity, With<shared::protocol::PlayerId>>()
            .iter(client_world)
            .collect();

        assert!(
            !client_players.is_empty(),
            "Client should have spawned players"
        );
        let client_player = client_players[0];

        // 3. Record initial positions
        let initial_pos = stepper
            .client_world(client_idx)
            .get::<Position>(client_player)
            .expect("Player should have position")
            .0;

        println!("Client Initial Pos: {:?}", initial_pos);

        // 4. Simulate movement
        let move_input = Vec2::new(0.0, 1.0); // W key
        for _ in 0..60 {
            simulate_player_movement(
                stepper.client_apps[client_idx].world_mut(),
                client_player,
                move_input,
            );
            stepper.frame_step();
        }

        // 5. Verify movement
        let final_pos = stepper
            .client_world(client_idx)
            .get::<Position>(client_player)
            .expect("Player should have position")
            .0;

        println!("Client Final Pos: {:?}", final_pos);
        let moved = (final_pos - initial_pos).length();
        println!("Client Moved: {}", moved);

        assert!(moved > 0.1, "Client should have moved");
    }
}
