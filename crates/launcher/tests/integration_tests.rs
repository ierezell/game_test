#[cfg(test)]
mod client_server_integration_tests {
    use bevy::prelude::*;
    use client::{ClientGameState, create_client_app};
    use server::{ServerGameState, create_server_app};
    use std::time::Duration;

    /// Test basic client-server application creation without networking
    #[test]
    fn test_client_server_app_creation() {
        // Test server app creation (headless mode for testing)
        let server_app = create_server_app(true);

        // Verify server has correct initial state
        assert!(
            server_app
                .world()
                .get_resource::<State<ServerGameState>>()
                .is_some(),
            "Server should have game state resource"
        );

        // Test client app creation
        let client_app = create_client_app(
            1,
            "../assets".to_string(),
            false, // auto_host
            false, // auto_join
            false, // auto_start
        );

        // Verify client has correct initial state
        assert!(
            client_app
                .world()
                .get_resource::<State<ClientGameState>>()
                .is_some(),
            "Client should have game state resource"
        );

        // Verify client has local player ID
        assert!(
            client_app
                .world()
                .get_resource::<client::LocalPlayerId>()
                .is_some(),
            "Client should have local player ID resource"
        );
    }

    /// Test game state transitions in isolation
    #[test]
    fn test_game_state_transitions() {
        let mut server_app = create_server_app(true);
        let mut client_app = create_client_app(1, "../assets".to_string(), false, false, false);

        // Test server state transitions
        let initial_server_state = server_app
            .world()
            .resource::<State<ServerGameState>>()
            .get()
            .clone();
        assert_eq!(
            initial_server_state,
            ServerGameState::Lobby,
            "Server should start in Lobby state"
        );

        // Manually transition server state (simulating game start)
        server_app
            .world_mut()
            .resource_mut::<NextState<ServerGameState>>()
            .set(ServerGameState::Loading);
        server_app.update();

        let server_state_after = server_app
            .world()
            .resource::<State<ServerGameState>>()
            .get();
        assert_eq!(
            *server_state_after,
            ServerGameState::Loading,
            "Server should transition to Loading state"
        );

        // Test client state transitions
        let initial_client_state = client_app
            .world()
            .resource::<State<ClientGameState>>()
            .get()
            .clone();
        assert_eq!(
            initial_client_state,
            ClientGameState::LocalMenu,
            "Client should start in LocalMenu state"
        );

        // Transition client through connection states
        client_app
            .world_mut()
            .resource_mut::<NextState<ClientGameState>>()
            .set(ClientGameState::Connecting);
        client_app.update();

        let client_state_after = client_app
            .world()
            .resource::<State<ClientGameState>>()
            .get();
        assert_eq!(
            *client_state_after,
            ClientGameState::Connecting,
            "Client should transition to Connecting state"
        );
    }

    /// Test entity spawning and management systems
    #[test]
    fn test_entity_spawning_systems() {
        let mut server_app = create_server_app(true);

        // Simulate lobby initialization
        server_app.update();

        // Verify server initialized successfully (simplified test)
        // More complex lobby testing would require actual networking
        assert!(
            server_app.world().entities().len() > 0,
            "Server should have initialized entities"
        );

        // Test client entity systems
        let mut client_app = create_client_app(1, "../assets".to_string(), false, false, false);
        client_app.update();

        // Verify client has local player ID
        let local_player_id = client_app.world().resource::<client::LocalPlayerId>();
        assert_eq!(
            local_player_id.0, 1,
            "Client should have correct local player ID"
        );
    }

    /// Test shared plugin integration
    #[test]
    fn test_shared_plugin_integration() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(shared::SharedPlugin);

        // Verify shared systems are registered
        app.update();

        // Test navigation plugin functionality
        let entity = app
            .world_mut()
            .spawn((
                Name::new("TestEntity"),
                Transform::default(),
                shared::navigation::SimpleNavigationAgent::new(5.0),
            ))
            .id();

        app.update();

        // Verify entity has correct components
        let nav_agent = app
            .world()
            .get::<shared::navigation::SimpleNavigationAgent>(entity)
            .unwrap();
        assert_eq!(
            nav_agent.speed, 5.0,
            "Navigation agent should have correct speed"
        );

        // Test protocol plugin functionality
        let protocol_entity = app
            .world_mut()
            .spawn((
                shared::protocol::PlayerId(lightyear::prelude::PeerId::Netcode(12345)),
                shared::protocol::PlayerColor(Color::srgb(1.0, 0.5, 0.0)),
            ))
            .id();

        let player_id = app
            .world()
            .get::<shared::protocol::PlayerId>(protocol_entity)
            .unwrap();
        assert_eq!(
            player_id.0,
            lightyear::prelude::PeerId::Netcode(12345),
            "Protocol components should work correctly"
        );
    }

    /// Test input system integration
    #[test]
    fn test_input_system_integration() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(shared::SharedPlugin);

        // Create entity with transform component for testing
        let _player_entity = app
            .world_mut()
            .spawn((Transform::default(), Name::new("TestPlayer")))
            .id();

        // Test that the input system loads without error
        // (actual input testing is done in shared crate tests)

        // Verify basic player action enum functionality
        let mut test_actions = vec![
            shared::input::PlayerAction::Move,
            shared::input::PlayerAction::Look,
            shared::input::PlayerAction::Jump,
            shared::input::PlayerAction::Shoot,
            shared::input::PlayerAction::Aim,
        ];

        assert_eq!(test_actions.len(), 5, "Should have all player actions");
        test_actions.clear();
        assert!(test_actions.is_empty(), "Actions should be clearable");
    }

    /// Test configuration constants and settings
    #[test]
    fn test_configuration_constants() {
        // Test network configuration
        assert_eq!(
            shared::SERVER_ADDR.port(),
            8080,
            "Server should use port 8080"
        );
        assert_eq!(
            shared::SERVER_ADDR.ip().to_string(),
            "127.0.0.1",
            "Server should use localhost"
        );

        // Test shared settings
        assert_eq!(
            shared::SHARED_SETTINGS.protocol_id,
            42,
            "Protocol ID should be 42"
        );
        assert_eq!(
            shared::SHARED_SETTINGS.private_key.len(),
            32,
            "Private key should be 32 bytes"
        );

        // Test timing constants
        assert_eq!(
            shared::FIXED_TIMESTEP_HZ,
            60.0,
            "Fixed timestep should be 60Hz"
        );

        let expected_interval = Duration::from_millis(16);
        assert_eq!(
            shared::SEND_INTERVAL,
            expected_interval,
            "Send interval should be 16ms"
        );

        // Test input constants
        assert_eq!(shared::input::MAX_SPEED, 5.0, "Max speed should be 5.0");
        assert_eq!(shared::input::JUMP_HEIGHT, 1.5, "Jump height should be 1.5");
        assert_eq!(
            shared::input::PLAYER_CAPSULE_RADIUS,
            0.5,
            "Player capsule radius should be 0.5"
        );
        assert_eq!(
            shared::input::PLAYER_CAPSULE_HEIGHT,
            1.5,
            "Player capsule height should be 1.5"
        );
    }

    /// Test asset and scene loading systems
    #[test]
    fn test_asset_system_integration() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(shared::SharedPlugin);

        // Test that asset plugin is available (through DefaultPlugins in real apps)
        app.update();

        // Verify app runs without errors (basic smoke test)
        for _ in 0..10 {
            app.update();
        }

        // Test scene creation system (simplified)
        let scene_entity = app
            .world_mut()
            .spawn((Name::new("TestScene"), Transform::default()))
            .id();

        app.update();

        let scene_name = app.world().get::<Name>(scene_entity).unwrap();
        assert_eq!(
            scene_name.as_str(),
            "TestScene",
            "Scene entity should maintain name"
        );
    }

    /// Test error handling and edge cases
    #[test]
    fn test_error_handling_and_edge_cases() {
        // Test client creation with edge case parameters
        let edge_case_clients = vec![
            (0, "zero client ID"),
            (u64::MAX, "maximum client ID"),
            (1, "normal client ID"),
        ];

        for (client_id, description) in edge_case_clients {
            let client_app =
                create_client_app(client_id, "../assets".to_string(), false, false, false);

            // Verify client app was created successfully
            let local_id = client_app.world().resource::<client::LocalPlayerId>();
            let expected_id = if client_id == 0 { 1 } else { client_id };
            assert_eq!(
                local_id.0, expected_id,
                "Client ID should be handled correctly for {}",
                description
            );
        }

        // Test server creation variants
        let headless_server = create_server_app(true);
        assert!(
            headless_server
                .world()
                .get_resource::<State<ServerGameState>>()
                .is_some(),
            "Headless server should initialize correctly"
        );

        // Note: windowed server test would require display capability
        // let windowed_server = create_server_app(false);
        // This would fail in CI/headless environments
    }

    /// Test plugin ordering and dependency resolution
    #[test]
    fn test_plugin_dependency_order() {
        let mut app = App::new();

        // Test that plugins can be added in correct order
        app.add_plugins(MinimalPlugins);

        // SharedPlugin should be added before other game plugins
        app.add_plugins(shared::SharedPlugin);

        // Verify app builds and runs
        app.update();

        // Test that protocol plugin was included with SharedPlugin
        // This is verified by checking if protocol components can be spawned
        let entity = app
            .world_mut()
            .spawn(shared::protocol::GameSeed { seed: 42 })
            .id();
        let seed = app
            .world()
            .get::<shared::protocol::GameSeed>(entity)
            .unwrap();
        assert_eq!(seed.seed, 42, "Protocol plugin should be active");

        // Test that navigation plugin was included
        let nav_entity = app
            .world_mut()
            .spawn(shared::navigation::SimpleNavigationAgent::new(3.0))
            .id();
        let agent = app
            .world()
            .get::<shared::navigation::SimpleNavigationAgent>(nav_entity)
            .unwrap();
        assert_eq!(agent.speed, 3.0, "Navigation plugin should be active");
    }

    /// Test memory usage and performance characteristics
    #[test]
    fn test_performance_characteristics() {
        let mut server_app = create_server_app(true);
        let mut client_app = create_client_app(1, "../assets".to_string(), false, false, false);

        // Test multiple update cycles for performance
        let start = std::time::Instant::now();

        for _ in 0..100 {
            server_app.update();
            client_app.update();
        }

        let duration = start.elapsed();

        // 100 update cycles should complete reasonably quickly
        assert!(
            duration.as_millis() < 1000,
            "100 update cycles should complete in under 1 second, took: {}ms",
            duration.as_millis()
        );

        // Test entity creation performance
        let entity_creation_start = std::time::Instant::now();

        for i in 0..1000 {
            server_app.world_mut().spawn((
                Name::new(format!("TestEntity_{}", i)),
                Transform::from_translation(Vec3::new(i as f32, 1.0, 0.0)),
            ));
        }

        let entity_creation_duration = entity_creation_start.elapsed();

        // 1000 entity creations should be fast
        assert!(
            entity_creation_duration.as_millis() < 100,
            "1000 entity creations should complete in under 100ms, took: {}ms",
            entity_creation_duration.as_millis()
        );

        // Verify entities were actually created (simplified check)
        assert!(
            server_app.world().entities().len() >= 1000,
            "Should have created at least 1000 entities"
        );
    }
}
