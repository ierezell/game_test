use super::*;
use avian3d::prelude::{LinearVelocity, Position, Rotation};
use bevy::prelude::{
    Commands, Entity, GlobalTransform, IntoScheduleConfigs, Plugin, Query, Res, Time, Transform,
    Update, Vec2, Vec3, With,
};
use bevy_enhanced_input::action::mock::ActionMock;
use bevy_enhanced_input::prelude::*;
use client::camera::PlayerCamera;
use lightyear::prelude::{Controlled, PeerId, Predicted};
use shared::entities::PlayerPhysicsBundle;
use shared::inputs::movement::GroundState;
use shared::inputs::{Look, Move};
use shared::protocol::{CharacterMarker, PlayerId};

fn integrate_position_from_velocity(
    mut query: Query<(
        &mut Position,
        &Rotation,
        &LinearVelocity,
        Option<&mut Transform>,
    )>,
    time: Res<Time>,
) {
    for (mut position, rotation, velocity, mut transform) in query.iter_mut() {
        position.0 += velocity.0 * time.delta_secs();
        if let Some(ref mut t) = transform {
            t.translation = position.0;
            t.rotation = rotation.0;
        }
    }
}

fn spawn_local_player_for_ccc(client_app: &mut App, player_id: u64) -> Entity {
    let player_entity = client_app
        .world_mut()
        .spawn((
            PlayerId(PeerId::Netcode(player_id)),
            Predicted,
            Controlled,
            CharacterMarker,
            Position::new(Vec3::new(0.0, 2.0, 0.0)),
            Rotation::default(),
            PlayerPhysicsBundle::default(),
            Transform::from_xyz(0.0, 2.0, 0.0),
            GlobalTransform::IDENTITY,
            LinearVelocity::default(),
            GroundState {
                is_grounded: true,
                ground_normal: Vec3::Y,
                ground_distance: 0.0,
                ground_tick: 0,
            },
            shared::inputs::get_player_actions(),
            bevy::prelude::Visibility::default(),
        ))
        .id();

    for _ in 0..4 {
        update_single_app(client_app, Duration::from_millis(16));
    }

    let has_player_camera = {
        let world = client_app.world_mut();
        let mut query = world.query_filtered::<Entity, With<PlayerCamera>>();
        query.iter(world).next().is_some()
    };
    assert!(
        has_player_camera,
        "expected PlayerCamera to be spawned by camera systems after player spawn"
    );

    player_entity
}

#[derive(bevy::prelude::Resource, Default, Clone)]
struct TestInput {
    look: Vec2,
    move_axis: Vec2,
}

fn advance_frames(app: &mut App, frames: u32) {
    for _ in 0..frames {
        update_single_app(app, Duration::from_millis(16));
    }
}

#[allow(clippy::collapsible_if)]
fn apply_test_input_system(
    test_input: Res<TestInput>,
    player_query: Query<Entity, With<CharacterMarker>>,
    mut move_action_query: Query<&mut Action<Move>>,
    mut look_action_query: Query<&mut Action<Look>>,
    actions_query: Query<&Actions<shared::inputs::PlayerActions>>,
    mut commands: Commands,
) {
    for player_entity in player_query.iter() {
        // Find the Actions relationship target to get child action entities
        if let Ok(actions) = actions_query.get(player_entity) {
            for action_entity in actions.iter() {
                if test_input.look != Vec2::ZERO {
                    if let Ok(mut look_action) = look_action_query.get_mut(*action_entity) {
                        **look_action = test_input.look;
                        commands.entity(*action_entity).insert(ActionMock::new(
                            TriggerState::Fired,
                            test_input.look,
                            MockSpan::Manual,
                        ));
                    }
                }
                if test_input.move_axis != Vec2::ZERO {
                    if let Ok(mut move_action) = move_action_query.get_mut(*action_entity) {
                        **move_action = test_input.move_axis;
                        commands.entity(*action_entity).insert(ActionMock::new(
                            TriggerState::Fired,
                            test_input.move_axis,
                            MockSpan::Manual,
                        ));
                    }
                }
            }
        }
    }
}

fn set_client_move_input(client_app: &mut App, player_entity: Entity, axis: Vec2) {
    let world = client_app.world_mut();
    let actions = world.get::<Actions<shared::inputs::PlayerActions>>(player_entity);
    let action_entities: Vec<_> = actions
        .map(|actions| actions.iter().copied().collect())
        .unwrap_or_default();
    let world = client_app.world_mut();
    for action_entity in action_entities {
        if let Some(mut move_action) = world.get_mut::<Action<Move>>(action_entity) {
            **move_action = axis;
            world.entity_mut(action_entity).insert(ActionMock::new(
                TriggerState::Fired,
                axis,
                MockSpan::Manual,
            ));
        }
    }
}

fn set_client_look_input(client_app: &mut App, player_entity: Entity, axis: Vec2) {
    let world = client_app.world_mut();
    let actions = world.get::<Actions<shared::inputs::PlayerActions>>(player_entity);
    let action_entities: Vec<_> = actions
        .map(|actions| actions.iter().copied().collect())
        .unwrap_or_default();
    let world = client_app.world_mut();
    for action_entity in action_entities {
        if let Some(mut look_action) = world.get_mut::<Action<Look>>(action_entity) {
            **look_action = axis;
            world.entity_mut(action_entity).insert(ActionMock::new(
                TriggerState::Fired,
                axis,
                MockSpan::Manual,
            ));
        }
    }
}

fn client_camera_global_transform(client_app: &mut App) -> GlobalTransform {
    let world = client_app.world_mut();
    let mut query = world.query_filtered::<&GlobalTransform, With<PlayerCamera>>();
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
    client_app.init_resource::<TestInput>();
    client_app.add_systems(
        Update,
        (
            apply_test_input_system,
            shared::inputs::look::update_player_rotation_from_input,
            integrate_position_from_velocity,
        )
            .chain(),
    );
    super::finish_if_needed(&mut client_app);

    let local_player_entity = spawn_local_player_for_ccc(&mut client_app, 1);

    let initial_local_rotation = client_app
        .world()
        .get::<Rotation>(local_player_entity)
        .expect("local player should have rotation")
        .0;
    let initial_camera_transform = client_camera_global_transform(&mut client_app);

    for _ in 0..24 {
        set_client_look_input(&mut client_app, local_player_entity, Vec2::new(300.0, 45.0));
        advance_frames(&mut client_app, 1);
    }
    set_client_look_input(&mut client_app, local_player_entity, Vec2::ZERO);
    advance_frames(&mut client_app, 10);

    let settled_rotation = client_app
        .world()
        .get::<Rotation>(local_player_entity)
        .expect("local player should still have rotation after look release")
        .0;
    let settled_camera_transform = client_camera_global_transform(&mut client_app);

    advance_frames(&mut client_app, 24);

    let updated_local_rotation = client_app
        .world()
        .get::<Rotation>(local_player_entity)
        .expect("local player should still have rotation")
        .0;
    let updated_camera_transform = client_camera_global_transform(&mut client_app);

    let local_dot = initial_local_rotation.dot(updated_local_rotation).abs();
    let camera_dot = initial_camera_transform
        .compute_transform()
        .rotation
        .dot(updated_camera_transform.compute_transform().rotation)
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
        .compute_transform()
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
        .compute_transform()
        .rotation
        .dot(updated_camera_transform.compute_transform().rotation)
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
    client_app.init_resource::<TestInput>();
    client_app.add_systems(
        Update,
        (
            apply_test_input_system,
            shared::inputs::look::update_player_rotation_from_input,
            shared::inputs::movement::apply_movement,
            integrate_position_from_velocity,
        )
            .chain(),
    );
    super::finish_if_needed(&mut client_app);

    let local_player_entity = spawn_local_player_for_ccc(&mut client_app, 1);

    let initial_local_position = client_app
        .world()
        .get::<Position>(local_player_entity)
        .expect("local player should have position")
        .0;
    let initial_camera_transform = client_camera_global_transform(&mut client_app);

    for _ in 0..40 {
        set_client_move_input(&mut client_app, local_player_entity, Vec2::new(0.0, 1.0));
        advance_frames(&mut client_app, 1);
    }
    set_client_move_input(&mut client_app, local_player_entity, Vec2::ZERO);
    advance_frames(&mut client_app, 12);

    let updated_local_position = client_app
        .world()
        .get::<Position>(local_player_entity)
        .expect("local player should still have position")
        .0;
    let updated_camera_transform = client_camera_global_transform(&mut client_app);

    let local_displacement = updated_local_position.distance(initial_local_position);
    let camera_displacement = updated_camera_transform
        .translation()
        .distance(initial_camera_transform.translation());

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

/// Regression test for the `MovementPlugin` wiring: before the plugin was
/// registered, `Action<Move>`/`Action<Look>` were computed by EnhancedInput but
/// never consumed, so WASD and mouse look did nothing at runtime.
///
/// Spawns a non-simulated local player (no `RigidBody`) and drives it through
/// the real `MovementPlugin`, asserting that:
///   - injected look input rotates the character *and* the parented camera, and
///   - injected move input translates the character *and* the camera follows.
#[test]
fn test_movement_plugin_moves_and_rotates_character_end_to_end() {
    let mut client_app = create_test_client_app_with_mode(1, true, NetworkMode::Local);
    client_app.insert_state(ClientGameState::Playing);
    client_app.init_resource::<TestInput>();
    client_app.add_systems(Update, apply_test_input_system);
    // Register the real movement wiring. The test harness finalises plugins
    // internally, so we drive the plugin's `build` directly (same systems the
    // production builders register via `App::add_plugins`).
    shared::inputs::movement::MovementPlugin::new(false).build(&mut client_app);
    super::finish_if_needed(&mut client_app);

    let player = spawn_local_player_for_movement_plugin(&mut client_app, 1);

    let initial_position = client_app
        .world()
        .get::<Position>(player)
        .expect("local player should have a Position")
        .0;
    let initial_rotation = client_app
        .world()
        .get::<Rotation>(player)
        .expect("local player should have a Rotation")
        .0;
    let initial_camera = client_camera_global_transform(&mut client_app);

    // --- Mouse look: turning the character should turn the camera too. ---
    set_client_look_input(&mut client_app, player, Vec2::new(250.0, 0.0));
    for _ in 0..20 {
        update_single_app(&mut client_app, Duration::from_millis(16));
    }
    set_client_look_input(&mut client_app, player, Vec2::ZERO);
    for _ in 0..8 {
        update_single_app(&mut client_app, Duration::from_millis(16));
    }

    let rotated_character = client_app
        .world()
        .get::<Rotation>(player)
        .expect("local player should still have a Rotation")
        .0;
    let rotated_camera = client_camera_global_transform(&mut client_app);

    let char_rotation_dot = initial_rotation.dot(rotated_character).abs();
    assert!(
        char_rotation_dot < 0.999,
        "character should rotate from look input, dot={}",
        char_rotation_dot
    );

    let camera_rotation_dot = initial_camera
        .compute_transform()
        .rotation
        .dot(rotated_camera.compute_transform().rotation)
        .abs();
    assert!(
        camera_rotation_dot < 0.999,
        "camera should turn with the character, dot={}",
        camera_rotation_dot
    );

    let settled_rotation = rotated_character;

    // --- WASD: moving forward should translate the character and camera. ---
    set_client_move_input(&mut client_app, player, Vec2::new(0.0, 1.0));
    for _ in 0..40 {
        update_single_app(&mut client_app, Duration::from_millis(16));
    }
    set_client_move_input(&mut client_app, player, Vec2::ZERO);
    for _ in 0..12 {
        update_single_app(&mut client_app, Duration::from_millis(16));
    }

    let moved_position = client_app
        .world()
        .get::<Position>(player)
        .expect("local player should still have a Position")
        .0;
    let moved_camera = client_camera_global_transform(&mut client_app);

    let translation = moved_position - initial_position;
    assert!(
        translation.length() > 0.25,
        "character should translate from movement input, displacement={}",
        translation.length()
    );

    let camera_translation = moved_camera
        .translation()
        .distance(initial_camera.translation());
    assert!(
        camera_translation > 0.20,
        "camera should follow character translation, displacement={}",
        camera_translation
    );

    // Rotation should remain stable after movement input is released.
    let final_rotation = client_app
        .world()
        .get::<Rotation>(player)
        .expect("local player should still have a Rotation")
        .0;
    assert!(
        settled_rotation.dot(final_rotation).abs() > 0.995,
        "character rotation should stay stable during movement, dot={}",
        settled_rotation.dot(final_rotation).abs()
    );
}

fn spawn_local_player_for_movement_plugin(client_app: &mut App, player_id: u64) -> Entity {
    // Mirror the battle-tested ccc spawn exactly (RigidBody::Dynamic + collider
    // + prediction markers) so the `PlayerCamera` observer fires and lightyear
    // prediction behaves as in the rest of the suite. Movement is driven by the
    // real `MovementPlugin` (which leaves simulated bodies to avian).
    let player_entity = client_app
        .world_mut()
        .spawn((
            PlayerId(PeerId::Netcode(player_id)),
            Predicted,
            Controlled,
            CharacterMarker,
            Position::new(Vec3::new(0.0, 2.0, 0.0)),
            Rotation::default(),
            PlayerPhysicsBundle::default(),
            Transform::from_xyz(0.0, 2.0, 0.0),
            GlobalTransform::IDENTITY,
            LinearVelocity::default(),
            GroundState {
                is_grounded: true,
                ground_normal: Vec3::Y,
                ground_distance: 0.0,
                ground_tick: 0,
            },
            shared::inputs::get_player_actions(),
            bevy::prelude::Visibility::default(),
        ))
        .id();

    for _ in 0..4 {
        update_single_app(client_app, Duration::from_millis(16));
    }

    let has_player_camera = {
        let world = client_app.world_mut();
        let mut query = world.query_filtered::<Entity, With<PlayerCamera>>();
        query.iter(world).next().is_some()
    };
    assert!(
        has_player_camera,
        "expected PlayerCamera to be spawned by camera systems after player spawn"
    );

    player_entity
}
