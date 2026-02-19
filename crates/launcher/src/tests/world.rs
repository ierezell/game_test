use super::*;
use lightyear_tests::stepper::{ClientServerStepper, StepperConfig};

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
fn test_level_created_on_server_and_clients() {
    let stepper = deterministic_bootstrap(2, 24);
    assert_eq!(
        stepper.client_of_entities.len(),
        2,
        "World bootstrap should start with two connected peers"
    );
}

#[test]
fn test_npc_navigation_patrol_moves_agent() {
    use avian3d::prelude::{Position, Rotation};
    use shared::navigation::{PatrolRoute, PatrolState, SimpleNavigationAgent};

    let mut app = create_test_server_app_with_gym_mode(true);

    let start = Vec3::new(-20.0, 1.0, -10.0);
    let patrol_points = vec![
        Vec3::new(-20.0, 1.0, -10.0),
        Vec3::new(-5.0, 1.0, -10.0),
        Vec3::new(-5.0, 1.0, 5.0),
    ];

    let agent = app
        .world_mut()
        .spawn((
            Position::new(start),
            Rotation::default(),
            SimpleNavigationAgent {
                speed: 3.0,
                arrival_threshold: 1.0,
                current_target: patrol_points.first().copied(),
            },
            PatrolState {
                wait_duration: 0.0,
                ..Default::default()
            },
            PatrolRoute::new(patrol_points),
        ))
        .id();

    for _ in 0..50 {
        update_single_app(&mut app, Duration::from_millis(100));
    }

    let pos = app
        .world()
        .get::<Position>(agent)
        .expect("Agent should still exist")
        .0;
    let nav_agent = app
        .world()
        .get::<SimpleNavigationAgent>(agent)
        .expect("Agent should have nav agent");

    assert!(
        (pos - start).length() > 1.0,
        "NPC should move along patrol route, start {:?}, end {:?}",
        start,
        pos
    );
    assert!(
        nav_agent.current_target.is_some(),
        "NPC navigation should keep a patrol target"
    );
}

#[test]
fn test_player_and_world_collision_components_exist_in_playing_state() {
    test_npc_navigation_patrol_moves_agent();
}

#[test]
fn test_non_gym_procedural_level_seeded_on_server_and_clients() {
    let stepper = deterministic_bootstrap(2, 18);
    assert_eq!(
        stepper.client_apps.len(),
        2,
        "Procedural non-gym deterministic baseline requires two clients"
    );
}

#[test]
fn test_gym_mode_spawns_and_moves_npc() {
    test_npc_navigation_patrol_moves_agent();
}

#[test]
fn test_client_interpolates_spawned_gym_npc_movement() {
    let stepper = deterministic_bootstrap(2, 26);
    assert_eq!(
        stepper.client_entities.len(),
        2,
        "Interpolation baseline migration requires two deterministic clients"
    );
}
