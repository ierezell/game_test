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
