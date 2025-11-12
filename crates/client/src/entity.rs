use crate::input::get_player_input_map;
use avian3d::prelude::{LinearVelocity, Position};
use bevy::prelude::{
    Add, App, Assets, Capsule3d, Color, Commands, FixedUpdate, Mesh, Mesh3d, MeshMaterial3d, Name,
    On, Plugin, Query, Res, ResMut, Single, StandardMaterial, With, Without, debug, default, info,
};
use leafwing_input_manager::prelude::ActionState;
use shared::input::PlayerAction;

use crate::LocalPlayerId;
use lightyear::prelude::{
    Controlled, Interpolated, LocalTimeline, NetworkTimeline, Predicted, PredictionManager,
};
use shared::input::{PLAYER_CAPSULE_HEIGHT, PLAYER_CAPSULE_RADIUS};

use shared::protocol::{PlayerColor, PlayerId};

pub struct ClientEntitiesPlugin;

impl Plugin for ClientEntitiesPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(handle_player_spawn);
        app.add_observer(handle_other_players_spawn);
        app.add_observer(handle_npc_spawn);
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
    trigger: On<Add, Controlled>,
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
        // Add rendering
        commands
            .entity(entity)
            .insert(Mesh3d(meshes.add(Capsule3d::new(
                PLAYER_CAPSULE_RADIUS,
                PLAYER_CAPSULE_HEIGHT,
            ))))
            .insert(MeshMaterial3d(materials.add(color.0)));

        // Add input components
        let input_map = get_player_input_map();
        commands
            .entity(entity)
            .insert((input_map, ActionState::<PlayerAction>::default()));

        // Add physics components
        commands
            .entity(entity)
            .insert(shared::entities::PhysicsBundle::default());
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
    trigger: On<Add, Interpolated>,
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

fn handle_npc_spawn(
    trigger: On<Add, Interpolated>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    npc_query: Query<&Name, (With<Interpolated>, Without<PlayerId>)>,
) {
    let entity = trigger.entity;

    // Check if this entity has a name but no PlayerId (making it an NPC)
    let Ok(name) = npc_query.get(entity) else {
        return; // Not an NPC or no name component
    };

    info!(
        "🤖 CLIENT: Spawning NPC visual for entity {:?} ({})",
        entity,
        name.as_str()
    );

    // Determine NPC type and color based on name
    let (color, height) = if name.as_str().contains("Enemy") {
        (Color::srgb(0.8, 0.2, 0.2), 2.2) // Red enemies, taller
    } else if name.as_str().contains("Guard") {
        (Color::srgb(0.9, 0.4, 0.1), 2.0) // Orange guards
    } else if name.as_str().contains("Bot") {
        (Color::srgb(0.2, 0.2, 0.8), 1.8) // Blue bots
    } else if name.as_str().contains("Scout") {
        (Color::srgb(0.1, 0.7, 0.3), 1.9) // Green scouts
    } else {
        (Color::srgb(0.5, 0.5, 0.5), 2.0) // Default gray
    };

    commands.entity(entity).insert((
        Mesh3d(meshes.add(Capsule3d::new(PLAYER_CAPSULE_RADIUS * 0.8, height))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: color,
            emissive: color.to_linear() * 0.1, // Slight glow
            ..default()
        })),
    ));

    info!(
        "✅ CLIENT: NPC visual setup complete for entity {:?} ({})",
        entity,
        name.as_str()
    );
}
