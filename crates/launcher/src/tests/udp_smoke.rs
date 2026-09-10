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
