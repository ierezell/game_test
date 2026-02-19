use lightyear_tests::stepper::{ClientServerStepper, StepperConfig};
use std::time::Duration;

#[test]
fn test_deterministic_stepper_two_clients_connected() {
    let mut config = StepperConfig::with_netcode_clients(2);
    config.tick_duration = Duration::from_millis(16);
    config.frame_duration = Duration::from_millis(16);
    config.init = true;
    let mut stepper = ClientServerStepper::from_config(config);

    stepper.frame_step(10);

    assert_eq!(
        stepper.client_apps.len(),
        2,
        "deterministic stepper should create two client apps"
    );
    assert_eq!(
        stepper.client_entities.len(),
        2,
        "deterministic stepper should create two client entities"
    );
    assert_eq!(
        stepper.client_of_entities.len(),
        2,
        "deterministic stepper should create two server-side client links"
    );
}
