use super::*;

use avian3d::prelude::Position;
use bevy::prelude::Vec3;
use bevy::prelude::{Entity, State as BevyState};
use bevy_enhanced_input::prelude::*;
use lightyear::prelude::PeerId;
use shared::components::flashlight::PlayerFlashlight;
use shared::inputs::ToggleFlashlight;
use shared::protocol::{CharacterMarker, PlayerId};

fn wait_until_crossbeam_playing(
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
            .resource::<BevyState<ServerGameState>>()
            .get()
            .clone();
        if server_state == ServerGameState::Playing {
            break;
        }
    }

    for _ in 0..500 {
        update_all(server_app, client_app1, client_app2);

        let c1 = client_app1
            .world()
            .resource::<BevyState<ClientGameState>>()
            .get()
            .clone();
        let c2 = client_app2
            .world()
            .resource::<BevyState<ClientGameState>>()
            .get()
            .clone();
        if c1 == ClientGameState::Playing && c2 == ClientGameState::Playing {
            return;
        }
    }
}

fn force_client_game_start(client_app: &mut App, gym_mode: bool) {
    use bevy::state::prelude::NextState;
    use shared::protocol::LevelSeed;

    let current_state = client_app
        .world()
        .resource::<BevyState<ClientGameState>>()
        .get()
        .clone();

    if current_state != ClientGameState::Playing {
        if !gym_mode {
            client_app.world_mut().spawn(LevelSeed { seed: 42 });
        }
        {
            let mut next = client_app
                .world_mut()
                .resource_mut::<NextState<ClientGameState>>();
            next.set(ClientGameState::Loading);
        }

        for _ in 0..100 {
            client_app.update();
            let state = client_app
                .world()
                .resource::<BevyState<ClientGameState>>()
                .get()
                .clone();
            if state == ClientGameState::Playing {
                break;
            }
        }
    }
}

fn server_player_position_by_peer_id(server_app: &mut App, peer_id: u64) -> Option<Vec3> {
    let world = server_app.world_mut();
    let mut q =
        world.query_filtered::<(&PlayerId, &Position), bevy::prelude::With<CharacterMarker>>();
    q.iter(world).find_map(|(pid, position)| match pid.0 {
        PeerId::Netcode(id) if id == peer_id => Some(position.0),
        _ => None,
    })
}

#[allow(clippy::collapsible_if)]
fn translate_server_player_by_id(server_app: &mut App, peer_id: u64, delta: Vec3) -> bool {
    let world = server_app.world_mut();
    let mut q =
        world.query_filtered::<(&PlayerId, bevy::prelude::Entity), bevy::prelude::With<PlayerId>>();
    let entity = q.iter(world).find_map(|(pid, e)| match pid.0 {
        PeerId::Netcode(id) if id == peer_id => Some(e),
        _ => None,
    });
    if let Some(e) = entity {
        if let Some(mut position) = world.get_mut::<Position>(e) {
            position.0 += delta;
            return true;
        }
    }
    false
}

#[test]
fn test_server_player_position_tracked_after_game_start() {
    let (mut server_app, mut client_app1, mut _client_app2) = setup_two_client_server(true);

    wait_until_crossbeam_playing(&mut server_app, &mut client_app1, &mut _client_app2);
    for client_app in [&mut client_app1, &mut _client_app2] {
        force_client_game_start(client_app, true);
    }

    let server_pos = server_player_position_by_peer_id(&mut server_app, 1);
    assert!(
        server_pos.is_some(),
        "Server should track player 1 position in Playing state"
    );
    let server_pos2 = server_player_position_by_peer_id(&mut server_app, 2);
    assert!(
        server_pos2.is_some(),
        "Server should track player 2 position in Playing state"
    );
}

#[test]
fn test_server_correction_updates_authoritative_position() {
    let (mut server_app, mut client_app1, mut _client_app2) = setup_two_client_server(true);

    wait_until_crossbeam_playing(&mut server_app, &mut client_app1, &mut _client_app2);
    for client_app in [&mut client_app1, &mut _client_app2] {
        force_client_game_start(client_app, true);
    }

    let before = server_player_position_by_peer_id(&mut server_app, 1)
        .expect("Server should track player 1 position");

    let teleport = Vec3::new(0.0, 50.0, 0.0);
    assert!(
        translate_server_player_by_id(&mut server_app, 1, teleport),
        "Server should be able to teleport player 1"
    );

    let after = server_player_position_by_peer_id(&mut server_app, 1)
        .expect("Server should still track player 1 position");

    let delta = (after - before).length();
    assert!(
        delta > 49.0,
        "Server correction should move player position, delta={}",
        delta
    );
}

#[test]
fn test_server_player_positions_are_independent() {
    let (mut server_app, mut client_app1, mut _client_app2) = setup_two_client_server(true);

    wait_until_crossbeam_playing(&mut server_app, &mut client_app1, &mut _client_app2);
    for client_app in [&mut client_app1, &mut _client_app2] {
        force_client_game_start(client_app, true);
    }

    let pos1 = server_player_position_by_peer_id(&mut server_app, 1)
        .expect("Server should track player 1 position");
    let pos2 = server_player_position_by_peer_id(&mut server_app, 2)
        .expect("Server should track player 2 position");

    let separation = (pos1 - pos2).length();
    assert!(
        separation > 1.0,
        "Two players should have distinct positions, separation={}",
        separation
    );
}

#[test]
fn test_server_player_position_persists_across_updates() {
    let (mut server_app, mut client_app1, mut _client_app2) = setup_two_client_server(true);

    wait_until_crossbeam_playing(&mut server_app, &mut client_app1, &mut _client_app2);
    for client_app in [&mut client_app1, &mut _client_app2] {
        force_client_game_start(client_app, true);
    }

    let pos_before = server_player_position_by_peer_id(&mut server_app, 1)
        .expect("Server should track player 1 position");

    for _ in 0..50 {
        update_all(&mut server_app, &mut client_app1, &mut _client_app2);
    }

    let pos_after = server_player_position_by_peer_id(&mut server_app, 1)
        .expect("Server should still track player 1 position after updates");

    let delta = (pos_before - pos_after).length();
    assert!(
        delta < 5.0,
        "Server player position should not drift significantly, delta={}",
        delta
    );
}

#[test]
fn test_server_player_position_changes_after_force_start() {
    let (mut server_app, mut client_app1, mut _client_app2) = setup_two_client_server(true);

    wait_until_crossbeam_playing(&mut server_app, &mut client_app1, &mut _client_app2);

    let pos_before_force = server_player_position_by_peer_id(&mut server_app, 1);

    for client_app in [&mut client_app1, &mut _client_app2] {
        force_client_game_start(client_app, true);
    }

    let pos_after_force = server_player_position_by_peer_id(&mut server_app, 1);

    assert!(
        pos_before_force.is_some() || pos_after_force.is_some(),
        "Server should track player 1 position either before or after force start"
    );

    if let (Some(before), Some(after)) = (pos_before_force, pos_after_force) {
        let delta = (before - after).length();
        assert!(
            delta < 100.0,
            "Server player position should not jump excessively after force start, delta={}",
            delta
        );
    }
}

#[test]
fn test_server_correction_persists_through_subsequent_updates() {
    let (mut server_app, mut client_app1, mut _client_app2) = setup_two_client_server(true);

    wait_until_crossbeam_playing(&mut server_app, &mut client_app1, &mut _client_app2);
    for client_app in [&mut client_app1, &mut _client_app2] {
        force_client_game_start(client_app, true);
    }

    let teleport = Vec3::new(0.0, 100.0, 0.0);
    assert!(
        translate_server_player_by_id(&mut server_app, 1, teleport),
        "Server should be able to teleport player 1"
    );

    for _ in 0..50 {
        update_all(&mut server_app, &mut client_app1, &mut _client_app2);
    }

    let pos_after = server_player_position_by_peer_id(&mut server_app, 1)
        .expect("Server should still track player 1 position after updates");

    let delta = pos_after.y - 100.0;
    assert!(
        delta.abs() < 5.0,
        "Server correction should persist, y delta from expected={}",
        delta
    );
}

#[test]
fn test_server_player_count_stablest_after_force_start() {
    let (mut server_app, mut client_app1, mut _client_app2) = setup_two_client_server(true);

    wait_until_crossbeam_playing(&mut server_app, &mut client_app1, &mut _client_app2);

    let count_before = server_player_entity(&mut server_app, 1).is_some() as usize
        + server_player_entity(&mut server_app, 2).is_some() as usize;

    force_client_game_start(&mut client_app1, true);
    force_client_game_start(&mut _client_app2, true);

    for _ in 0..50 {
        update_all(&mut server_app, &mut client_app1, &mut _client_app2);
    }

    let count_after = server_player_entity(&mut server_app, 1).is_some() as usize
        + server_player_entity(&mut server_app, 2).is_some() as usize;

    assert!(
        count_before >= 1,
        "Server should have at least one player before force start, count={}",
        count_before
    );
    assert_eq!(
        count_before, count_after,
        "Server should have same player count before and after force start"
    );
}

#[test]
fn test_server_flashlight_toggle_observer_fires() {
    let (mut server_app, mut client_app1, mut _client_app2) = setup_two_client_server(true);

    wait_until_crossbeam_playing(&mut server_app, &mut client_app1, &mut _client_app2);

    let entity = server_player_entity(&mut server_app, 1).expect("Server should have player 1");

    {
        let world = server_app.world_mut();
        let flashlight = world.get::<PlayerFlashlight>(entity);
        assert!(
            flashlight.is_some(),
            "Player 1 should have a PlayerFlashlight component on the server"
        );
        let flashlight = flashlight.unwrap();
        assert!(
            flashlight.is_on,
            "Player 1 flashlight should start ON (is_on={})",
            flashlight.is_on
        );
    }

    server_app.world_mut().trigger(Start::<ToggleFlashlight> {
        context: entity,
        action: Entity::PLACEHOLDER,
        value: false,
        state: TriggerState::Fired,
    });

    for _ in 0..20 {
        server_app.update();
    }

    let world = server_app.world_mut();
    let flashlight = world
        .get::<PlayerFlashlight>(entity)
        .expect("Player 1 should still have PlayerFlashlight after toggle");
    assert!(
        !flashlight.is_on,
        "Player 1 flashlight should be OFF after toggle, is_on={}",
        flashlight.is_on
    );
}

#[test]
fn test_client_flashlight_toggle_replicates_to_server() {
    use crate::host::create_host_app;
    use bevy::prelude::State;
    use bevy::time::TimeUpdateStrategy;
    use bevy_enhanced_input::action::mock::ActionMock;
    use client::ClientGameState;
    use client::lobby::AutoStart;
    use lightyear::prelude::input::bei::Action;
    use server::lobby::AutoStartOnLobbyReady;
    use shared::components::weapons::Gun;
    use shared::inputs::PlayerActions;

    let mut app = create_host_app(true, "../../assets".to_string());
    app.insert_resource(bevy::ui::UiScale::default());
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
        16,
    )));
    app.insert_resource(AutoStart(true));
    app.insert_resource(AutoStartOnLobbyReady(true));
    app.insert_resource(shared::GymMode(true));
    finish_if_needed(&mut app);

    for _ in 0..800 {
        app.update();
        let state = app
            .world()
            .resource::<State<ClientGameState>>()
            .get()
            .clone();
        if state == ClientGameState::Playing {
            break;
        }
    }

    let server_entity = {
        let world = app.world_mut();
        let mut query = world.query_filtered::<bevy::prelude::Entity, (
            bevy::prelude::With<Gun>,
            bevy::prelude::With<PlayerId>,
        )>();
        query.iter(world).next()
    };

    let server_entity =
        server_entity.expect("Host should have a player entity with Gun and PlayerId");

    let server_flashlight_before = app
        .world()
        .get::<PlayerFlashlight>(server_entity)
        .expect("Player should have PlayerFlashlight before toggle");
    assert!(server_flashlight_before.is_on, "Flashlight should start ON");

    {
        let world = app.world_mut();
        let actions = world
            .get::<Actions<PlayerActions>>(server_entity)
            .expect("Player should have Actions<PlayerActions>");
        for action_entity in actions.iter() {
            if world
                .get::<Action<ToggleFlashlight>>(*action_entity)
                .is_some()
            {
                let mut action = world.entity_mut(*action_entity);
                **action.get_mut::<Action<ToggleFlashlight>>().unwrap() = true;
                action.insert(ActionMock::new(TriggerState::Fired, true, MockSpan::Manual));
                break;
            }
        }
    }

    for _ in 0..20 {
        app.update();
    }

    let server_flashlight_after = app
        .world()
        .get::<PlayerFlashlight>(server_entity)
        .expect("Player should still have PlayerFlashlight after toggle");
    assert!(
        !server_flashlight_after.is_on,
        "Flashlight should be OFF after client toggle, is_on={}",
        server_flashlight_after.is_on
    );
}
