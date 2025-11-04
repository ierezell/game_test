use crate::input::get_player_input_map;
use avian3d::prelude::{LinearVelocity, Position};
use bevy::prelude::{
    Add, App, Assets, Capsule3d, Commands, FixedUpdate, Mesh, Mesh3d, MeshMaterial3d, Name, On,
    Plugin, Query, Res, ResMut, Single, StandardMaterial, With, debug, info,
};

use crate::LocalPlayerId;
use lightyear::prelude::{
    Controlled, Interpolated, LocalTimeline, NetworkTimeline, Predicted, PredictionManager,
};
use shared::input::{PLAYER_CAPSULE_HEIGHT, PLAYER_CAPSULE_RADIUS};
use shared::protocol::{PlayerColor, PlayerId};

pub struct ClientRenderPlugin;

impl Plugin for ClientRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(handle_player_spawn);
        app.add_observer(handle_other_players_spawn);
        app.add_systems(FixedUpdate, debug_player_position);
    }
}

fn debug_player_position(
    player_query: Query<
        (&Name, &Position, &LinearVelocity),
        (With<PlayerId>, With<Predicted>, With<Controlled>),
    >,
    timeline: Single<&LocalTimeline, With<PredictionManager>>,
) {
    for (name, position, linear_velocity) in player_query.iter() {
        debug!(
            "C:{:?} pos:{:?} vel:{:?} tick:{:?}",
            name,
            position,
            linear_velocity,
            timeline.tick()
        );
    }
}

fn handle_player_spawn(
    trigger: On<Add, (Predicted, Controlled, PlayerId)>,
    player_query: Query<
        (&Name, &PlayerColor, &PlayerId),
        (With<Predicted>, With<Controlled>, With<PlayerId>),
    >,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    local_player_id: Res<LocalPlayerId>,
) {
    let entity = trigger.entity;
    info!(
        "🎯 CLIENT: handle_player_spawn triggered for entity {:?}",
        entity
    );

    let Ok((name, color, player_id)) = player_query.get(entity) else {
        info!(
            "❌ CLIENT: Failed to get player data for entity {:?}",
            entity
        );
        return;
    };

    let local_peer_id = lightyear::prelude::PeerId::Netcode(local_player_id.0);
    info!(
        "🔍 CLIENT: Entity spawn check: player_id={:?}, local_peer_id={:?}, local_player_id={}",
        player_id.0, local_peer_id, local_player_id.0
    );
    info!("🔍 CLIENT: Entity name: {:?}, color: {:?}", name, color);

    if player_id.0 == local_peer_id {
        info!(
            "✅ CLIENT: This is our local player! Attaching mesh, physics, and input map to entity {:?} ({:?})",
            entity, name
        );
        commands
            .entity(entity)
            .insert(Mesh3d(meshes.add(Capsule3d::new(
                PLAYER_CAPSULE_RADIUS,
                PLAYER_CAPSULE_HEIGHT,
            ))))
            .insert(MeshMaterial3d(materials.add(color.0)))
            .insert(shared::entities::player::PlayerPhysicsBundle::default());

        let input_map = get_player_input_map();
        commands.entity(entity).insert(input_map);
        info!(
            "✅ CLIENT: Local player setup complete for entity {:?}",
            entity
        );
    } else {
        info!(
            "ℹ️ CLIENT: This is a remote player (id: {:?}), skipping local player setup",
            player_id.0
        );
    }
}

fn handle_other_players_spawn(
    trigger: On<Add, (PlayerId, Interpolated)>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    player_query: Query<(&Name, &PlayerColor), With<Interpolated>>,
) {
    let entity = trigger.entity;
    info!(
        "🌐 CLIENT: handle_other_players_spawn triggered for entity {:?}",
        entity
    );

    let Ok((name, color)) = player_query.get(entity) else {
        info!(
            "❌ CLIENT: Failed to get interpolated player data for entity {:?}",
            entity
        );
        return;
    };

    commands.entity(entity).insert((
        Mesh3d(meshes.add(Capsule3d::new(PLAYER_CAPSULE_RADIUS, PLAYER_CAPSULE_HEIGHT))),
        MeshMaterial3d(materials.add(color.0)),
    ));
    info!(
        "✅ CLIENT: INTERPOLATED player setup complete! Entity: {:?} Player: {:?}",
        entity, name
    );
}
