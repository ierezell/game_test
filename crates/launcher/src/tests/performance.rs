use super::*;
use std::time::Instant;

fn wait_until_pair_server_playing(server_app: &mut App, client_app: &mut App) {
    server_app.insert_resource(server::lobby::AutoStartOnLobbyReady(true));

    for _ in 0..600 {
        update_pair(server_app, client_app);

        let server_state = server_app
            .world()
            .resource::<bevy::prelude::State<ServerGameState>>()
            .get()
            .clone();
        if server_lobby_player_count(server_app) >= 1 && server_state == ServerGameState::Playing {
            return;
        }
    }

    let server_state = server_app
        .world()
        .resource::<bevy::prelude::State<ServerGameState>>()
        .get()
        .clone();
    let client_state = client_app
        .world()
        .resource::<bevy::prelude::State<ClientGameState>>()
        .get()
        .clone();

    panic!(
        "Timed out waiting for server to enter Playing in single-pair perf harness: server={:?}, client={:?}, lobby_players={}",
        server_state,
        client_state,
        server_lobby_player_count(server_app)
    );
}

fn wait_until_two_client_server_playing(
    server_app: &mut App,
    client_app1: &mut App,
    client_app2: &mut App,
) {
    for _ in 0..900 {
        update_all(server_app, client_app1, client_app2);

        if server_lobby_player_count(server_app) >= 2 {
            server_app.insert_resource(server::lobby::AutoStartOnLobbyReady(true));
        }

        let server_state = server_app
            .world()
            .resource::<bevy::prelude::State<ServerGameState>>()
            .get()
            .clone();
        if server_state == ServerGameState::Playing {
            return;
        }
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

    panic!(
        "Timed out waiting for server to enter Playing in two-client perf harness: server={:?}, client1={:?}, client2={:?}, lobby_players={}",
        server_state,
        client1_state,
        client2_state,
        server_lobby_player_count(server_app)
    );
}

fn measure_pair_fps(gym_mode: bool, sample_frames: usize) -> f64 {
    let (mut server_app, mut client_app) = setup_one_client_server(gym_mode);
    wait_until_pair_server_playing(&mut server_app, &mut client_app);

    for _ in 0..120 {
        update_pair(&mut server_app, &mut client_app);
    }

    let start = Instant::now();
    for _ in 0..sample_frames {
        update_pair(&mut server_app, &mut client_app);
    }

    let elapsed = start.elapsed().as_secs_f64();
    sample_frames as f64 / elapsed
}

#[test]
fn simulation_sustains_over_30_fps_with_two_clients() {
    let (mut server_app, mut client_app1, mut client_app2) = setup_two_client_server(false);

    wait_until_two_client_server_playing(&mut server_app, &mut client_app1, &mut client_app2);

    // Warm up caches and schedules before measuring.
    for _ in 0..120 {
        update_all(&mut server_app, &mut client_app1, &mut client_app2);
    }

    let sample_frames = 600usize;
    let start = Instant::now();

    for _ in 0..sample_frames {
        update_all(&mut server_app, &mut client_app1, &mut client_app2);
    }

    let elapsed = start.elapsed().as_secs_f64();
    let fps = sample_frames as f64 / elapsed;

    assert!(
        fps > 30.0,
        "Expected >30 FPS simulation throughput, got {:.2} FPS over {} frames ({:.3}s elapsed)",
        fps,
        sample_frames,
        elapsed
    );
}

#[test]
fn gym_simulation_sustains_over_30_fps_and_stays_near_baseline() {
    let sample_frames = 600usize;
    let gym_fps = measure_pair_fps(true, sample_frames);
    let non_gym_fps = measure_pair_fps(false, sample_frames);

    assert!(
        gym_fps > 30.0,
        "Expected gym simulation throughput >30 FPS, got {:.2} FPS",
        gym_fps
    );
    assert!(
        gym_fps >= non_gym_fps * 0.7,
        "Gym simulation should not regress far below non-gym baseline. gym={:.2} FPS, non_gym={:.2} FPS",
        gym_fps,
        non_gym_fps
    );
}

#[test]
fn gym_simulation_sustains_over_60_fps_when_enforced() {
    let gym_fps = measure_pair_fps(true, 600);
    assert!(
        gym_fps >= 60.0,
        "Expected strict gym throughput >=60 FPS, got {:.2} FPS",
        gym_fps
    );
}
