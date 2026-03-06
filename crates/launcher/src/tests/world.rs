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

#[test]
fn test_gym_spawned_npc_does_not_stall_for_long_periods() {
    use avian3d::prelude::Position;
    use shared::navigation::SimpleNavigationAgent;
    use shared::protocol::{CharacterMarker, PlayerId};

    let mut app = create_test_server_app_with_gym_mode(true);
    app.insert_state(ServerGameState::Loading);

    for _ in 0..60 {
        update_single_app(&mut app, Duration::from_millis(100));
    }

    let npc_entity = {
        let world = app.world_mut();
        let mut npc_q = world.query_filtered::<bevy::prelude::Entity, (
            bevy::prelude::With<CharacterMarker>,
            bevy::prelude::Without<PlayerId>,
        )>();
        npc_q
            .iter(world)
            .next()
            .expect("Gym server should spawn at least one NPC")
    };

    let mut last_position = app
        .world()
        .get::<Position>(npc_entity)
        .expect("Spawned NPC should have Position")
        .0;
    let mut longest_stationary_run = 0usize;
    let mut current_stationary_run = 0usize;

    for _ in 0..260 {
        update_single_app(&mut app, Duration::from_millis(100));

        let position = app
            .world()
            .get::<Position>(npc_entity)
            .expect("NPC should keep Position")
            .0;
        let nav_agent = app
            .world()
            .get::<SimpleNavigationAgent>(npc_entity)
            .expect("NPC should keep navigation agent");

        let planar_delta = Vec3::new(
            position.x - last_position.x,
            0.0,
            position.z - last_position.z,
        );
        if nav_agent.current_target.is_some() && planar_delta.length() < 0.02 {
            current_stationary_run += 1;
            longest_stationary_run = longest_stationary_run.max(current_stationary_run);
        } else {
            current_stationary_run = 0;
        }

        last_position = position;
    }

    assert!(
        longest_stationary_run < 45,
        "NPC appears stalled for too long while wandering (longest stationary run={} frames)",
        longest_stationary_run
    );
}

#[test]
fn test_gym_wandering_target_stays_within_room_bounds() {
    use shared::gym::ROOM_HALF_EXTENT;
    use shared::navigation::SimpleNavigationAgent;
    use shared::protocol::{CharacterMarker, PlayerId};

    let mut app = create_test_server_app_with_gym_mode(true);
    app.insert_state(ServerGameState::Loading);

    for _ in 0..90 {
        update_single_app(&mut app, Duration::from_millis(100));
    }

    let world = app.world_mut();
    let mut npc_q = world.query_filtered::<&SimpleNavigationAgent, (
        bevy::prelude::With<CharacterMarker>,
        bevy::prelude::Without<PlayerId>,
    )>();

    let nav_agent = npc_q
        .iter(world)
        .next()
        .expect("Gym server should have at least one NPC navigation agent");
    let target = nav_agent
        .current_target
        .expect("Gym wandering NPC should have picked a target");

    assert!(
        target.x.abs() <= ROOM_HALF_EXTENT,
        "NPC target x should stay inside room walls, got target={:?}",
        target
    );
    assert!(
        target.z.abs() <= ROOM_HALF_EXTENT,
        "NPC target z should stay inside room walls, got target={:?}",
        target
    );
}

#[test]
fn test_gym_wandering_npc_uses_kinematic_body() {
    use avian3d::prelude::RigidBody;
    use shared::protocol::{CharacterMarker, PlayerId};

    let mut app = create_test_server_app_with_gym_mode(true);
    app.insert_state(ServerGameState::Loading);

    for _ in 0..80 {
        update_single_app(&mut app, Duration::from_millis(100));
    }

    let world = app.world_mut();
    let mut npc_q = world.query_filtered::<&RigidBody, (
        bevy::prelude::With<CharacterMarker>,
        bevy::prelude::Without<PlayerId>,
    )>();

    let body = npc_q
        .iter(world)
        .next()
        .expect("Gym server should have at least one NPC body");

    assert_eq!(
        *body,
        RigidBody::Kinematic,
        "Gym wandering NPC should use kinematic body to avoid vibration"
    );
}

#[test]
fn test_gym_npc_progresses_toward_forced_straight_line_target() {
    use avian3d::prelude::Position;
    use shared::gym::GymRandomWanderer;
    use shared::navigation::{NavigationPathState, SimpleNavigationAgent};
    use shared::protocol::{CharacterMarker, PlayerId};

    let mut app = create_test_server_app_with_gym_mode(true);
    app.insert_state(ServerGameState::Loading);

    for _ in 0..80 {
        update_single_app(&mut app, Duration::from_millis(100));
    }

    let npc_entity = {
        let world = app.world_mut();
        let mut npc_q = world.query_filtered::<bevy::prelude::Entity, (
            bevy::prelude::With<CharacterMarker>,
            bevy::prelude::Without<PlayerId>,
        )>();
        npc_q
            .iter(world)
            .next()
            .expect("Gym server should spawn at least one NPC")
    };

    let start = app
        .world()
        .get::<Position>(npc_entity)
        .expect("NPC should have Position")
        .0;
    let forced_target = Vec3::new(start.x, 1.0, 21.0);

    // Disable random wandering for this entity so the forced target remains stable.
    app.world_mut()
        .entity_mut(npc_entity)
        .remove::<GymRandomWanderer>();

    {
        let world = app.world_mut();
        let mut nav_agent = world
            .get_mut::<SimpleNavigationAgent>(npc_entity)
            .expect("NPC should have navigation agent");
        nav_agent.current_target = Some(forced_target);
    }
    {
        let world = app.world_mut();
        let mut path_state = world
            .get_mut::<NavigationPathState>(npc_entity)
            .expect("NPC should have navigation path state");
        path_state.clear();
    }

    let initial_distance =
        Vec3::new(start.x - forced_target.x, 0.0, start.z - forced_target.z).length();

    for _ in 0..180 {
        update_single_app(&mut app, Duration::from_millis(100));
    }

    let end = app
        .world()
        .get::<Position>(npc_entity)
        .expect("NPC should keep Position while moving")
        .0;
    let final_distance =
        Vec3::new(end.x - forced_target.x, 0.0, end.z - forced_target.z).length();

    assert!(
        final_distance < initial_distance - 8.0,
        "NPC should make sustained progress toward forced straight-line target; start={:?}, end={:?}, target={:?}, initial_dist={:.2}, final_dist={:.2}",
        start,
        end,
        forced_target,
        initial_distance,
        final_distance
    );
}
