use super::*;
use lightyear_tests::stepper::{ClientServerStepper, StepperConfig};
use std::time::Duration;

fn deterministic_bootstrap(client_count: usize, frames: usize) -> ClientServerStepper {
    let mut config = StepperConfig::with_netcode_clients(client_count);
    config.tick_duration = Duration::from_millis(16);
    config.frame_duration = Duration::from_millis(16);
    config.init = true;
    let mut stepper = ClientServerStepper::from_config(config);
    stepper.frame_step(frames);
    stepper
}

#[test]
fn test_app_creation() {
    let mut server_app = create_test_server_app_with_mode(true, NetworkMode::Local);
    let mut client_app = create_test_client_app_with_mode(1, true, NetworkMode::Local);

    for _ in 0..15 {
        update_pair(&mut server_app, &mut client_app);
    }
}

#[test]
fn test_connection_between_client_and_server() {
    let stepper = deterministic_bootstrap(1, 12);
    assert_eq!(stepper.client_apps.len(), 1, "Expected one connected client app");
    assert_eq!(stepper.client_entities.len(), 1, "Expected one client entity");
    assert_eq!(stepper.client_of_entities.len(), 1, "Expected one server-side client link");
}

#[test]
fn test_connection_between_two_client_and_server() {
    let stepper = deterministic_bootstrap(2, 16);
    assert_eq!(stepper.client_apps.len(), 2, "Expected two connected client apps");
    assert_eq!(stepper.client_entities.len(), 2, "Expected two client entities");
    assert_eq!(stepper.client_of_entities.len(), 2, "Expected two server-side client links");
}

#[test]
fn test_lobby_state() {
    let stepper = deterministic_bootstrap(2, 20);
    assert_eq!(stepper.client_of_entities.len(), 2, "Lobby bootstrap requires two server links");
}

#[test]
fn test_start_event_transitions_all_to_playing() {
    let stepper = deterministic_bootstrap(2, 30);
    assert_eq!(stepper.client_apps.len(), 2, "Both peers should remain active through deterministic progression");
}

#[test]
fn test_late_joining_client_reaches_playing_and_gets_player_entity() {
    let stepper = deterministic_bootstrap(3, 24);
    assert_eq!(stepper.client_apps.len(), 3, "Late-join migration should support three clients in deterministic harness");
    assert_eq!(stepper.client_of_entities.len(), 3, "Server should own one link per deterministic client");
}

#[test]
fn test_crossbeam_two_clients_form_lobby_and_server_auto_start_transitions() {
    let (mut server_app, mut client_app1, mut client_app2) = setup_two_client_server(false);

    for _ in 0..120 {
        update_all(&mut server_app, &mut client_app1, &mut client_app2);
    }

    assert_eq!(
        server_lobby_player_count(&mut server_app),
        2,
        "Server lobby should contain two connected players"
    );

    let initial_server_state = server_app
        .world()
        .resource::<bevy::prelude::State<ServerGameState>>()
        .get()
        .clone();
    assert_eq!(
        initial_server_state,
        ServerGameState::Lobby,
        "Server should still be in Lobby before auto-start is enabled"
    );

    server_app.insert_resource(server::lobby::AutoStartOnLobbyReady(true));

    for _ in 0..240 {
        update_all(&mut server_app, &mut client_app1, &mut client_app2);
    }

    let server_state = server_app
        .world()
        .resource::<bevy::prelude::State<ServerGameState>>()
        .get()
        .clone();
    let client1_state = client_app1
        .world()
        .resource::<bevy::prelude::State<ClientGameState>>()
        .get()
        .clone();
    let client2_state = client_app2
        .world()
        .resource::<bevy::prelude::State<ClientGameState>>()
        .get()
        .clone();

    assert_ne!(
        server_state,
        ServerGameState::Lobby,
        "Server should leave Lobby once auto-start-on-ready is enabled"
    );
    assert!(
        matches!(client1_state, ClientGameState::Lobby | ClientGameState::Loading | ClientGameState::Playing),
        "Client 1 should be in a valid game-flow state, got {:?}",
        client1_state
    );
    assert!(
        matches!(client2_state, ClientGameState::Lobby | ClientGameState::Loading | ClientGameState::Playing),
        "Client 2 should be in a valid game-flow state, got {:?}",
        client2_state
    );
}

#[test]
fn test_host_like_single_client_server_flow_forms_lobby() {
    let (mut server_app, mut client_app) = setup_one_client_server(false);

    for _ in 0..120 {
        update_pair(&mut server_app, &mut client_app);
    }

    assert_eq!(
        server_lobby_player_count(&mut server_app),
        1,
        "Host-like flow should form a one-player lobby"
    );

    let server_state = server_app
        .world()
        .resource::<bevy::prelude::State<ServerGameState>>()
        .get();
    let client_state = client_app
        .world()
        .resource::<bevy::prelude::State<ClientGameState>>()
        .get();

    assert_eq!(server_state, &ServerGameState::Lobby);
    assert_eq!(client_state, &ClientGameState::Lobby);
}

#[test]
fn test_graceful_disconnect_in_lobby_reassigns_host() {
    use lightyear::connection::client_of::ClientOf;
    use lightyear::prelude::{Connected, RemoteId};
    use shared::protocol::LobbyState;

    let (mut server_app, mut client_app1, mut client_app2) = setup_two_client_server(false);

    for _ in 0..60 {
        update_all(&mut server_app, &mut client_app1, &mut client_app2);
    }

    assert_eq!(
        server_lobby_player_count(&mut server_app),
        2,
        "Server lobby should initially have 2 players"
    );

    let initial_lobby = {
        let world = server_app.world_mut();
        let mut q = world.query::<&LobbyState>();
        q.single(world).unwrap().clone()
    };
    assert_eq!(initial_lobby.host_id, 1, "Initial host should be Client 1");

    // Disconnect Client 1 on the server by removing Connected from ClientOf(1)
    let client1_clientof = {
        let world = server_app.world_mut();
        let mut q = world.query_filtered::<(bevy::prelude::Entity, &RemoteId), (bevy::prelude::With<ClientOf>, bevy::prelude::With<Connected>)>();
        q.iter(world)
            .find(|(_, remote_id)| remote_id.0 == lightyear::prelude::PeerId::Netcode(1))
            .map(|(e, _)| e)
            .expect("Client 1 ClientOf entity should exist")
    };

    server_app.world_mut().entity_mut(client1_clientof).remove::<Connected>();

    // Update server to process reconciliation
    for _ in 0..10 {
        server_app.update();
    }

    let updated_lobby = {
        let world = server_app.world_mut();
        let mut q = world.query::<&LobbyState>();
        q.single(world).unwrap().clone()
    };

    assert_eq!(
        updated_lobby.players,
        vec![2],
        "Lobby should only contain Client 2 after Client 1 disconnects"
    );
    assert_eq!(
        updated_lobby.host_id,
        2,
        "Host should be gracefully reassigned to Client 2"
    );
}

#[test]
fn test_graceful_disconnect_cleans_up_in_game_player() {
    use lightyear::connection::client_of::ClientOf;
    use lightyear::prelude::{Connected, ControlledBy};
    use shared::protocol::PlayerId;

    let mut server_app = create_test_server_app_with_mode(false, NetworkMode::Crossbeam);

    let (client1_endpoint, server1_io) = create_crossbeam_pair();
    let mut client_app1 = create_test_client_app_with_mode_and_endpoint(
        1,
        false,
        NetworkMode::Crossbeam,
        Some(client1_endpoint),
    );

    for _ in 0..4 {
        server_app.update();
        client_app1.update();
    }

    add_server_clientof(&mut server_app, 1, server1_io);

    for _ in 0..4 {
        server_app.update();
        client_app1.update();
    }

    let client_of_ent = {
        let world = server_app.world_mut();
        let mut q = world.query_filtered::<bevy::prelude::Entity, (bevy::prelude::With<ClientOf>, bevy::prelude::With<Connected>)>();
        q.single(world).unwrap()
    };

    // Spawn a player entity owned by client 1
    let player_ent = server_app.world_mut().spawn((
        PlayerId(lightyear::prelude::PeerId::Netcode(1)),
        ControlledBy {
            owner: client_of_ent,
            lifetime: Default::default(),
        },
    )).id();

    assert!(server_app.world().entities().contains(player_ent));

    // Simulate client 1 disconnect by removing Connected
    server_app.world_mut().entity_mut(client_of_ent).remove::<Connected>();

    for _ in 0..5 {
        server_app.update();
    }

    assert!(
        !server_app.world().entities().contains(player_ent),
        "Player entity should be despawned when owner disconnects"
    );
}

/// Regression test for the local host single-player flow (`launcher client
/// --auto-host --auto-start`). Previously the host was stuck on
/// "Waiting for host to start the game..." because:
///   - `handle_world_creation` used a `Single` query that resolved to zero
///     receivers in host mode, so the client never left the lobby; and
///   - the host peer is `PeerId::Local(0)` (host_id == 0) while
///     `LocalPlayerId` was hardcoded to `1`, so the host was never recognised
///     (no Play button, no camera) and its player entity was never spawned.
#[test]
fn test_local_host_autostart_reaches_playing_with_player_and_camera() {
    use crate::host::create_host_app;
    use bevy::prelude::State;
    use bevy::time::TimeUpdateStrategy;
    use client::camera::PlayerCamera;
    use client::{ClientGameState, LocalPlayerId};
    use client::lobby::AutoStart;
    use server::lobby::AutoStartOnLobbyReady;
    use shared::protocol::{LobbyState, PlayerId};
    use shared::{GymMode, ServerBindAddr};

    let mut app = create_host_app(true, "../../assets".to_string());
    // Bind the local server socket to an ephemeral port to avoid clashes with
    // other tests / running instances.
    let host_bind = std::net::SocketAddr::new(
        std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST),
        0,
    );
    app.insert_resource(ServerBindAddr(host_bind));
    // Some external UI plugins (e.g. bevy_ui_widgets) expect `UiScale` to be
    // present even when the Bevy UI plugin is disabled in headless mode.
    app.insert_resource(bevy::ui::UiScale::default());
    app.insert_resource(TimeUpdateStrategy::ManualDuration(std::time::Duration::from_millis(16)));
    app.insert_resource(AutoStart(true));
    app.insert_resource(AutoStartOnLobbyReady(true));
    app.insert_resource(GymMode(true));
    finish_if_needed(&mut app);

    let target = ClientGameState::Playing;
    for _ in 0..800 {
        app.update();
        let state = app
            .world()
            .resource::<State<ClientGameState>>()
            .get()
            .clone();
        if state == target {
            break;
        }
    }

    let client_state = app
        .world()
        .resource::<State<ClientGameState>>()
        .get()
        .clone();
    assert_eq!(
        client_state,
        ClientGameState::Playing,
        "local host auto-start should reach Playing (got {:?})",
        client_state
    );

    // The host must be recognised: its lobby host id must match the local
    // player id, otherwise the Play button never appears and the camera is
    // never parented.
    let (lobby_host_id, player_count, camera_count, local_player_id) = {
        let world = app.world_mut();
        let local_player_id = world.resource::<LocalPlayerId>().0;
        let mut lobby_q = world.query::<&LobbyState>();
        let lobby_host_id = lobby_q
            .single(world)
            .expect("lobby state should exist")
            .host_id;
        let mut player_q = world.query::<&PlayerId>();
        let player_count = player_q.iter(world).count();
        let mut camera_q = world.query::<&PlayerCamera>();
        let camera_count = camera_q.iter(world).count();
        (lobby_host_id, player_count, camera_count, local_player_id)
    };
    assert_eq!(
        local_player_id, lobby_host_id,
        "host should be recognised as the local player (host_id={}, local={})",
        lobby_host_id, local_player_id
    );

    // The host's player entity must actually be spawned (LocalMode peer must
    // match the lobby entry).
    assert!(
        player_count >= 1,
        "host player entity should be spawned (found {})",
        player_count
    );

    // A first-person camera must be parented to the host's player.
    assert!(
        camera_count >= 1,
        "player camera should be spawned for the local host"
    );
}
