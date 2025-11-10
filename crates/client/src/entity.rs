use crate::input::get_player_input_map;
use avian3d::prelude::{LinearVelocity, Position};
use bevy::prelude::{
    Add, App, Assets, Capsule3d, Commands, FixedUpdate, Mesh, Mesh3d, MeshMaterial3d, Name, On,
    Plugin, Query, Res, ResMut, Single, StandardMaterial, Update, With, debug, info,
};
use shared::level::create_static::setup_static_level;

use crate::LocalPlayerId;
use lightyear::prelude::{
    Controlled, Interpolated, LocalTimeline, MessageReceiver, NetworkTimeline, Predicted,
    PredictionManager,
};
use shared::input::{PLAYER_CAPSULE_HEIGHT, PLAYER_CAPSULE_RADIUS};

use shared::protocol::{PlayerColor, PlayerId, StartLoadingGameEvent};

pub struct ClientEntitiesPlugin;

impl Plugin for ClientEntitiesPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(handle_player_spawn);
        app.add_observer(handle_other_players_spawn);
        app.add_systems(Update, handle_static_world);
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
    // Should spawn when exiting lobby after world creation, then we add controls and rendering
    let entity = trigger.entity;
    info!("🎯 CLIENT: Received player entity from server {:?}", entity);

    let Ok((name, color, player_id)) = player_query.get(entity) else {
        info!(
            "❌ CLIENT: Failed to get player data for entity {:?}",
            entity
        );
        return;
    };

    info!(
        "🔍 CLIENT: Entity received: player_id={:?}, local_player_id={}",
        player_id.0, local_player_id.0
    );
    info!("🔍 CLIENT: Entity name: {:?}, color: {:?}", name, color);

    if player_id.0.to_bits() == local_player_id.0 {
        info!(
            "✅ CLIENT: This is our local player! Adding rendering and input to entity {:?} ({:?})",
            entity, name
        );
        // Only add client-side rendering and input components - no physics for now
        commands
            .entity(entity)
            .insert(Mesh3d(meshes.add(Capsule3d::new(
                PLAYER_CAPSULE_RADIUS,
                PLAYER_CAPSULE_HEIGHT,
            ))))
            .insert(MeshMaterial3d(materials.add(color.0)));

        let input_map = get_player_input_map();
        commands.entity(entity).insert(input_map);
        info!(
            "✅ CLIENT: Local player rendering and input setup complete for entity {:?}",
            entity
        );
    } else {
        info!(
            "ℹ️ CLIENT: This is a remote player (id: {:?}), will be handled by interpolation observer",
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
    // Should spawn when exiting lobby after world creation, then we add rendering
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

fn handle_static_world(
    mut receiver: Single<&mut MessageReceiver<StartLoadingGameEvent>>,
    mut commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    materials: Option<ResMut<Assets<StandardMaterial>>>,
) {
    if receiver.has_messages() {
        setup_static_level(commands.reborrow(), meshes, materials, None);
    }
    receiver.receive().for_each(drop);
}
