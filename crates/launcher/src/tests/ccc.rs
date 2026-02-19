use super::*;
use avian3d::prelude::{LinearVelocity, Position, Rotation};
use bevy::prelude::{Entity, IntoScheduleConfigs, Query, Res, Time, Transform, Update, Vec2, Vec3};
use client::camera::PlayerCamera;
use leafwing_input_manager::prelude::ActionState;
use lightyear::prelude::{Controlled, PeerId, Predicted};
use shared::inputs::movement::GroundState;
use shared::protocol::CharacterMarker;
use shared::inputs::input::PlayerAction;
use shared::protocol::PlayerId;

fn integrate_position_from_velocity(
    mut query: Query<(&mut Position, &LinearVelocity)>,
    time: Res<Time>,
) {
    for (mut position, velocity) in query.iter_mut() {
        position.0 += velocity.0 * time.delta_secs();
    }
}

fn spawn_local_player_for_ccc(client_app: &mut App, player_id: u64) -> Entity {
    let mut action_state = ActionState::<PlayerAction>::default();
    action_state.enable();

    client_app.world_mut().spawn((
        PlayerId(PeerId::Netcode(player_id)),
        Predicted,
        Controlled,
        CharacterMarker,
        Position::new(Vec3::new(0.0, 2.0, 0.0)),
        Rotation::default(),
        LinearVelocity::default(),
        GroundState {
            is_grounded: true,
            ground_normal: Vec3::Y,
            ground_distance: 0.0,
            ground_tick: 0,
        },
        action_state,
    )).id()
}

fn set_client_move_input(client_app: &mut App, player_entity: Entity, axis: Vec2) {
    let world = client_app.world_mut();
    if let Some(mut action_state) = world.get_mut::<ActionState<PlayerAction>>(player_entity) {
        action_state.enable();
        action_state.set_axis_pair(&PlayerAction::Move, axis);
    }
}

fn set_client_look_input(client_app: &mut App, player_entity: Entity, axis: Vec2) {
    let world = client_app.world_mut();
    if let Some(mut action_state) = world.get_mut::<ActionState<PlayerAction>>(player_entity) {
        action_state.enable();
        action_state.set_axis_pair(&PlayerAction::Look, axis);
    }
}

fn client_camera_transform(client_app: &mut App) -> Transform {
    let world = client_app.world_mut();
    let mut query = world.query_filtered::<&Transform, bevy::prelude::With<PlayerCamera>>();
    query
        .iter(world)
        .next()
        .copied()
        .expect("client should have a PlayerCamera transform")
}

#[test]
fn test_ccc_mouse_look_rotates_character_and_camera_end_to_end() {
    let mut client_app = create_test_client_app_with_mode(1, true, NetworkMode::Local);
    client_app.insert_state(ClientGameState::Playing);
    client_app.add_systems(
        Update,
        (
            shared::inputs::movement::apply_movement,
            integrate_position_from_velocity,
        )
            .chain(),
    );

    let local_player_entity = spawn_local_player_for_ccc(&mut client_app, 1);

    for _ in 0..6 {
        update_single_app(&mut client_app, Duration::from_millis(16));
    }

    let initial_local_rotation = client_app
        .world()
        .get::<Rotation>(local_player_entity)
        .expect("local player should have rotation")
        .0;
    let initial_camera_transform = client_camera_transform(&mut client_app);

    for _ in 0..24 {
        set_client_look_input(&mut client_app, local_player_entity, Vec2::new(300.0, 45.0));
        update_single_app(&mut client_app, Duration::from_millis(16));
    }
    set_client_look_input(&mut client_app, local_player_entity, Vec2::ZERO);
    for _ in 0..10 {
        update_single_app(&mut client_app, Duration::from_millis(16));
    }

    let settled_rotation = client_app
        .world()
        .get::<Rotation>(local_player_entity)
        .expect("local player should still have rotation after look release")
        .0;
    let settled_camera_transform = client_camera_transform(&mut client_app);

    for _ in 0..24 {
        update_single_app(&mut client_app, Duration::from_millis(16));
    }

    let updated_local_rotation = client_app
        .world()
        .get::<Rotation>(local_player_entity)
        .expect("local player should still have rotation")
        .0;
    let updated_camera_transform = client_camera_transform(&mut client_app);

    let local_dot = initial_local_rotation.dot(updated_local_rotation).abs();
    let camera_dot = initial_camera_transform
        .rotation
        .dot(updated_camera_transform.rotation)
        .abs();

    assert!(
        local_dot < 0.999,
        "Local player rotation should change from look input, dot={}",
        local_dot
    );
    assert!(
        camera_dot < 0.999,
        "Camera rotation should change with character look, dot={}",
        camera_dot
    );

    let camera_to_local_dot = updated_camera_transform
        .rotation
        .dot(updated_local_rotation)
        .abs();
    assert!(
        camera_to_local_dot > 0.995,
        "Camera and local player rotations should stay aligned, dot={}",
        camera_to_local_dot
    );

    let no_snapback_local_dot = settled_rotation.dot(updated_local_rotation).abs();
    let no_snapback_camera_dot = settled_camera_transform
        .rotation
        .dot(updated_camera_transform.rotation)
        .abs();

    assert!(
        no_snapback_local_dot > 0.995,
        "Local rotation should remain stable after look release (no snapback), dot={}",
        no_snapback_local_dot
    );
    assert!(
        no_snapback_camera_dot > 0.995,
        "Camera rotation should remain stable after look release (no snapback), dot={}",
        no_snapback_camera_dot
    );
}

#[test]
fn test_ccc_move_input_moves_character_and_camera_end_to_end() {
    let mut client_app = create_test_client_app_with_mode(1, true, NetworkMode::Local);
    client_app.insert_state(ClientGameState::Playing);
    client_app.add_systems(
        Update,
        (
            shared::inputs::movement::apply_movement,
            integrate_position_from_velocity,
        )
            .chain(),
    );

    let local_player_entity = spawn_local_player_for_ccc(&mut client_app, 1);

    for _ in 0..6 {
        update_single_app(&mut client_app, Duration::from_millis(16));
    }

    let initial_local_position = client_app
        .world()
        .get::<Position>(local_player_entity)
        .expect("local player should have position")
        .0;
    let initial_camera_transform = client_camera_transform(&mut client_app);

    for _ in 0..40 {
        set_client_move_input(&mut client_app, local_player_entity, Vec2::new(0.0, 1.0));
        update_single_app(&mut client_app, Duration::from_millis(16));
    }
    set_client_move_input(&mut client_app, local_player_entity, Vec2::ZERO);
    for _ in 0..12 {
        update_single_app(&mut client_app, Duration::from_millis(16));
    }

    let updated_local_position = client_app
        .world()
        .get::<Position>(local_player_entity)
        .expect("local player should still have position")
        .0;
    let updated_camera_transform = client_camera_transform(&mut client_app);

    let local_displacement = updated_local_position.distance(initial_local_position);
    let camera_displacement = updated_camera_transform
        .translation
        .distance(initial_camera_transform.translation);

    assert!(
        local_displacement > 0.25,
        "Local player should move from movement input, displacement={}",
        local_displacement
    );
    assert!(
        camera_displacement > 0.20,
        "Camera should follow local player movement, displacement={}",
        camera_displacement
    );
}
