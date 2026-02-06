#[cfg(test)]
mod test {
    use bevy::MinimalPlugins;
    use bevy::ecs::world::World;
    use bevy::prelude::{
        App, AssetApp, AssetPlugin, DefaultPlugins, Image, Mesh, PluginGroup, Shader,
        StandardMaterial, default,
    };
    use bevy::state::app::AppExtStates;
    use bevy::window::WindowPlugin;
    use client::camera::ClientCameraPlugin;
    use client::{ClientGameState, Headless, LocalPlayerId};
    use lightyear::prelude::server::ServerPlugins;

    use server::ServerGameState;

    use std::time::Duration;

    use client::entities::ClientEntitiesPlugin;
    use client::game::ClientGameCyclePlugin;
    use client::inputs::ClientInputPlugin;
    use client::lobby::ClientLobbyPlugin;
    use client::network::ClientNetworkPlugin;

    use bevy::log::LogPlugin;

    use lightyear::prelude::client::ClientPlugins;

    use server::entities::ServerEntitiesPlugin;
    use server::lobby::ServerLobbyPlugin;
    use server::network::ServerNetworkPlugin;

    use shared::{NetworkMode, SharedPlugin};

    pub fn create_test_client_app(
        client_id: u64,
        crossbeam_io: client::network::CrossbeamClientEndpoint,
    ) -> App {
        let mut client_app = App::new();
        let client_id = if client_id == 0 { 1 } else { client_id };
        client_app.insert_resource(Headless(true));
        client_app.add_plugins(AssetPlugin {
            file_path: "../../../../assets".to_string(),
            ..Default::default()
        });

        client_app.init_asset::<Mesh>();
        client_app.init_asset::<StandardMaterial>();
        client_app.init_asset::<Shader>();
        client_app.init_asset::<Image>();

        client_app.add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: None,
                    exit_condition: bevy::window::ExitCondition::DontExit,
                    ..default()
                })
                .disable::<AssetPlugin>()
                .disable::<LogPlugin>()
                .disable::<bevy::winit::WinitPlugin>()
                .disable::<bevy::render::RenderPlugin>()
                .disable::<bevy::pbr::PbrPlugin>()
                .disable::<bevy::sprite::SpritePlugin>()
                .disable::<bevy::audio::AudioPlugin>()
                .disable::<bevy::gilrs::GilrsPlugin>()
                .disable::<bevy::ui::UiPlugin>()
                .disable::<bevy::text::TextPlugin>(),
        );

        client_app.insert_resource(NetworkMode::Crossbeam);
        client_app.insert_resource(crossbeam_io);
        client_app.insert_resource(shared::GymMode(true));
        client_app.add_plugins(SharedPlugin);
        client_app.add_plugins(ClientPlugins {
            tick_duration: Duration::from_secs_f64(1.0 / shared::FIXED_TIMESTEP_HZ),
        });

        client_app.insert_resource(LocalPlayerId(client_id));
        client_app.add_plugins(ClientNetworkPlugin);
        client_app.add_plugins(ClientInputPlugin);
        client_app.add_plugins(ClientCameraPlugin);

        client_app.add_plugins(ClientEntitiesPlugin);
        client_app.add_plugins(ClientLobbyPlugin);
        client_app.add_plugins(ClientGameCyclePlugin);

        client_app.init_state::<ClientGameState>();
        client_app.insert_state(ClientGameState::Lobby);

        client_app
    }

    pub fn create_test_server_app() -> App {
        let mut app = App::new();

        app.add_plugins((
            MinimalPlugins,
            bevy::state::app::StatesPlugin,
            bevy::diagnostic::DiagnosticsPlugin,
            bevy::asset::AssetPlugin::default(),
            bevy::scene::ScenePlugin,
            bevy::mesh::MeshPlugin,
            bevy::animation::AnimationPlugin,
        ));

        app.insert_resource(NetworkMode::Crossbeam);
        app.insert_resource(shared::GymMode(true)); // Use gym mode for tests
        app.add_plugins(SharedPlugin);
        app.add_plugins(ServerPlugins {
            tick_duration: Duration::from_secs_f64(1.0 / shared::FIXED_TIMESTEP_HZ),
        });
        app.add_plugins(ServerNetworkPlugin);
        app.add_plugins(ServerLobbyPlugin);
        app.add_plugins(ServerEntitiesPlugin);
        app.init_state::<ServerGameState>();
        app.insert_state(ServerGameState::Lobby);

        app
    }

    pub fn create_crossbeam_pair() -> (
        client::network::CrossbeamClientEndpoint,
        lightyear::crossbeam::CrossbeamIo,
    ) {
        let (client_io, server_io) = lightyear::crossbeam::CrossbeamIo::new_pair();
        (
            client::network::CrossbeamClientEndpoint(client_io),
            server_io,
        )
    }

    /// Helper to setup a server-side ClientOf entity for a crossbeam client connection
    pub fn add_server_clientof(
        server_app: &mut App,
        client_id: u64,
        server_io: lightyear::crossbeam::CrossbeamIo,
    ) {
        use lightyear::prelude::server::{ClientOf, Server};
        use lightyear::prelude::{Link, LinkOf, Linked, PingConfig, PingManager, Transport};
        use lightyear::prelude::{LocalId, PeerId, RemoteId};
        use lightyear::prelude::{ReplicationReceiver, ReplicationSender};
        use std::time::Duration;

        // Find the server entity
        let server_entity = server_app
            .world_mut()
            .query_filtered::<bevy::prelude::Entity, bevy::prelude::With<Server>>()
            .single(server_app.world())
            .expect("Server entity should exist");

        // Spawn a ClientOf entity linked to the server
        server_app.world_mut().spawn((
            ClientOf,
            LinkOf {
                server: server_entity,
            },
            Link::new(None),
            Linked, // Crossbeam is always immediately linked
            server_io,
            Transport::default(), // Add Transport for message passing
            RemoteId(PeerId::Netcode(client_id)),
            LocalId(PeerId::Server),
            PingManager::new(PingConfig {
                ping_interval: Duration::default(),
            }),
            ReplicationSender::default(),
            ReplicationReceiver::default(),
            bevy::prelude::Name::from(format!("ClientOf {}", client_id)),
        ));
    }

    #[test]
    pub fn test_app_creation() {
        let mut server_app = create_test_server_app();
        let (client1_io, _server1_io) = create_crossbeam_pair();
        let mut client_app = create_test_client_app(1, client1_io);

        for _ in 0..1000 {
            server_app.update();
            client_app.update();
        }
    }

    #[test]
    pub fn test_connection_between_client_and_server() {
        let mut server_app = create_test_server_app();
        let (client1_io, server1_io) = create_crossbeam_pair();
        let mut client_app = create_test_client_app(1, client1_io);

        // Let apps initialize
        for _ in 0..10 {
            server_app.update();
            client_app.update();
        }

        // Manually create server-side ClientOf entity for crossbeam connection
        add_server_clientof(&mut server_app, 1, server1_io);

        // Continue updating to allow connection to establish
        for _ in 0..990 {
            server_app.update();
            client_app.update();
        }

        {
            use lightyear::prelude::{Connected, RemoteId};

            // Validate server has a connected client link
            {
                use lightyear::connection::client_of::ClientOf;
                let world = server_app.world_mut();
                let mut q = world.query_filtered::<&RemoteId, (
                    bevy::prelude::With<ClientOf>,
                    bevy::prelude::With<Connected>,
                )>();
                let conn_count = q.iter(world).count();
                assert!(
                    conn_count >= 1,
                    "Server should have at least one connected client link, found {}",
                    conn_count
                );
            }

            // Validate each client has an active Client entity connected
            {
                use lightyear::prelude::Client;
                let w1 = client_app.world_mut();
                let mut q1 = w1.query_filtered::<bevy::prelude::Entity, (bevy::prelude::With<Client>, bevy::prelude::With<Connected>)>();
                let c1_connected = q1.iter(w1).count();
                assert!(
                    c1_connected >= 1,
                    "Client1 should have a connected Client entity"
                );
            }
        }
    }

    #[test]
    pub fn test_connection_between_two_client_and_server() {
        let mut server_app = create_test_server_app();
        let (client1_io, _server1_io) = create_crossbeam_pair();
        let (client2_io, _server2_io) = create_crossbeam_pair();
        let mut client_app1 = create_test_client_app(1, client1_io);
        let mut client_app2 = create_test_client_app(2, client2_io);

        for _ in 0..1000 {
            server_app.update();
            client_app1.update();
            client_app2.update();
        }

        // Ensure we reached Playing state on server and clients; wait for client lobby replication, then start
        {
            use lightyear::prelude::{Connected, RemoteId};

            // Validate server has two connected client links
            {
                use lightyear::connection::client_of::ClientOf;
                let world = server_app.world_mut();
                let mut q = world.query_filtered::<&RemoteId, (
                    bevy::prelude::With<ClientOf>,
                    bevy::prelude::With<Connected>,
                )>();
                let conn_count = q.iter(world).count();
                assert!(
                    conn_count >= 2,
                    "Server should have at least two connected client links, found {}",
                    conn_count
                );
            }

            // Validate each client has an active Client entity connected
            {
                use lightyear::prelude::Client;
                let w1 = client_app1.world_mut();
                let w2 = client_app2.world_mut();
                let mut q1 = w1.query_filtered::<bevy::prelude::Entity, (bevy::prelude::With<Client>, bevy::prelude::With<Connected>)>();
                let mut q2 = w2.query_filtered::<bevy::prelude::Entity, (bevy::prelude::With<Client>, bevy::prelude::With<Connected>)>();
                let c1_connected = q1.iter(w1).count();
                let c2_connected = q2.iter(w2).count();
                assert!(
                    c1_connected >= 1,
                    "Client1 should have a connected Client entity"
                );
                assert!(
                    c2_connected >= 1,
                    "Client2 should have a connected Client entity"
                );
            }
        }
    }

    #[test]
    pub fn test_lobby_state() {
        let mut server_app = create_test_server_app();
        let (client1_io, _server1_io) = create_crossbeam_pair();
        let (client2_io, _server2_io) = create_crossbeam_pair();
        let mut client_app1 = create_test_client_app(1, client1_io);
        let mut client_app2 = create_test_client_app(2, client2_io);

        for _ in 0..10 {
            server_app.update();
            client_app1.update();
            client_app2.update();
        }

        // Wait until both clients have replicated LobbyState with players and correct host
        {
            use shared::protocol::LobbyState;
            fn client_has_lobby(world: &mut bevy::prelude::World) -> bool {
                let mut q = world.query::<&LobbyState>();
                if let Some(lobby) = q.iter(world).next() {
                    !lobby.players.is_empty() && lobby.host_id == 1
                } else {
                    false
                }
            }
            let mut lobby_ticks = 0;
            while lobby_ticks < 100 {
                server_app.update();
                client_app1.update();
                client_app2.update();
                if client_has_lobby(client_app1.world_mut())
                    && client_has_lobby(client_app2.world_mut())
                {
                    break;
                }
                lobby_ticks += 1;
            }
            assert!(
                client_has_lobby(client_app1.world_mut())
                    && client_has_lobby(client_app2.world_mut()),
                "Clients should have replicated LobbyState before starting"
            );
        }

        // Validate lobby replicated state and host assignment
        {
            fn get_lobby(world: &mut World) -> Option<shared::protocol::LobbyState> {
                let mut q = world.query::<&shared::protocol::LobbyState>();
                q.iter(world).next().cloned()
            }
            let server_lobby =
                get_lobby(server_app.world_mut()).expect("Server should have LobbyState entity");
            assert!(
                server_lobby.players.len() >= 2,
                "Server lobby should have at least 2 players, found {}",
                server_lobby.players.len()
            );
            assert_eq!(
                server_lobby.host_id, 1,
                "Server lobby host should be client 1"
            );
        }
    }

    #[test]
    #[ignore = "Broken - being replaced by stepper_test::test_lobby_to_playing_with_stepper"]
    pub fn test_playing_state_reached() {
        let mut server_app = create_test_server_app();
        let (client1_io, server1_io) = create_crossbeam_pair();
        let (client2_io, server2_io) = create_crossbeam_pair();
        let mut client_app1 = create_test_client_app(1, client1_io);
        let mut client_app2 = create_test_client_app(2, client2_io);

        // Run server once to create the Server entity
        server_app.update();

        // Now create server-side ClientOf entities for crossbeam connections
        add_server_clientof(&mut server_app, 1, server1_io);
        add_server_clientof(&mut server_app, 2, server2_io);
        server_app.world_mut().flush();

        // Update client apps to spawn Client entities
        client_app1.update();
        client_app2.update();

        // Trigger Connect events for clients (like lightyear's stepper.init())
        let (client1_entity, client2_entity) = {
            use lightyear::prelude::client::Connect;

            // Find Client entities
            let client1_entity = client_app1
                .world_mut()
                .query_filtered::<bevy::prelude::Entity, bevy::prelude::With<lightyear::prelude::client::Client>>()
                .single(client_app1.world())
                .expect("Client1 entity should exist");

            let client2_entity = client_app2
                .world_mut()
                .query_filtered::<bevy::prelude::Entity, bevy::prelude::With<lightyear::prelude::client::Client>>()
                .single(client_app2.world())
                .expect("Client2 entity should exist");

            client_app1.world_mut().trigger(Connect {
                entity: client1_entity,
            });
            client_app2.world_mut().trigger(Connect {
                entity: client2_entity,
            });

            (client1_entity, client2_entity)
        };

        // Wait for clients to connect (like lightyear's wait_for_connection)
        {
            use lightyear::prelude::client::Connected;
            println!("Waiting for clients to connect...");

            for tick in 0..50 {
                server_app.update();
                client_app1.update();
                client_app2.update();

                // Check if Client entities have Connected component
                let c1_connected = client_app1
                    .world()
                    .get::<Connected>(client1_entity)
                    .is_some();
                let c2_connected = client_app2
                    .world()
                    .get::<Connected>(client2_entity)
                    .is_some();

                if tick % 10 == 0 {
                    println!(
                        "  Tick {}: c1_connected={}, c2_connected={}",
                        tick, c1_connected, c2_connected
                    );
                }

                if c1_connected && c2_connected {
                    println!("✓ All clients connected after {} ticks", tick);
                    break;
                }
            }
            println!("Connection wait loop completed");
        }

        // Verify all start in Lobby state
        {
            use bevy::prelude::State;
            let s_state = server_app
                .world()
                .resource::<State<ServerGameState>>()
                .get();
            let c1_state = client_app1
                .world()
                .resource::<State<ClientGameState>>()
                .get();
            let c2_state = client_app2
                .world()
                .resource::<State<ClientGameState>>()
                .get();

            assert_eq!(
                *s_state,
                ServerGameState::Lobby,
                "Server should start in Lobby"
            );
            assert_eq!(
                *c1_state,
                ClientGameState::Lobby,
                "Client1 should start in Lobby"
            );
            assert_eq!(
                *c2_state,
                ClientGameState::Lobby,
                "Client2 should start in Lobby"
            );
            println!("✓ All apps initialized in Lobby state");
        }

        // Trigger start: Host client sends HostStartGameEvent to server
        {
            use lightyear::prelude::{MessageSender, MetadataChannel};
            use shared::protocol::HostStartGameEvent;

            let w = client_app1.world_mut();
            let mut q = w.query::<&mut MessageSender<HostStartGameEvent>>();
            for mut sender in q.iter_mut(w) {
                println!("Client1 (host) sending HostStartGameEvent via Crossbeam");
                sender.send::<MetadataChannel>(HostStartGameEvent);
            }
        }

        // Wait for state transitions: Server receives message, transitions to Loading,
        // sends StartLoadingGameEvent back to clients, clients transition to Loading
        {
            use bevy::prelude::State;
            let mut ticks = 0;
            while ticks < 300 {
                server_app.update();
                client_app1.update();
                client_app2.update();

                let s = server_app
                    .world()
                    .get_resource::<State<ServerGameState>>()
                    .map(|st: &State<ServerGameState>| st.get().clone());
                let c1 = client_app1
                    .world()
                    .get_resource::<State<ClientGameState>>()
                    .map(|st: &State<ClientGameState>| st.get().clone());
                let c2 = client_app2
                    .world()
                    .get_resource::<State<ClientGameState>>()
                    .map(|st: &State<ClientGameState>| st.get().clone());

                // Check if all reached Loading state
                if s == Some(ServerGameState::Loading)
                    && c1 == Some(ClientGameState::Loading)
                    && c2 == Some(ClientGameState::Loading)
                {
                    println!("✓ All apps reached Loading state after {} ticks", ticks);
                    break;
                }
                ticks += 1;
            }

            let s_state = server_app
                .world()
                .resource::<State<ServerGameState>>()
                .get();
            let c1_state = client_app1
                .world()
                .resource::<State<ClientGameState>>()
                .get();
            let c2_state = client_app2
                .world()
                .resource::<State<ClientGameState>>()
                .get();

            assert_eq!(
                *s_state,
                ServerGameState::Loading,
                "Server should transition to Loading state after receiving HostStartGameEvent"
            );
            assert_eq!(
                *c1_state,
                ClientGameState::Loading,
                "Client1 should transition to Loading state after receiving StartLoadingGameEvent"
            );
            assert_eq!(
                *c2_state,
                ClientGameState::Loading,
                "Client2 should transition to Loading state after receiving StartLoadingGameEvent"
            );
        }

        // Continue running to allow gym environment to spawn and transition to Playing
        // TODO: Add verification for gym environment (4 walls + floor + ceiling)
        {
            use bevy::prelude::State;
            let mut ticks = 0;
            while ticks < 300 {
                server_app.update();
                client_app1.update();
                client_app2.update();

                let s = server_app
                    .world()
                    .get_resource::<State<ServerGameState>>()
                    .map(|st: &State<ServerGameState>| st.get().clone());
                let c1 = client_app1
                    .world()
                    .get_resource::<State<ClientGameState>>()
                    .map(|st: &State<ClientGameState>| st.get().clone());
                let c2 = client_app2
                    .world()
                    .get_resource::<State<ClientGameState>>()
                    .map(|st: &State<ClientGameState>| st.get().clone());

                // Check if all reached Playing state
                if s == Some(ServerGameState::Playing)
                    && c1 == Some(ClientGameState::Playing)
                    && c2 == Some(ClientGameState::Playing)
                {
                    println!(
                        "✓ All apps reached Playing state after {} additional ticks",
                        ticks
                    );
                    break;
                }
                ticks += 1;
            }

            let s_state = server_app
                .world()
                .resource::<State<ServerGameState>>()
                .get();
            let c1_state = client_app1
                .world()
                .resource::<State<ClientGameState>>()
                .get();
            let c2_state = client_app2
                .world()
                .resource::<State<ClientGameState>>()
                .get();

            assert_eq!(
                *s_state,
                ServerGameState::Playing,
                "Server should transition to Playing state after gym environment loads"
            );
            assert_eq!(
                *c1_state,
                ClientGameState::Playing,
                "Client1 should transition to Playing state after gym environment loads"
            );
            assert_eq!(
                *c2_state,
                ClientGameState::Playing,
                "Client2 should transition to Playing state after gym environment loads"
            );
        }

        // Verify gym environment loaded (4 walls + floor + ceiling = 6 static colliders)
        // Note: This verification depends on gym environment spawning logic
        println!("✓ Test passed: All state transitions completed successfully");
    }

    #[test]
    pub fn test_init_ressources_between_two_client_and_server() {
        use lightyear::prelude::{MessageSender, MetadataChannel};
        use shared::protocol::HostStartGameEvent;
        let mut server_app = create_test_server_app();
        let (client1_io, _server1_io) = create_crossbeam_pair();
        let (client2_io, _server2_io) = create_crossbeam_pair();
        let mut client_app1 = create_test_client_app(1, client1_io);
        let mut client_app2 = create_test_client_app(2, client2_io);

        for _ in 0..10 {
            server_app.update();
            client_app1.update();
            client_app2.update();
        }

        // Trigger start from client host via message (like clicking on the button)
        {
            let w = client_app1.world_mut();
            let mut q = w.query::<&mut MessageSender<HostStartGameEvent>>();
            for mut sender in q.iter_mut(w) {
                sender.send::<MetadataChannel>(HostStartGameEvent);
            }

            for _ in 0..100 {
                server_app.update();
                client_app1.update();
                client_app2.update();
            }
        }

        // Validate exactly two players are spawned on server and replicated to clients
        {
            use shared::protocol::{CharacterMarker, PlayerId};
            use std::collections::HashSet;

            // Server players match lobby count
            let server_world = server_app.world_mut();
            let lobby_players_len = {
                let mut q = server_world.query::<&shared::protocol::LobbyState>();
                let lobby = q
                    .iter(server_world)
                    .next()
                    .expect("Server should have LobbyState entity");
                lobby.players.len()
            };

            let mut q_server =
                server_world.query_filtered::<&PlayerId, bevy::prelude::With<CharacterMarker>>();

            let server_ids: HashSet<u64> = q_server
                .iter(server_world)
                .map(|pid| pid.0.to_bits())
                .collect();

            assert_eq!(
                server_ids.len(),
                lobby_players_len,
                "Server should spawn one player per lobby entry"
            );

            // Each client should have exactly those player ids replicated
            fn unique_client_player_ids(app: &mut App) -> HashSet<u64> {
                let world = app.world_mut();
                let mut q =
                    world.query_filtered::<&PlayerId, bevy::prelude::With<CharacterMarker>>();
                q.iter(world).map(|pid| pid.0.to_bits()).collect()
            }
            let c1_ids = unique_client_player_ids(&mut client_app1);
            let c2_ids = unique_client_player_ids(&mut client_app2);
            assert_eq!(
                c1_ids.len(),
                lobby_players_len,
                "Client 1 should have exactly {} replicated players",
                lobby_players_len
            );
            assert_eq!(
                c2_ids.len(),
                lobby_players_len,
                "Client 2 should have exactly {} replicated players",
                lobby_players_len
            );
            assert_eq!(
                c1_ids, server_ids,
                "Client 1 player ids should match server"
            );
            assert_eq!(
                c2_ids, server_ids,
                "Client 2 player ids should match server"
            );
        }

        //     // Validate server-side players have required components to play
        //     {
        //         use avian3d::prelude::{Collider, Position, RigidBody};
        //         use leafwing_input_manager::prelude::ActionState;
        //         use shared::inputs::input::PlayerAction;
        //         use shared::protocol::CharacterMarker;

        //         let world = server_app.world_mut();
        //         let mut q = world
        //             .query_filtered::<bevy::prelude::Entity, bevy::prelude::With<CharacterMarker>>();
        //         let entities: Vec<_> = q.iter(world).collect();
        //         // If players spawned, validate they have required components; otherwise skip gracefully
        //         for e in entities.iter() {
        //             assert!(
        //                 world.get::<Position>(*e).is_some(),
        //                 "Server player missing Position"
        //             );
        //             assert!(
        //                 world.get::<ActionState<PlayerAction>>(*e).is_some(),
        //                 "Server player missing ActionState<PlayerAction>"
        //             );
        //             assert!(
        //                 world.get::<Collider>(*e).is_some(),
        //                 "Server player missing Collider"
        //             );
        //             assert!(
        //                 world.get::<RigidBody>(*e).is_some(),
        //                 "Server player missing RigidBody"
        //             );
        //         }
        //         bevy::log::info!(
        //             "Validated {} server-side player entities (if zero, spawn may not have occurred yet)",
        //             entities.len()
        //         );
        //     }

        //     // Validate we have walls, collisions, etc. spawned from the level (on server)
        //     {
        //         use avian3d::prelude::{Collider, RigidBody};

        //         let world = server_app.world_mut();
        //         let mut q = world.query::<(&Collider, &RigidBody)>();
        //         let mut static_count = 0usize;
        //         for (_collider, rb) in q.iter(world) {
        //             if matches!(rb, RigidBody::Static) {
        //                 static_count += 1;
        //             }
        //         }
        //         if static_count == 0 {
        //             bevy::log::warn!(
        //                 "No static colliders found on server; level generation may not have run"
        //             );
        //         }
        //     }

        //     // Validate players have runtime components; skip strict stability in headless physics
        //     {
        //         use avian3d::prelude::{LinearVelocity, Position};
        //         use leafwing_input_manager::prelude::ActionState;
        //         use shared::inputs::movement::GroundState;
        //         use shared::protocol::{CharacterMarker, PlayerId};

        //         let world = server_app.world_mut();
        //         let mut q = world.query_filtered::<(
        //             bevy::prelude::Entity,
        //             &PlayerId,
        //             &Position,
        //             &LinearVelocity,
        //             Option<&GroundState>,
        //         ), bevy::prelude::With<CharacterMarker>>();

        //         let mut checked = 0usize;
        //         for (entity, pid, pos, vel, ground) in q.iter(world) {
        //             // Players spawn around y ~3.5 and should settle on the floor (> 0.3)
        //             assert!(
        //                 pos.0.y > 0.3,
        //                 "Player {} has invalid Y position ({}), likely falling",
        //                 pid.0.to_bits(),
        //                 pos.0.y
        //             );
        //             // No input: they shouldn't drift; allow tiny tolerance for settling
        //             let horiz_v = vel.0.length();
        //             assert!(
        //                 vel.0.y.abs() < 0.5,
        //                 "Player {} vertical velocity too high: {:?}",
        //                 pid.0.to_bits(),
        //                 vel.0
        //             );
        //             assert!(
        //                 horiz_v < 1.0,
        //                 "Player {} horizontal speed too high: {}",
        //                 pid.0.to_bits(),
        //                 horiz_v
        //             );
        //             // Server-side player should have input state component
        //             assert!(
        //                 world
        //                     .get::<ActionState<shared::inputs::input::PlayerAction>>(entity)
        //                     .is_some(),
        //                 "Server player {} missing ActionState<PlayerAction>",
        //                 pid.0.to_bits()
        //             );
        //             if let Some(gs) = ground {
        //                 assert!(
        //                     gs.is_grounded,
        //                     "Player {} not grounded despite level colliders",
        //                     pid.0.to_bits()
        //                 );
        //             }
        //             checked += 1;
        //         }
        //         bevy::log::info!(
        //             "Stability checked on {} server-side player entities",
        //             checked
        //         );
        //     }
    }
}
