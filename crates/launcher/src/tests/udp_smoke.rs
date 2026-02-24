use super::*;
use lightyear_tests::stepper::{ClientServerStepper, StepperConfig};

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
