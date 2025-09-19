use avian3d::prelude::*;
use bevy::prelude::*;

use leafwing_input_manager::prelude::*;

use lightyear::prelude::*;

use crate::app::LocalPlayerId;

use shared::input::{
    MOUSE_SENSITIVITY, PLAYER_CAPSULE_HEIGHT, PLAYER_CAPSULE_RADIUS, PlayerAction,
    shared_player_movement,
};
use shared::protocol::PlayerColor;
use shared::protocol::PlayerId;
use shared::scene::PlayerPhysicsBundle;

pub struct ClientInputPlugin;

impl Plugin for ClientInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, client_player_movement);
        app.add_systems(Update, update_player_rotation_from_inputs);
        app.add_observer(handle_player_spawn);
        app.add_observer(handle_other_players_spawn);
    }
}

fn handle_player_spawn(
    trigger: Trigger<OnAdd, (PlayerId, Predicted)>,
    mut player_query: Query<(&mut PlayerColor, &PlayerId, Option<&Name>)>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    local_player_id: Res<LocalPlayerId>,
) {
    let entity = trigger.target();

    info!("🚀 PREDICTED SPAWN OBSERVER FIRED! Entity: {:?}", entity);

    if let Ok((mut color, player_id, name)) = player_query.get_mut(entity) {
        let entity_name = name.map(|n| n.as_str()).unwrap_or("Unknown");
        let is_local_player = player_id.0.to_bits() == local_player_id.0;

        let hsva = bevy::color::Hsva {
            saturation: 0.4,
            ..bevy::color::Hsva::from(color.0)
        };
        color.0 = Color::from(hsva);

        commands.entity(entity).insert((
            PlayerPhysicsBundle::default(),
            LinearVelocity::default(),
            Mesh3d(meshes.add(Capsule3d::new(PLAYER_CAPSULE_RADIUS, PLAYER_CAPSULE_HEIGHT))),
            MeshMaterial3d(materials.add(color.0)),
        ));

        // Only add input map to the local player
        if is_local_player {
            let input_map = InputMap::<PlayerAction>::new([
                (PlayerAction::Jump, KeyCode::Space),
                (PlayerAction::Shoot, KeyCode::Enter),
            ])
            .with_dual_axis(PlayerAction::Move, VirtualDPad::wasd())
            .with_dual_axis(PlayerAction::Move, VirtualDPad::arrow_keys())
            .with_dual_axis(PlayerAction::Look, MouseMove::default());

            commands.entity(entity).insert(input_map);

            info!(
                "✅ ADDED INPUT HANDLING to LOCAL predicted player: {:?} ({})",
                entity, entity_name
            );
        } else {
            info!(
                "📊 Spawned REMOTE predicted player (no input): {:?} ({})",
                entity, entity_name
            );
        }
    } else {
        error!("❌ FAILED TO GET PLAYER COMPONENTS for entity {:?}", entity);
    }
}

fn handle_other_players_spawn(
    trigger: Trigger<OnAdd, (PlayerId, Interpolated)>,
    mut player_query: Query<(&PlayerColor, &PlayerId, Option<&Name>)>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    local_player_id: Res<LocalPlayerId>,
) {
    let entity = trigger.target();

    info!("📊 INTERPOLATED SPAWN OBSERVER FIRED! Entity: {:?}", entity);

    if let Ok((color, player_id, name)) = player_query.get_mut(entity) {
        let entity_name = name.map(|n| n.as_str()).unwrap_or("Unknown");
        let is_local_player = player_id.0.to_bits() == local_player_id.0;
        if !is_local_player {
            commands.entity(entity).insert((
                PlayerPhysicsBundle::default(),
                LinearVelocity::default(),
                Mesh3d(meshes.add(Capsule3d::new(
                    shared::input::PLAYER_CAPSULE_RADIUS,
                    shared::input::PLAYER_CAPSULE_HEIGHT,
                ))),
                MeshMaterial3d(materials.add(color.0)),
            ));
            info!(
                "📊 Spawned interpolated player: {:?} ({})",
                entity, entity_name
            );
        }
    } else {
        error!(
            "❌ FAILED TO GET INTERPOLATED PLAYER COMPONENTS for entity {:?}",
            entity
        );
    }
}

/// Handle inputs for camera rotation (client-only)
fn update_player_rotation_from_inputs(
    mut player_query: Query<(&mut Rotation, &ActionState<PlayerAction>, &PlayerId)>,
    local_player_id: Res<LocalPlayerId>,
) {
    let mut found = false;
    for (mut rotation, action_state, player_id) in player_query.iter_mut() {
        // Only process rotation for the local player
        info!(
            "Found player {:?} ({:?})",
            player_id.0.to_bits(),
            local_player_id.0
        );
        if player_id.0.to_bits() != local_player_id.0 {
            continue;
        }
        found = true;
        let mouse_delta = action_state.axis_pair(&PlayerAction::Look);
        info!("🔄 Updating player rotation from inputs: {:?}", mouse_delta);
        if mouse_delta != Vec2::ZERO {
            // Calculate yaw (horizontal) and pitch (vertical) rotation
            let yaw_delta = -mouse_delta.x * MOUSE_SENSITIVITY;
            let pitch_delta = -mouse_delta.y * MOUSE_SENSITIVITY;

            // Get current rotation as Euler angles
            let (mut yaw, mut pitch, roll) = rotation.0.to_euler(EulerRot::YXZ);

            // Apply deltas
            yaw += yaw_delta;
            pitch += pitch_delta;

            // Clamp pitch to prevent over-rotation
            pitch = pitch.clamp(
                -std::f32::consts::FRAC_PI_2 + 0.1,
                std::f32::consts::FRAC_PI_2 - 0.1,
            );

            // Apply new rotation to player entity
            rotation.0 = Quat::from_euler(EulerRot::YXZ, yaw, pitch, roll);

            info!(
                "🎮 Player rotation: Mouse Delta: {:?}, Yaw: {:.3}, Pitch: {:.3}",
                mouse_delta, yaw, pitch
            );
        }
    }
    if !found {
        debug!("No local player entity with Predicted+Controlled for mouse look");
    }
}

fn client_player_movement(
    time: Res<Time>,
    mut player_query: Query<
        (
            Entity,
            &mut Position,
            &Rotation,
            &mut LinearVelocity,
            &ActionState<PlayerAction>,
            &PlayerId,
        ),
        (With<PlayerId>, With<Predicted>, With<Controlled>),
    >,
    local_player_id: Res<LocalPlayerId>,
) {
    for (entity, mut position, rotation, mut velocity, action_state, player_id) in
        player_query.iter_mut()
    {
        // Only process movement for the local player
        if player_id.0.to_bits() != local_player_id.0 {
            continue;
        }

        let axis_pair = action_state.axis_pair(&PlayerAction::Move);
        if axis_pair != Vec2::ZERO || !action_state.get_pressed().is_empty() {
            info!(
                "🎮 Player input: Entity {:?} (Local Player ID: {}), Move: {:?}",
                entity, local_player_id.0, axis_pair
            );
        }

        // Use shared movement logic with player rotation
        // Note: Player rotation is now handled by the camera system
        shared_player_movement(&time, action_state, &mut position, &rotation, &mut velocity);
    }
}
