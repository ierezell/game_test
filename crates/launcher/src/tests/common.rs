#[cfg(test)]
pub mod test {
    use ::client::{ClientGameState, create_client_app};
    use ::server::{ServerGameState, create_server_app};
    use avian3d::prelude::*;
    use bevy::prelude::*;

    use leafwing_input_manager::prelude::ActionState;

    use shared::NetworkMode;
    use shared::input::PlayerAction;

    use std::net::{Ipv4Addr, SocketAddr};
    use std::time::Duration;

    // Legacy imports for restored functions
    use ::client::lobby::AutoStart;
    use ::launcher::{AutoHost, AutoJoin};
    use avian3d::collision::CollisionDiagnostics;
    use avian3d::dynamics::solver::SolverDiagnostics;
    use avian3d::spatial_query::SpatialQueryDiagnostics;
    use shared::protocol::{CharacterMarker, PlayerId};
    use std::thread;

    // Constants
    pub const TICK_DURATION: Duration = Duration::from_millis(16); // 60hz
    pub const SHARED_CONFIG_MODE: NetworkMode = NetworkMode::Crossbeam;

    pub struct ClientServerStepper {
        pub server_app: App,
        pub client_apps: Vec<App>,
        pub client_ids: Vec<u64>,
        pub frame_duration: Duration,
        pub current_time: std::time::Instant,
    }

    impl ClientServerStepper {
        pub fn new(num_clients: usize, headless: bool) -> Self {
            let server_app = create_server_app(headless, SHARED_CONFIG_MODE);

            let mut client_apps = Vec::new();
            let mut client_ids = Vec::new();

            for i in 0..num_clients {
                let client_id = (i + 1) as u64;
                let client_app = create_client_app(
                    client_id,
                    "../../assets".to_string(),
                    headless,
                    SHARED_CONFIG_MODE,
                );
                client_apps.push(client_app);
                client_ids.push(client_id);
            }

            Self {
                server_app,
                client_apps,
                client_ids,
                frame_duration: TICK_DURATION,
                current_time: std::time::Instant::now(),
            }
        }

        pub fn init(&mut self) {
            // 1. Initialize server and let it stabilize
            self.server_app.finish();
            self.server_app.cleanup();
            for _ in 0..20 {
                self.server_app.update();
            }

            // 2. Create channel pairs and store server ends temporarily
            let mut server_channels = Vec::new();

            for (i, client_app) in self.client_apps.iter_mut().enumerate() {
                client_app.finish();
                client_app.cleanup();

                let (client_channel, server_channel) =
                    lightyear::crossbeam::CrossbeamIo::new_pair();

                // Insert client endpoint
                client_app
                    .insert_resource(::client::network::CrossbeamClientEndpoint(client_channel));

                // CRITICAL: Set state to Lobby to trigger OnEnter(Lobby) on first update
                client_app.insert_state(ClientGameState::Lobby);

                // Store server channel for later
                server_channels.push((self.client_ids[i], server_channel));
            }

            // 3. Let clients update to establish connection
            for _ in 0..50 {
                for client_app in &mut self.client_apps {
                    client_app.update();
                }
                self.server_app.update();
            }

            // 4. NOW spawn server-side link entities for each client
            let server_world = self.server_app.world_mut();
            let server_entity = server_world
                .query_filtered::<Entity, With<Name>>()
                .iter(server_world)
                .find(|e| {
                    server_world
                        .get::<Name>(*e)
                        .map(|n| n.as_str() == "Server")
                        .unwrap_or(false)
                })
                .expect("Server entity not found");

            for (client_id, server_channel) in server_channels {
                self.server_app.world_mut().spawn((
                    lightyear::prelude::LinkOf {
                        server: server_entity,
                    },
                    lightyear::prelude::ReplicationSender::new(
                        std::time::Duration::from_millis(16),
                        lightyear::prelude::SendUpdatesMode::SinceLastAck,
                        true,
                    ),
                    lightyear::prelude::ReplicationReceiver::default(),
                    lightyear::prelude::Link::new(None),
                    lightyear::prelude::PeerAddr(SocketAddr::new(
                        std::net::IpAddr::V4(Ipv4Addr::LOCALHOST),
                        client_id as u16,
                    )),
                    lightyear::prelude::Connected,
                    server_channel,
                    Name::new(format!("Client_{}_Link", client_id)),
                    lightyear::prelude::RemoteId(lightyear::prelude::PeerId::Netcode(client_id)),
                ));

                // Update LobbyState
                let server_world = self.server_app.world_mut();
                for mut lobby_state in server_world
                    .query::<&mut shared::protocol::LobbyState>()
                    .iter_mut(server_world)
                {
                    if !lobby_state.players.contains(&client_id) {
                        lobby_state.players.push(client_id);
                    }
                }
            }

            // 5. Run MANY more updates to let replication stabilize (like setup_two_player_game's 800 cycles)
            for _ in 0..400 {
                self.server_app.update();
                for client_app in &mut self.client_apps {
                    client_app.update();
                }
            }
        }

        pub fn advance_time(&mut self, duration: Duration) {
            self.current_time += duration;
        }

        pub fn frame_step(&mut self) {
            self.advance_time(self.frame_duration);

            // Manually advance time to ensure physics/logic steps with non-zero delta
            if let Some(mut time) = self.server_app.world_mut().get_resource_mut::<Time>() {
                time.advance_by(self.frame_duration);
            }
            self.server_app.update();

            for client_app in &mut self.client_apps {
                if let Some(mut time) = client_app.world_mut().get_resource_mut::<Time>() {
                    time.advance_by(self.frame_duration);
                }
                client_app.update();
            }
        }

        pub fn loop_step(&mut self, updates: usize) {
            for _ in 0..updates {
                self.frame_step();
            }
        }

        #[allow(dead_code)]
        pub fn server_world(&self) -> &World {
            self.server_app.world()
        }

        #[allow(dead_code)]
        pub fn server_world_mut(&mut self) -> &mut World {
            self.server_app.world_mut()
        }

        #[allow(dead_code)]
        pub fn client_world(&self, client_index: usize) -> &World {
            self.client_apps[client_index].world()
        }

        pub fn client_world_mut(&mut self, client_index: usize) -> &mut World {
            self.client_apps[client_index].world_mut()
        }

        #[allow(dead_code)]
        pub fn simulate_client_input(
            &mut self,
            client_index: usize,
            mut input_fn: impl FnMut(&mut ActionState<PlayerAction>),
        ) {
            let world = self.client_world_mut(client_index);
            let mut query = world.query::<&mut ActionState<PlayerAction>>();
            for mut action_state in query.iter_mut(world) {
                input_fn(&mut action_state);
            }
        }
    }

    // ==========================================
    // RESTORED LEGACY HELPERS
    // ==========================================

    pub fn create_test_server() -> App {
        let mut server_app = create_server_app(true, shared::NetworkMode::Crossbeam);

        server_app.insert_resource(CollisionDiagnostics::default());
        server_app.insert_resource(SolverDiagnostics::default());
        server_app.insert_resource(SpatialQueryDiagnostics::default());

        server_app.add_plugins(bevy::log::LogPlugin {
            level: bevy::log::Level::TRACE,
            filter: "wgpu=error,bevy_render=info,bevy_ecs=info,avian3d=info,lightyear=info"
                .to_string(),
            ..default()
        });

        server_app
    }

    pub fn create_test_client(
        client_id: u64,
        auto_start: bool,
        auto_host: bool,
        auto_join: bool,
        headless: bool,
    ) -> App {
        let client_id = if client_id == 0 { 1 } else { client_id };
        let mut client_app = create_client_app(
            client_id,
            "../../assets".to_string(),
            headless,
            shared::NetworkMode::Crossbeam,
        );

        if auto_start {
            client_app.insert_resource(AutoStart(true));
        }
        if auto_host {
            client_app.insert_resource(AutoHost(true));
        }
        if auto_join {
            client_app.insert_resource(AutoJoin(true));
            client_app.insert_state(ClientGameState::Lobby);
        } else {
            client_app.insert_state(ClientGameState::LocalMenu);
        }

        client_app.insert_resource(CollisionDiagnostics::default());
        client_app.insert_resource(SolverDiagnostics::default());
        client_app.insert_resource(SpatialQueryDiagnostics::default());

        client_app
    }

    #[allow(dead_code)]
    pub fn run_app_updates(app: &mut App, cycles: usize) {
        for _ in 0..cycles {
            app.update();
            std::thread::sleep(std::time::Duration::from_millis(16));
        }
    }

    pub fn run_apps_updates(apps: &mut [&mut App], cycles: usize) {
        for _ in 0..cycles {
            for app in apps.iter_mut() {
                app.update();
            }
            std::thread::sleep(std::time::Duration::from_millis(16));
        }
    }

    #[allow(dead_code)]
    pub fn run_app_updates_with_delay(app: &mut App, cycles: usize, delay_ms: u64) {
        for _ in 0..cycles {
            app.update();
            if delay_ms > 0 {
                thread::sleep(Duration::from_millis(delay_ms));
            }
        }
    }

    #[allow(dead_code)]
    pub fn has_component<T: Component>(world: &World, entity: Entity) -> bool {
        world.get::<T>(entity).is_some()
    }

    #[allow(dead_code)]
    pub fn count_entities_with<T: Component>(world: &mut World) -> usize {
        let mut query = world.query::<&T>();
        query.iter(world).count()
    }

    #[allow(dead_code)]
    pub fn get_entities_with<T: Component>(world: &mut World) -> Vec<Entity> {
        let mut query = world.query::<(Entity, &T)>();
        query.iter(world).map(|(entity, _)| entity).collect()
    }

    pub fn simulate_player_movement(world: &mut World, player_entity: Entity, movement: Vec2) {
        if let Some(mut action_state) = world.get_mut::<ActionState<PlayerAction>>(player_entity) {
            action_state.set_axis_pair(&PlayerAction::Move, movement);
        }
    }

    #[allow(dead_code)]
    pub fn simulate_player_look(world: &mut World, player_entity: Entity, look_delta: Vec2) {
        if let Some(mut action_state) = world.get_mut::<ActionState<PlayerAction>>(player_entity) {
            action_state.set_axis_pair(&PlayerAction::Look, look_delta);
        }
    }

    #[allow(dead_code)]
    pub fn simulate_player_shoot(world: &mut World, player_entity: Entity, shooting: bool) {
        if let Some(mut action_state) = world.get_mut::<ActionState<PlayerAction>>(player_entity) {
            if shooting {
                action_state.press(&PlayerAction::Shoot);
            } else {
                action_state.release(&PlayerAction::Shoot);
            }
        }
    }

    pub fn get_entity_position(world: &World, entity: Entity) -> Option<Vec3> {
        world.get::<Position>(entity).map(|pos| pos.0)
    }

    pub fn assert_entity_moved(
        world: &World,
        entity: Entity,
        initial_position: Vec3,
        min_distance: f32,
    ) {
        let current_position =
            get_entity_position(world, entity).expect("Entity should have position component");

        let distance_moved = (current_position - initial_position).length();
        assert!(
            distance_moved >= min_distance,
            "Entity should have moved at least {} units, but only moved {} units. Initial: {:?}, Current: {:?}",
            min_distance,
            distance_moved,
            initial_position,
            current_position
        );
    }

    #[allow(dead_code)]
    pub fn assert_entity_stable(
        world: &World,
        entity: Entity,
        initial_position: Vec3,
        max_distance: f32,
    ) {
        let current_position =
            get_entity_position(world, entity).expect("Entity should have position component");

        let distance_moved = (current_position - initial_position).length();
        assert!(
            distance_moved <= max_distance,
            "Entity should not have moved more than {} units, but moved {} units",
            max_distance,
            distance_moved
        );
    }

    #[allow(dead_code)]
    pub fn wait_for_condition<F>(mut condition: F, max_attempts: usize, delay_ms: u64) -> bool
    where
        F: FnMut() -> bool,
    {
        for _ in 0..max_attempts {
            if condition() {
                return true;
            }
            if delay_ms > 0 {
                thread::sleep(Duration::from_millis(delay_ms));
            }
        }
        false
    }

    pub fn get_spawned_npcs(world: &mut World) -> Vec<Entity> {
        let mut query =
            world.query_filtered::<Entity, (With<CharacterMarker>, Without<PlayerId>)>();
        query.iter(world).collect()
    }

    pub fn get_spawned_players(world: &mut World) -> Vec<Entity> {
        let mut query = world.query::<(Entity, &PlayerId)>();
        query.iter(world).map(|(entity, _)| entity).collect()
    }

    #[allow(dead_code)]
    pub fn is_game_session_active(server_world: &mut World) -> bool {
        let playing = matches!(
            *server_world.resource::<State<ServerGameState>>().get(),
            ServerGameState::Playing
        );

        let has_players = !get_spawned_players(server_world).is_empty();

        playing && has_players
    }

    pub fn setup_two_player_game() -> (App, App, App) {
        let mut server_app = create_test_server();
        for _ in 0..20 {
            server_app.update();
            std::thread::sleep(std::time::Duration::from_millis(16));
        }

        // Create endpoints properly
        let mut client1 = create_test_client(1, true, false, true, true);
        let mut client2 = create_test_client(2, false, false, true, true);

        // Wire up Crossbeam channels
        use ::client::network::CrossbeamClientEndpoint;
        use bevy::prelude::Name;
        use lightyear::crossbeam::CrossbeamIo;
        use lightyear::prelude::{
            Connected, Link, LinkOf, PeerAddr, PeerId, RemoteId, ReplicationReceiver,
            ReplicationSender,
        };
        use shared::protocol::LobbyState;

        // Client 1 Connection
        let (c1, s1) = CrossbeamIo::new_pair();
        client1.insert_resource(CrossbeamClientEndpoint(c1));

        // Client 2 Connection
        let (c2, s2) = CrossbeamIo::new_pair();
        client2.insert_resource(CrossbeamClientEndpoint(c2));

        // Get Server Entity - be more robust with diagnostics
        let mut server_entity = None;
        for i in 0..20 {
            server_app.update();

            // Search by Name
            {
                let mut name_query = server_app.world_mut().query::<(Entity, &Name)>();
                server_entity = name_query
                    .iter(server_app.world())
                    .find(|(_, name)| name.as_str() == "Server")
                    .map(|(e, _)| e);
            }

            if server_entity.is_some() {
                break;
            }

            if i % 5 == 0 {
                println!(
                    "🔍 setup_two_player_game: Attempt {}/20. Total entities: {}",
                    i,
                    server_app.world().entities().len()
                );
                // Print some named entities to see what's there
                let mut name_query = server_app.world_mut().query::<(Entity, &Name)>();
                let named_list: Vec<_> = name_query.iter(server_app.world()).take(10).collect();
                println!("   - Samples of named entities: {:?}", named_list);
            }

            std::thread::sleep(std::time::Duration::from_millis(16));
        }

        let server_entity = server_entity.unwrap_or_else(|| {
            panic!(
                "Server entity not found after 20 updates! Total entities: {}",
                server_app.world().entities().len()
            )
        });

        // Spawn Server-side link for Client 1
        server_app.world_mut().spawn((
            LinkOf {
                server: server_entity,
            },
            ReplicationSender::new(
                std::time::Duration::from_millis(16),
                lightyear::prelude::SendUpdatesMode::SinceLastAck,
                true,
            ),
            ReplicationReceiver::default(),
            Link::new(None),
            PeerAddr(std::net::SocketAddr::new(
                std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST),
                1,
            )),
            Connected,
            s1,
            Name::new("Client_1_Link"),
            RemoteId(PeerId::Netcode(1)),
        ));

        // Spawn Server-side link for Client 2
        server_app.world_mut().spawn((
            LinkOf {
                server: server_entity,
            },
            ReplicationSender::new(
                std::time::Duration::from_millis(16),
                lightyear::prelude::SendUpdatesMode::SinceLastAck,
                true,
            ),
            ReplicationReceiver::default(),
            Link::new(None),
            PeerAddr(std::net::SocketAddr::new(
                std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST),
                2,
            )),
            Connected,
            s2,
            Name::new("Client_2_Link"),
            RemoteId(PeerId::Netcode(2)),
        ));

        // Manually update LobbyState since handle_connected won't run (missing ClientOf)
        let mut lobby_query = server_app.world_mut().query::<&mut LobbyState>();
        if let Some(mut lobby_state) = lobby_query.iter_mut(server_app.world_mut()).next() {
            if !lobby_state.players.contains(&1) {
                lobby_state.players.push(1);
            }
            if !lobby_state.players.contains(&2) {
                lobby_state.players.push(2);
            }
        }

        for _ in 0..300 {
            server_app.update();
            client1.update();
            client2.update();
            std::thread::sleep(std::time::Duration::from_millis(16));
        }

        for _ in 0..500 {
            server_app.update();
            client1.update();
            client2.update();
            std::thread::sleep(std::time::Duration::from_millis(16));
        }

        (server_app, client1, client2)
    }
}
