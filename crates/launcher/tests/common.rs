#[cfg(test)]
pub mod test {
    use avian3d::collision::CollisionDiagnostics;
    use avian3d::dynamics::solver::SolverDiagnostics;
    use avian3d::prelude::Position;
    use avian3d::spatial_query::SpatialQueryDiagnostics;
    use bevy::prelude::*;
    use client::lobby::AutoStart;
    use client::{ClientGameState, create_client_app};
    use launcher::{AutoHost, AutoJoin};
    use leafwing_input_manager::prelude::ActionState;
    use server::{ServerGameState, create_server_app};
    use shared::{
        input::PlayerAction,
        protocol::{CharacterMarker, PlayerId},
    };
    use std::{thread, time::Duration};

    pub fn create_test_server() -> App {
        let mut server_app = create_server_app(true, shared::NetworkMode::Crossbeam);

        server_app.insert_resource(CollisionDiagnostics::default());
        server_app.insert_resource(SolverDiagnostics::default());
        server_app.insert_resource(SpatialQueryDiagnostics::default());

        // Enable LogPlugin for debugging
        server_app.add_plugins(bevy::log::LogPlugin {
            level: bevy::log::Level::TRACE,
            filter: "wgpu=error,bevy_render=info,bevy_ecs=info,avian3d=info,lightyear=info".to_string(),
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
        let mut client_app = create_client_app(client_id, "../../assets".to_string(), headless, shared::NetworkMode::Crossbeam);

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

    pub fn run_app_updates_with_delay(app: &mut App, cycles: usize, delay_ms: u64) {
        for _ in 0..cycles {
            app.update();
            if delay_ms > 0 {
                thread::sleep(Duration::from_millis(delay_ms));
            }
        }
    }

    pub fn has_component<T: Component>(world: &World, entity: Entity) -> bool {
        world.get::<T>(entity).is_some()
    }

    pub fn count_entities_with<T: Component>(world: &mut World) -> usize {
        let mut query = world.query::<&T>();
        query.iter(world).count()
    }

    pub fn get_entities_with<T: Component>(world: &mut World) -> Vec<Entity> {
        let mut query = world.query::<(Entity, &T)>();
        query.iter(world).map(|(entity, _)| entity).collect()
    }

    pub fn simulate_player_movement(world: &mut World, player_entity: Entity, movement: Vec2) {
        if let Some(mut action_state) = world.get_mut::<ActionState<PlayerAction>>(player_entity) {
            action_state.set_axis_pair(&PlayerAction::Move, movement);
        }
    }

    pub fn simulate_player_look(world: &mut World, player_entity: Entity, look_delta: Vec2) {
        if let Some(mut action_state) = world.get_mut::<ActionState<PlayerAction>>(player_entity) {
            action_state.set_axis_pair(&PlayerAction::Look, look_delta);
        }
    }

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
        use lightyear::crossbeam::CrossbeamIo;
        use lightyear::prelude::{Link, LinkOf, PeerAddr, Connected, RemoteId, PeerId, ReplicationSender, ReplicationReceiver};
        use bevy::prelude::Name;
        use client::network::CrossbeamClientEndpoint;
        use shared::protocol::LobbyState;
        
        // Client 1 Connection
        let (c1, s1) = CrossbeamIo::new_pair();
        client1.insert_resource(CrossbeamClientEndpoint(c1));
        
        // Client 2 Connection
        let (c2, s2) = CrossbeamIo::new_pair();
        client2.insert_resource(CrossbeamClientEndpoint(c2));

        // Get Server Entity
        server_app.update(); 
        
        let mut server_query = server_app.world_mut().query_filtered::<Entity, With<Name>>();
        let server_entity = server_query.iter(server_app.world())
            .find(|e| server_app.world().get::<Name>(*e).map(|n| n.as_str() == "Server").unwrap_or(false));
            
        let server_entity = server_entity.expect("Server entity not found! Did PreStartup run?");
        
        // Spawn Server-side link for Client 1
        server_app.world_mut().spawn((
            LinkOf { server: server_entity },
            ReplicationSender::new(std::time::Duration::from_millis(16), lightyear::prelude::SendUpdatesMode::SinceLastAck, true),
            ReplicationReceiver::default(),
            Link::new(None),
            PeerAddr(std::net::SocketAddr::new(std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST), 1)), 
            Connected, 
            s1,
            Name::new("Client_1_Link"),
            RemoteId(PeerId::Netcode(1)),
        ));

        // Spawn Server-side link for Client 2
        server_app.world_mut().spawn((
            LinkOf { server: server_entity },
            ReplicationSender::new(std::time::Duration::from_millis(16), lightyear::prelude::SendUpdatesMode::SinceLastAck, true),
            ReplicationReceiver::default(),
            Link::new(None),
            PeerAddr(std::net::SocketAddr::new(std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST), 2)),
            Connected,
            s2,
            Name::new("Client_2_Link"),
            RemoteId(PeerId::Netcode(2)),
        ));
        
        // Manually update LobbyState since handle_connected won't run (missing ClientOf)
        let mut lobby_query = server_app.world_mut().query::<&mut LobbyState>();
        if let Ok(mut lobby_state) = lobby_query.get_single_mut(server_app.world_mut()) {
            println!("DEBUG: Manually adding players to LobbyState");
            if !lobby_state.players.contains(&1) {
                lobby_state.players.push(1);
            }
            if !lobby_state.players.contains(&2) {
                lobby_state.players.push(2);
            }
        } else {
             println!("WARN: LobbyState not found!");
        }

        // Run many cycles to ensure connection and LobbyState replication
        for _ in 0..300 {
            server_app.update();
            client1.update();
            client2.update();
            std::thread::sleep(std::time::Duration::from_millis(16));
        }

        // AutoStart should trigger game start automatically
        for _ in 0..500 {
            server_app.update();
            client1.update();
            client2.update();
            std::thread::sleep(std::time::Duration::from_millis(16));
        }

        let _server_state = server_app.world().resource::<State<ServerGameState>>().get();
        let _client1_state = client1.world().resource::<State<ClientGameState>>().get();
        let _client2_state = client2.world().resource::<State<ClientGameState>>().get();

        (server_app, client1, client2)
    }
}
