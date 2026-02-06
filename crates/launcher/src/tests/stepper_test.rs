//! Tests using lightyear_tests::stepper for proper Crossbeam-based integration testing

#[cfg(test)]
mod test {
    use bevy::prelude::*;
    use bevy::state::app::AppExtStates;
    use lightyear::prelude::MessageSender;
    use lightyear_tests::stepper::{ClientServerStepper, ClientType, ServerType, StepperConfig};
    use std::time::Duration;

    use client::ClientGameState;
    use client::entities::ClientEntitiesPlugin;
    use client::lobby::ClientLobbyPlugin;
    use server::ServerGameState;
    use server::entities::ServerEntitiesPlugin;
    use server::lobby::ServerLobbyPlugin;
    use shared::protocol::{HostStartGameEvent, ProtocolPlugin};

    #[test]
    fn test_lobby_to_playing_with_stepper() {
        // Create stepper with 2 Netcode clients
        let mut stepper = ClientServerStepper::from_config(StepperConfig {
            tick_duration: Duration::from_millis(10),
            frame_duration: Duration::from_millis(10),
            clients: vec![ClientType::Netcode, ClientType::Netcode],
            server: ServerType::Netcode,
            init: false, // We'll add our plugins before calling init()
            ..default()
        });

        // Add server plugins
        stepper.server_app.add_plugins(ProtocolPlugin);
        stepper.server_app.add_plugins(ServerLobbyPlugin);
        stepper.server_app.add_plugins(ServerEntitiesPlugin);
        stepper.server_app.init_state::<ServerGameState>();
        stepper.server_app.insert_state(ServerGameState::Lobby);
        stepper.server_app.insert_resource(shared::GymMode(true));

        // Add client plugins
        for client_app in &mut stepper.client_apps {
            client_app.add_plugins(ProtocolPlugin);
            client_app.add_plugins(ClientLobbyPlugin);
            client_app.add_plugins(ClientEntitiesPlugin);
            client_app.init_state::<ClientGameState>();
            client_app.insert_state(ClientGameState::Lobby);
            client_app.insert_resource(shared::GymMode(true));
        }

        // Initialize connections
        stepper.init();

        println!("✓ Clients connected and synced");

        // Verify initial Lobby state
        assert_eq!(
            *stepper
                .server_app
                .world()
                .resource::<State<ServerGameState>>()
                .get(),
            ServerGameState::Lobby
        );
        println!("✓ Server in Lobby state");

        // Send HostStartGameEvent from client 0
        stepper
            .client_mut(0)
            .get_mut::<MessageSender<HostStartGameEvent>>()
            .unwrap()
            .send::<lightyear::prelude::MetadataChannel>(HostStartGameEvent);

        println!("✓ Client 0 sent HostStartGameEvent");

        // Advance to allow message processing
        stepper.frame_step(10);

        // Server should be in Loading
        let server_state = stepper
            .server_app
            .world()
            .resource::<State<ServerGameState>>()
            .get();
        assert_eq!(
            *server_state,
            ServerGameState::Loading,
            "Server should transition to Loading after receiving HostStartGameEvent"
        );
        println!("✓ Server transitioned to Loading");

        // Continue to Playing state
        stepper.frame_step(50);

        let server_state = stepper
            .server_app
            .world()
            .resource::<State<ServerGameState>>()
            .get();
        assert_eq!(
            *server_state,
            ServerGameState::Playing,
            "Server should transition to Playing after gym spawns"
        );
        println!("✓ Test PASSED: Lobby → Loading → Playing transitions successful!");
    }
}
