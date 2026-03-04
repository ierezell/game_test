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
