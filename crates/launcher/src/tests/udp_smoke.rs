use super::*;
use lightyear_tests::stepper::{ClientServerStepper, StepperConfig};

fn wait_until_two_clients_connected_and_lobby_ready(
    server_app: &mut App,
    client_app1: &mut App,
    client_app2: &mut App,
    max_ticks: usize,
) -> bool {
    for _ in 0..max_ticks {
        super::update_all(server_app, client_app1, client_app2);

        let server_state = server_app
            .world()
            .resource::<bevy::prelude::State<server::ServerGameState>>()
            .get()
            .clone();

        let client1_state = client_app1
            .world()
            .resource::<bevy::prelude::State<client::ClientGameState>>()
            .get()
            .clone();

        let client2_state = client_app2
            .world()
            .resource::<bevy::prelude::State<client::ClientGameState>>()
            .get()
            .clone();

        if super::server_lobby_player_count(server_app) >= 2
            && server_state == server::ServerGameState::Lobby
            && client1_state == client::ClientGameState::Lobby
            && client2_state == client::ClientGameState::Lobby
        {
            return true;
        }
    }
    false
}

#[test]
fn test_udp_lobby_replication_smoke() {
    let mut server_app = create_test_server_app_with_mode(true, NetworkMode::Udp);
    let mut client_app1 = create_test_client_app_with_mode(1, true, NetworkMode::Udp);
    let mut client_app2 = create_test_client_app_with_mode(2, true, NetworkMode::Udp);

    assert!(
        wait_until_two_clients_connected_and_lobby_ready(
            &mut server_app,
            &mut client_app1,
            &mut client_app2,
            300,
        ),
        "UDP smoke should reach connected lobby-ready state for both clients"
    );
}

#[test]
fn test_udp_end_to_end_playing_smoke() {
    let mut config = StepperConfig::with_netcode_clients(2);
    config.init = true;
    let mut stepper = ClientServerStepper::from_config(config);
    stepper.frame_step(12);

    assert_eq!(
        stepper.client_apps.len(),
        2,
        "Crossbeam deterministic smoke should create two client apps"
    );
    assert_eq!(
        stepper.client_entities.len(),
        2,
        "Crossbeam deterministic smoke should create two client entities"
    );
    assert_eq!(
        stepper.client_of_entities.len(),
        2,
        "Crossbeam deterministic smoke should create two server-side client links"
    );
}

fn create_test_host_app(gym_mode: bool) -> App {
    let mut app = super::create_test_server_app_with_mode_unfinished(gym_mode, NetworkMode::Udp);

    // Use a different UDP port to avoid conflicts with other UDP tests
    let host_addr = std::net::SocketAddr::new(
        std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)),
        8081,
    );
    app.insert_resource(shared::ServerBindAddr(host_addr));
    app.insert_resource(client::network::ServerAddr(host_addr));

    // Add client plugins to the same app!
    app.insert_resource(client::Headless(true));
    app.init_asset::<bevy::prelude::Mesh>();
    app.init_asset::<bevy::prelude::StandardMaterial>();
    app.add_plugins(lightyear::prelude::client::ClientPlugins {
        tick_duration: std::time::Duration::from_secs_f64(1.0 / shared::FIXED_TIMESTEP_HZ),
    });

    app.insert_resource(client::LocalPlayerId(1));
    app.add_plugins(client::network::ClientNetworkPlugin);
    app.add_plugins(client::inputs::ClientInputPlugin);
    // The host test app uses MinimalPlugins (no DefaultPlugins), so the
    // client input systems that require Bevy input/window resources panic.
    // Add the minimal input and window plugins so cursor-capture and focus
    // systems can run when the host client enters Playing state.
    app.add_plugins(bevy::input::InputPlugin);
    app.add_plugins(bevy::window::WindowPlugin {
        primary_window: Some(bevy::prelude::Window::default()),
        ..Default::default()
    });
    app.add_plugins(client::camera::ClientCameraPlugin);
    app.add_plugins(client::entities::ClientEntitiesPlugin);
    app.add_plugins(client::lobby::ClientLobbyPlugin);
    app.add_plugins(client::game::ClientGameCyclePlugin);

    app.init_state::<client::ClientGameState>();
    app.insert_state(client::ClientGameState::Lobby);

    super::finish_if_needed(&mut app);
    app
}

#[test]
fn test_udp_host_and_client_smoke() {
    // Client 1 (Host) runs the server and client inside the exact same App!
    let mut host_app = create_test_host_app(true);

    // Client 2 is a separate machine connecting via UDP
    let mut external_client_app =
        super::create_test_client_app_with_mode(2, true, NetworkMode::Udp);
    external_client_app.insert_resource(client::network::ServerAddr(std::net::SocketAddr::new(
        std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)),
        8081,
    )));

    let mut success = false;
    for _ in 0..300 {
        super::update_single_app(&mut host_app, std::time::Duration::from_millis(16));
        super::update_single_app(
            &mut external_client_app,
            std::time::Duration::from_millis(16),
        );

        let server_state = host_app
            .world()
            .resource::<bevy::prelude::State<server::ServerGameState>>()
            .get()
            .clone();

        let client1_state = host_app
            .world()
            .resource::<bevy::prelude::State<client::ClientGameState>>()
            .get()
            .clone();

        let client2_state = external_client_app
            .world()
            .resource::<bevy::prelude::State<client::ClientGameState>>()
            .get()
            .clone();

        if super::server_lobby_player_count(&mut host_app) >= 2
            && server_state == server::ServerGameState::Lobby
            && client1_state == client::ClientGameState::Lobby
            && client2_state == client::ClientGameState::Lobby
        {
            success = true;
            break;
        }
    }

    assert!(
        success,
        "UDP host smoke should reach connected lobby-ready state for both the local host client and the external UDP client"
    );
}

#[test]
fn test_udp_late_joiner_reaches_playing() {
    // Host (server+client in one App) starts the game.
    // A second (external) client connects _after_ the game has already entered
    // the Loading / Playing phase.  The late joiner must receive a
    // StartLoadingGameEvent and transition away from Lobby.
    let mut host_app = create_test_host_app(true);

    // Override the UDP port to avoid conflicts with other UDP tests running
    // in parallel.
    let late_joiner_host_addr = std::net::SocketAddr::new(
        std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)),
        8082,
    );
    host_app.insert_resource(shared::ServerBindAddr(late_joiner_host_addr));
    host_app.insert_resource(client::network::ServerAddr(late_joiner_host_addr));

    let mut external_client_app =
        super::create_test_client_app_with_mode(2, true, NetworkMode::Udp);
    external_client_app.insert_resource(client::network::ServerAddr(std::net::SocketAddr::new(
        std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)),
        8082,
    )));

    // Step 1: wait until the host is connected and in the lobby, then auto-start.
    let mut auto_started = false;
    for _ in 0..200 {
        super::update_single_app(&mut host_app, std::time::Duration::from_millis(16));

        let server_state = host_app
            .world()
            .resource::<bevy::prelude::State<server::ServerGameState>>()
            .get()
            .clone();

        if super::server_lobby_player_count(&mut host_app) >= 1
            && server_state == server::ServerGameState::Lobby
        {
            host_app.insert_resource(server::lobby::AutoStartOnLobbyReady(true));
            auto_started = true;
            break;
        }
    }
    assert!(auto_started, "Host should have connected and auto-started");

    // Step 2: advance until the server enters Playing.
    let mut server_playing = false;
    for _ in 0..300 {
        super::update_single_app(&mut host_app, std::time::Duration::from_millis(16));

        let server_state = host_app
            .world()
            .resource::<bevy::prelude::State<server::ServerGameState>>()
            .get()
            .clone();

        if server_state == server::ServerGameState::Playing {
            server_playing = true;
            break;
        }
    }
    assert!(
        server_playing,
        "Server should reach Playing after auto-start"
    );

    // Step 3: connect the late-joining external client.
    let mut late_joiner_left_lobby = false;
    for _ in 0..600 {
        super::update_single_app(&mut host_app, std::time::Duration::from_millis(16));
        super::update_single_app(
            &mut external_client_app,
            std::time::Duration::from_millis(16),
        );

        let external_state = external_client_app
            .world()
            .resource::<bevy::prelude::State<client::ClientGameState>>()
            .get()
            .clone();

        // The late joiner must leave the Lobby state (entering Loading or Playing).
        if external_state != client::ClientGameState::Lobby {
            late_joiner_left_lobby = true;
            break;
        }
    }

    assert!(
        late_joiner_left_lobby,
        "Late-joining UDP client should leave Lobby state after game start \
         (regression: handle_connected must send StartLoadingGameEvent during Loading/Playing)"
    );
}

#[test]
fn test_udp_both_in_lobby_host_clicks_play_both_reach_playing() {
    // Normal flow: both host and external client are in the lobby. The host
    // client sends HostStartGameEvent (simulating "host clicks Play"). Both
    // clients must receive the StartLoadingGameEvent + LevelSeed and reach Playing.
    let mut host_app = create_test_host_app(true);

    let host_addr = std::net::SocketAddr::new(
        std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)),
        8084,
    );
    host_app.insert_resource(shared::ServerBindAddr(host_addr));
    host_app.insert_resource(client::network::ServerAddr(host_addr));

    // Add a PostUpdate debug system to trace the message pipeline
    host_app.add_systems(bevy::prelude::PostUpdate, |q: bevy::ecs::system::Query<&lightyear::prelude::Link, bevy::ecs::query::With<lightyear::prelude::Client>>| {
        use shared::debug::debug_println;
        for link in q.iter() {
            if link.send.len() > 0 {
                debug_println(format_args!("DEBUG POSTUPDATE: Link send has {} payloads", link.send.len()));
            }
        }
    });

    // Add a PostUpdate debug system to check Transport send_channel
    host_app.add_systems(bevy::prelude::PostUpdate, |q: bevy::ecs::system::Query<&lightyear::prelude::Transport, (bevy::prelude::With<lightyear::prelude::Client>, bevy::prelude::With<lightyear::prelude::Connected>)>| {
        use shared::debug::debug_println;
        for transport in q.iter() {
            let send_count = transport.send_channel.len();
            let recv_count = transport.recv_channel.len();
            if send_count > 0 || recv_count > 0 {
                debug_println(format_args!("DEBUG POSTUPDATE: Transport send_channel={}, recv_channel={}", send_count, recv_count));
            }
        }
    });

    let mut external_client_app =
        super::create_test_client_app_with_mode(2, true, NetworkMode::Udp);
    external_client_app.insert_resource(client::network::ServerAddr(std::net::SocketAddr::new(
        std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)),
        8084,
    )));

    // Step 1: wait until both clients are in Lobby and the server has 2 players.
    let mut both_in_lobby = false;
    for _ in 0..300 {
        super::update_single_app(&mut host_app, std::time::Duration::from_millis(16));
        super::update_single_app(
            &mut external_client_app,
            std::time::Duration::from_millis(16),
        );

        let server_state = host_app
            .world()
            .resource::<bevy::prelude::State<server::ServerGameState>>()
            .get()
            .clone();

        let client1_state = host_app
            .world()
            .resource::<bevy::prelude::State<client::ClientGameState>>()
            .get()
            .clone();

        let client2_state = external_client_app
            .world()
            .resource::<bevy::prelude::State<client::ClientGameState>>()
            .get()
            .clone();

        if super::server_lobby_player_count(&mut host_app) >= 2
            && server_state == server::ServerGameState::Lobby
            && client1_state == client::ClientGameState::Lobby
            && client2_state == client::ClientGameState::Lobby
        {
            both_in_lobby = true;
            break;
        }
    }
    assert!(both_in_lobby, "Both clients should reach Lobby state");

    // Step 2: enable host-side AutoStart so the host client sends HostStartGameEvent.
    // This tests the HostStartGameEvent path (host clicks Play), not the server-side
    // AutoStartOnLobbyReady path.
    host_app.insert_resource(client::lobby::AutoStart(true));

    // Step 3: wait for both to reach Playing.
    let mut host_playing = false;
    let mut external_playing = false;
    for _ in 0..600 {
        super::update_single_app(&mut host_app, std::time::Duration::from_millis(16));
        super::update_single_app(
            &mut external_client_app,
            std::time::Duration::from_millis(16),
        );

        let client1_state = host_app
            .world()
            .resource::<bevy::prelude::State<client::ClientGameState>>()
            .get()
            .clone();

        let client2_state = external_client_app
            .world()
            .resource::<bevy::prelude::State<client::ClientGameState>>()
            .get()
            .clone();

        if client1_state == client::ClientGameState::Playing {
            host_playing = true;
        }
        if client2_state == client::ClientGameState::Playing {
            external_playing = true;
        }

        if host_playing && external_playing {
            break;
        }
    }

    // Diagnostics on failure
    let server_state = host_app
        .world()
        .resource::<bevy::prelude::State<server::ServerGameState>>()
        .get()
        .clone();

    let external_state = external_client_app
        .world()
        .resource::<bevy::prelude::State<client::ClientGameState>>()
        .get()
        .clone();

    let external_msg_receiver_count = {
        use lightyear::prelude::MessageReceiver;
        use shared::protocol::StartLoadingGameEvent;
        let world = external_client_app.world_mut();
        let mut q = world.query::<&MessageReceiver<StartLoadingGameEvent>>();
        q.iter(world).count()
    };

    let external_buffered_messages = {
        use lightyear::prelude::MessageReceiver;
        use shared::protocol::StartLoadingGameEvent;
        let world = external_client_app.world_mut();
        let mut q = world.query::<&mut MessageReceiver<StartLoadingGameEvent>>();
        let mut total = 0usize;
        for receiver in q.iter_mut(world) {
            total += receiver.num_messages();
        }
        total
    };

    let external_has_level_seed = {
        use shared::protocol::LevelSeed;
        let world = external_client_app.world_mut();
        let mut q = world.query::<&LevelSeed>();
        q.iter(world).count()
    };

    // Server-side diagnostics: check if server has MessageReceiver on ClientOf entities
    let server_host_start_receivers = {
        use lightyear::prelude::{Connected, MessageReceiver};
        use shared::protocol::HostStartGameEvent;
        let world = host_app.world_mut();
        let mut q = world.query::<&MessageReceiver<HostStartGameEvent>>();
        let mut count = 0usize;
        let mut buffered = 0usize;
        for receiver in q.iter(world) {
            count += 1;
            buffered += receiver.num_messages();
        }
        (count, buffered)
    };

    // Check server has LobbyState entity
    let server_lobby_count = {
        use shared::protocol::LobbyState;
        let world = host_app.world_mut();
        let mut q = world.query::<&LobbyState>();
        q.iter(world).count()
    };

    // Check server has Connected ClientOf entities
    let server_clientof_count = {
        use lightyear::connection::client_of::ClientOf;
        use lightyear::prelude::Connected;
        let world = host_app.world_mut();
        let mut q = world.query::<&ClientOf>();
        q.iter(world).count()
    };

    // Diagnostics: check host client side state
    let host_client_info = {
        use bevy::prelude::Entity;
        use lightyear::prelude::{Connected, Linked, Transport};
        use lightyear::prelude::MessageSender as LSender;
        use shared::protocol::HostStartGameEvent;
        use lightyear::connection::client::Client;
        let world = host_app.world_mut();
        let mut q = world.query::<(
            Entity,
            &Client,
            Option<&Connected>,
            Option<&Linked>,
            Option<&Transport>,
            Option<&LSender<HostStartGameEvent>>,
        )>();
        let mut found = false;
        let mut has_connected = false;
        let mut has_linked = false;
        let mut has_transport = false;
        let mut has_sender = false;
        let mut sender_entities = 0;
        for (entity, _client, connected, linked, transport, sender) in q.iter(world) {
            found = true;
            has_connected = connected.is_some();
            has_linked = linked.is_some();
            has_transport = transport.is_some();
            has_sender = sender.is_some();
            sender_entities += 1;
        }
        format!(
            "client_found={}, connected={}, linked={}, transport={}, has_sender={}, sender_entities={}",
            found, has_connected, has_linked, has_transport, has_sender, sender_entities
        )
    };

    // Diagnostics: check server-side ClientOf transport channels
    let server_transport_info = {
        use lightyear::prelude::{Connected, Transport};
        use lightyear::connection::client_of::ClientOf;
        let world = host_app.world_mut();
        let mut q = world.query::<(&ClientOf, Option<&Transport>, Option<&Connected>)>();
        let mut count = 0usize;
        let mut with_transport = 0usize;
        let mut with_connected = 0usize;
        for (of, transport, connected) in q.iter(world) {
            count += 1;
            if transport.is_some() { with_transport += 1; }
            if connected.is_some() { with_connected += 1; }
        }
        format!("clientof_count={}, with_transport={}, with_connected={}", count, with_transport, with_connected)
    };

    // Check LobbyControlChannel on client Transport
    let client_channel_info = {
        use lightyear::prelude::{Client, Connected, Transport};
        use lightyear::transport::channel::ChannelKind;
        use shared::protocol::LobbyControlChannel;
        use bevy::ecs::query::With;
        let world = host_app.world_mut();
        let mut q = world.query_filtered::<&Transport, (With<Client>, With<Connected>)>();
        let mut sender_count = 0usize;
        let mut receiver_count = 0usize;
        for transport in q.iter(world) {
            if transport.channel_sends().any(|(kind, _)| kind.0 == ChannelKind::of::<LobbyControlChannel>().0) {
                sender_count += 1;
            }
            if transport.channel_receives().any(|r| r.channel_kind().0 == ChannelKind::of::<LobbyControlChannel>().0) {
                receiver_count += 1;
            }
        }
        format!("clients_with_sender={}, clients_with_receiver={}", sender_count, receiver_count)
    };

    // Check LobbyControlChannel on server ClientOf Transport
    let server_channel_info = {
        use lightyear::prelude::{Connected, Transport};
        use lightyear::connection::client_of::ClientOf;
        use lightyear::transport::channel::ChannelKind;
        use shared::protocol::LobbyControlChannel;
        use bevy::ecs::query::With;
        let world = host_app.world_mut();
        let mut q = world.query_filtered::<&Transport, (With<ClientOf>, With<Connected>)>();
        let mut sender_count = 0usize;
        let mut receiver_count = 0usize;
        for transport in q.iter(world) {
            if transport.channel_sends().any(|(kind, _)| kind.0 == ChannelKind::of::<LobbyControlChannel>().0) {
                sender_count += 1;
            }
            if transport.channel_receives().any(|r| r.channel_kind().0 == ChannelKind::of::<LobbyControlChannel>().0) {
                receiver_count += 1;
            }
        }
        format!("server_senders={}, server_receivers={}", sender_count, receiver_count)
    };

    assert!(
        external_playing,
        "External UDP client should reach Playing when host clicks Play.\n\
         server_state={:?}, external_state={:?}, \
         external_msg_receiver_count={}, external_buffered_messages={}, \
         external_has_level_seed={}, \
         server_host_start_receivers={}/{}, server_lobby_count={}, \
         server_clientof_count={}, \
         host_client_info: {}, \
         server_transport_info: {}, \
         client_channel_info: {}, \
         server_channel_info: {}",
        server_state,
        external_state,
        external_msg_receiver_count,
        external_buffered_messages,
        external_has_level_seed,
        server_host_start_receivers.1, server_host_start_receivers.0,
        server_lobby_count,
        server_clientof_count,
        host_client_info,
        server_transport_info,
        client_channel_info,
        server_channel_info,
    );
}
