use crate::input::get_player_input_map;
use avian3d::prelude::{LinearVelocity, Position};
use bevy::prelude::{
    Add, App, Assets, Capsule3d, Commands, FixedUpdate, Mesh, Mesh3d, MeshMaterial3d, Name, On,
    Plugin, Query, Res, ResMut, Single, StandardMaterial, With, debug, info,
};
// Removed unused window imports
// Removed unused input manager imports

use crate::app::LocalPlayerId;
use lightyear::prelude::{
    Controlled, Interpolated, LocalTimeline, NetworkTimeline, Predicted, PredictionManager,
};
use shared::input::{PLAYER_CAPSULE_HEIGHT, PLAYER_CAPSULE_RADIUS};
use shared::protocol::{PlayerColor, PlayerId};
use shared::scene::{FloorPhysicsBundle, WallPhysicsBundle};

pub struct ClientRenderPlugin;

impl Plugin for ClientRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(handle_player_spawn);
        app.add_observer(handle_other_players_spawn);
        app.add_systems(FixedUpdate, debug_player_position);
        app.add_observer(WallPhysicsBundle::observer);
        app.add_observer(FloorPhysicsBundle::observer);
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
    let Ok((name, color, player_id)) = player_query.get(entity) else {
        debug!("Failed to get player data for entity {:?}", entity);
        return;
    };

    if player_id.0.to_bits() == local_player_id.0 {
        info!(
            "🚀 Attaching mesh, physics, and input map to PREDICTED player: {:?} ({:?})",
            entity, name
        );
        commands
            .entity(entity)
            .insert(Mesh3d(meshes.add(Capsule3d::new(
                PLAYER_CAPSULE_RADIUS,
                PLAYER_CAPSULE_HEIGHT,
            ))))
            .insert(MeshMaterial3d(materials.add(color.0)))
            .insert(shared::scene::PlayerPhysicsBundle::default());

        let input_map = get_player_input_map();
        commands.entity(entity).insert(input_map);
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
    let Ok((name, color)) = player_query.get(entity) else {
        debug!(
            "Failed to get interpolated player data for entity {:?}",
            entity
        );
        return;
    };

    commands.entity(entity).insert((
        Mesh3d(meshes.add(Capsule3d::new(PLAYER_CAPSULE_RADIUS, PLAYER_CAPSULE_HEIGHT))),
        MeshMaterial3d(materials.add(color.0)),
    ));
    info!(
        "🚀 INTERPOLATED SPAWN! Entity: {:?} Player: {:?}",
        entity, name
    );
}
