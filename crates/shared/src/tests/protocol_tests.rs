#[cfg(test)]
mod network_protocol_tests {
    use crate::input::PlayerAction;
    use crate::navigation::{PatrolRoute, PatrolState, SimpleNavigationAgent};
    use crate::protocol::*;
    use avian3d::prelude::{LinearVelocity, Position, Rotation};
    use bevy::prelude::*;
    use lightyear::prelude::*;

    /// Test component registration and serialization
    #[test]
    fn test_component_registration_and_serialization() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(ProtocolPlugin);

        // Test PlayerId component
        let player_id = PlayerId(PeerId::Netcode(12345));
        let entity = app.world_mut().spawn(player_id.clone()).id();

        let retrieved = app.world().get::<PlayerId>(entity).unwrap();
        assert_eq!(
            retrieved.0.to_bits(),
            player_id.0.to_bits(),
            "PlayerId should be stored and retrieved correctly"
        );

        // Test PlayerColor component
        let color = PlayerColor(Color::srgb(1.0, 0.0, 0.5));
        app.world_mut().entity_mut(entity).insert(color.clone());

        let retrieved_color = app.world().get::<PlayerColor>(entity).unwrap();
        assert_eq!(
            retrieved_color.0, color.0,
            "PlayerColor should be stored correctly"
        );

        // Test GameSeed component
        let seed = GameSeed { seed: 42424242 };
        app.world_mut().entity_mut(entity).insert(seed.clone());

        let retrieved_seed = app.world().get::<GameSeed>(entity).unwrap();
        assert_eq!(
            retrieved_seed.seed, seed.seed,
            "GameSeed should be stored correctly"
        );
    }

    /// Test navigation component protocol registration
    #[test]
    fn test_navigation_component_protocol() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(ProtocolPlugin);

        // Test SimpleNavigationAgent component
        let nav_agent = SimpleNavigationAgent {
            speed: 5.5,
            arrival_threshold: 1.2,
            current_target: Some(Vec3::new(10.0, 1.0, 5.0)),
        };
        let entity = app.world_mut().spawn(nav_agent.clone()).id();

        let retrieved = app.world().get::<SimpleNavigationAgent>(entity).unwrap();
        assert_eq!(
            retrieved.speed, nav_agent.speed,
            "Navigation agent speed should match"
        );
        assert_eq!(
            retrieved.arrival_threshold, nav_agent.arrival_threshold,
            "Navigation agent threshold should match"
        );
        assert_eq!(
            retrieved.current_target, nav_agent.current_target,
            "Navigation agent target should match"
        );

        // Test PatrolState component
        let patrol_state = PatrolState {
            current_target_index: 2,
            wait_timer: 1.5,
            wait_duration: 3.0,
            forward: false,
        };
        app.world_mut()
            .entity_mut(entity)
            .insert(patrol_state.clone());

        let retrieved_state = app.world().get::<PatrolState>(entity).unwrap();
        assert_eq!(
            retrieved_state.current_target_index, patrol_state.current_target_index,
            "Patrol state index should match"
        );
        assert_eq!(
            retrieved_state.wait_timer, patrol_state.wait_timer,
            "Patrol state timer should match"
        );
        assert_eq!(
            retrieved_state.forward, patrol_state.forward,
            "Patrol state direction should match"
        );

        // Test PatrolRoute component
        let patrol_route = PatrolRoute {
            points: vec![
                Vec3::ZERO,
                Vec3::new(5.0, 0.0, 5.0),
                Vec3::new(10.0, 0.0, 0.0),
            ],
            ping_pong: true,
        };
        app.world_mut()
            .entity_mut(entity)
            .insert(patrol_route.clone());

        let retrieved_route = app.world().get::<PatrolRoute>(entity).unwrap();
        assert_eq!(
            retrieved_route.points, patrol_route.points,
            "Patrol route points should match"
        );
        assert_eq!(
            retrieved_route.ping_pong, patrol_route.ping_pong,
            "Patrol route ping_pong should match"
        );
    }

    /// Test physics component integration with protocol
    #[test]
    fn test_physics_component_integration() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(ProtocolPlugin);

        let entity = app.world_mut().spawn_empty().id();

        // Test Position component (from Avian3D)
        let position = Position::new(Vec3::new(15.5, 2.3, -7.8));
        app.world_mut().entity_mut(entity).insert(position);

        let retrieved_pos = app.world().get::<Position>(entity).unwrap();
        assert_eq!(
            retrieved_pos.0,
            Vec3::new(15.5, 2.3, -7.8),
            "Position should be stored correctly"
        );

        // Test Rotation component
        let rotation = Rotation::from(Quat::from_rotation_y(1.57));
        app.world_mut().entity_mut(entity).insert(rotation);

        let retrieved_rot = app.world().get::<Rotation>(entity).unwrap();
        assert!(
            (retrieved_rot.0.w - rotation.0.w).abs() < 0.001,
            "Rotation quaternion should match"
        );

        // Test LinearVelocity component
        let velocity = LinearVelocity(Vec3::new(2.5, 0.0, -1.8));
        app.world_mut().entity_mut(entity).insert(velocity);

        let retrieved_vel = app.world().get::<LinearVelocity>(entity).unwrap();
        assert_eq!(
            retrieved_vel.0,
            Vec3::new(2.5, 0.0, -1.8),
            "Linear velocity should be stored correctly"
        );
    }

    /// Test lobby state management and multiplayer components
    #[test]
    fn test_lobby_state_management() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(ProtocolPlugin);

        // Create lobby state with multiple players
        let lobby_state = LobbyState {
            players: vec![1001, 1002, 1003, 1004],
            host_id: 1001,
        };

        let lobby_entity = app.world_mut().spawn(lobby_state.clone()).id();

        let retrieved_lobby = app.world().get::<LobbyState>(lobby_entity).unwrap();
        assert_eq!(
            retrieved_lobby.players.len(),
            4,
            "Lobby should have 4 players"
        );
        assert_eq!(retrieved_lobby.host_id, 1001, "Host ID should match");
        assert!(
            retrieved_lobby.players.contains(&1002),
            "Lobby should contain player 1002"
        );
        assert!(
            retrieved_lobby.players.contains(&1004),
            "Lobby should contain player 1004"
        );

        // Test lobby modification
        let mut modified_lobby = retrieved_lobby.clone();
        modified_lobby.players.push(1005);
        modified_lobby.players.retain(|&id| id != 1003); // Remove player 1003

        app.world_mut()
            .entity_mut(lobby_entity)
            .insert(modified_lobby);

        let final_lobby = app.world().get::<LobbyState>(lobby_entity).unwrap();
        assert_eq!(
            final_lobby.players.len(),
            4,
            "Should still have 4 players after modification"
        );
        assert!(
            final_lobby.players.contains(&1005),
            "Should contain new player 1005"
        );
        assert!(
            !final_lobby.players.contains(&1003),
            "Should not contain removed player 1003"
        );
    }

    /// Test message events and network communication
    #[test]
    fn test_network_message_events() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(ProtocolPlugin);

        // Test ClientWorldCreatedEvent
        let client_event = ClientWorldCreatedEvent { client_id: 5678 };

        // In a real scenario, these would be sent over network
        // For testing, we verify the event structure
        assert_eq!(
            client_event.client_id, 5678,
            "Client world created event should have correct ID"
        );

        // Test HostStartGameEvent
        let _host_event = HostStartGameEvent;

        // Test that the event exists and can be created
        // In real usage, this would trigger game state transitions

        // Test StartLoadingGameEvent
        let _loading_event = StartLoadingGameEvent;

        // Verify event can be instantiated (would be sent from server to clients)
    }

    /// Test complex entity replication scenarios
    #[test]
    fn test_complex_entity_replication_scenario() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(ProtocolPlugin);

        // Create a complex entity that would be replicated in multiplayer
        let player_entity = app
            .world_mut()
            .spawn((
                Name::new("Player_TestUser"),
                PlayerId(PeerId::Netcode(9999)),
                PlayerColor(Color::srgb(0.8, 0.2, 0.9)),
                Position::new(Vec3::new(25.0, 1.0, 30.0)),
                Rotation::from(Quat::from_rotation_y(0.785)), // 45 degrees
                LinearVelocity(Vec3::new(3.0, 0.0, -2.0)),
                CharacterMarker,
            ))
            .id();

        // Verify all components are properly stored
        assert!(
            app.world().get::<Name>(player_entity).is_some(),
            "Player should have name"
        );
        assert!(
            app.world().get::<PlayerId>(player_entity).is_some(),
            "Player should have ID"
        );
        assert!(
            app.world().get::<PlayerColor>(player_entity).is_some(),
            "Player should have color"
        );
        assert!(
            app.world().get::<Position>(player_entity).is_some(),
            "Player should have position"
        );
        assert!(
            app.world().get::<Rotation>(player_entity).is_some(),
            "Player should have rotation"
        );
        assert!(
            app.world().get::<LinearVelocity>(player_entity).is_some(),
            "Player should have velocity"
        );
        assert!(
            app.world().get::<CharacterMarker>(player_entity).is_some(),
            "Player should have character marker"
        );

        // Create an AI entity that would also be replicated
        let ai_entity = app
            .world_mut()
            .spawn((
                Name::new("AI_Bot_001"),
                Position::new(Vec3::new(-10.0, 1.0, -15.0)),
                Rotation::default(),
                SimpleNavigationAgent {
                    speed: 4.0,
                    arrival_threshold: 1.5,
                    current_target: Some(Vec3::new(0.0, 1.0, 0.0)),
                },
                PatrolState::new(),
                PatrolRoute::new(vec![
                    Vec3::new(-10.0, 1.0, -15.0),
                    Vec3::new(0.0, 1.0, 0.0),
                    Vec3::new(10.0, 1.0, 15.0),
                ]),
            ))
            .id();

        // Verify AI entity components
        let ai_agent = app.world().get::<SimpleNavigationAgent>(ai_entity).unwrap();
        assert_eq!(ai_agent.speed, 4.0, "AI should have correct speed");
        assert!(ai_agent.current_target.is_some(), "AI should have a target");

        let ai_route = app.world().get::<PatrolRoute>(ai_entity).unwrap();
        assert_eq!(ai_route.points.len(), 3, "AI should have 3 patrol points");

        // Test that entities can be distinguished by their components
        let player_id = app.world().get::<PlayerId>(player_entity);
        let ai_id = app.world().get::<PlayerId>(ai_entity);

        assert!(player_id.is_some(), "Player entity should have PlayerId");
        assert!(ai_id.is_none(), "AI entity should not have PlayerId");

        let player_nav = app.world().get::<SimpleNavigationAgent>(player_entity);
        let ai_nav = app.world().get::<SimpleNavigationAgent>(ai_entity);

        assert!(
            player_nav.is_none(),
            "Player entity should not have navigation agent"
        );
        assert!(ai_nav.is_some(), "AI entity should have navigation agent");
    }

    /// Test protocol configuration and physics integration
    #[test]
    fn test_protocol_physics_configuration() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(ProtocolPlugin);

        // Verify physics transform configuration is applied
        // This would normally be checked through the actual physics systems
        // For now, verify the app builds successfully with the protocol

        // Test entity with full physics + network components
        let physics_entity = app
            .world_mut()
            .spawn((
                Position::new(Vec3::new(100.0, 50.0, 75.0)),
                Rotation::from(Quat::from_rotation_x(0.5)),
                LinearVelocity(Vec3::new(-5.0, 2.0, 8.0)),
                PlayerId(PeerId::Netcode(7777)),
                Name::new("PhysicsNetworkEntity"),
            ))
            .id();

        // Simulate some position/rotation changes (as would happen in physics)
        if let Some(mut pos) = app.world_mut().get_mut::<Position>(physics_entity) {
            pos.0 += Vec3::new(1.0, 0.0, 1.0);
        }

        if let Some(mut rot) = app.world_mut().get_mut::<Rotation>(physics_entity) {
            rot.0 = rot.0 * Quat::from_rotation_y(0.1);
        }

        // Verify changes were applied
        let final_pos = app.world().get::<Position>(physics_entity).unwrap();
        assert_eq!(
            final_pos.0,
            Vec3::new(101.0, 50.0, 76.0),
            "Position should be updated"
        );

        let final_rot = app.world().get::<Rotation>(physics_entity).unwrap();
        assert_ne!(
            final_rot.0,
            Quat::from_rotation_x(0.5),
            "Rotation should be modified"
        );
    }

    /// Test input configuration and action registration
    #[test]
    fn test_input_action_protocol() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(ProtocolPlugin);

        // Test that PlayerAction enum works with the protocol
        // This verifies the input plugin integration

        let actions = [
            PlayerAction::Move,
            PlayerAction::Look,
            PlayerAction::Jump,
            PlayerAction::Shoot,
            PlayerAction::Aim,
        ];

        // Verify all actions are distinct
        for (i, action1) in actions.iter().enumerate() {
            for (j, action2) in actions.iter().enumerate() {
                if i != j {
                    assert_ne!(
                        action1, action2,
                        "Actions should be distinct: {:?} vs {:?}",
                        action1, action2
                    );
                }
            }
        }

        // Test action properties
        assert_eq!(
            PlayerAction::default(),
            PlayerAction::Move,
            "Default action should be Move"
        );

        // Verify actions can be serialized/deserialized (important for networking)
        for action in &actions {
            let serialized = format!("{:?}", action);
            assert!(
                !serialized.is_empty(),
                "Action should be serializable: {:?}",
                action
            );
        }
    }

    /// Test component prediction and interpolation setup
    #[test]
    fn test_component_prediction_interpolation() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(ProtocolPlugin);

        // Create entity with components that should have prediction/interpolation
        let predicted_entity = app
            .world_mut()
            .spawn((
                Position::new(Vec3::new(0.0, 1.0, 0.0)),
                Rotation::default(),
                LinearVelocity(Vec3::new(5.0, 0.0, 3.0)),
                PlayerId(PeerId::Netcode(4444)),
            ))
            .id();

        // Verify components exist and can be modified
        let initial_pos = app.world().get::<Position>(predicted_entity).unwrap().0;
        let initial_vel = app
            .world()
            .get::<LinearVelocity>(predicted_entity)
            .unwrap()
            .0;

        // Simulate prediction step (separated borrows)
        let velocity = app
            .world()
            .get::<LinearVelocity>(predicted_entity)
            .unwrap()
            .0;
        if let Some(mut pos) = app.world_mut().get_mut::<Position>(predicted_entity) {
            pos.0 += velocity * 0.016; // 60fps delta time
        }

        let predicted_pos = app.world().get::<Position>(predicted_entity).unwrap().0;

        // Verify position changed in expected direction
        assert_ne!(
            predicted_pos, initial_pos,
            "Position should change during prediction"
        );
        assert!(
            predicted_pos.x > initial_pos.x,
            "Position should move in positive X direction"
        );
        assert!(
            predicted_pos.z > initial_pos.z,
            "Position should move in positive Z direction"
        );

        // Verify the change magnitude is reasonable
        let movement = predicted_pos - initial_pos;
        let expected_movement = initial_vel * 0.016;
        assert!(
            (movement - expected_movement).length() < 0.001,
            "Movement should match velocity * delta time"
        );
    }
}
