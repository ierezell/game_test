use crate::inputs::input::{PLAYER_CAPSULE_HEIGHT, PLAYER_CAPSULE_RADIUS};
use crate::protocol::{CharacterMarker, PlayerColor, PlayerId};
use avian3d::prelude::Position;
use bevy::prelude::{
    Add, Assets, Capsule3d, Commands, Entity, Mesh, Mesh3d, MeshMaterial3d, On, Query, ResMut,
    Sphere, StandardMaterial, With, Without, debug, default,
};

pub fn add_player_visuals(
    trigger: On<Add, PlayerId>,
    player_query: Query<(Entity, &Position, &PlayerColor), Without<Mesh3d>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    let Ok((entity, position, color)) = player_query.get(trigger.entity) else {
        debug!("Failed to get player entity for visual addition.");
        return;
    };

    commands.entity(entity).insert((
        Mesh3d(meshes.add(Capsule3d::new(PLAYER_CAPSULE_RADIUS, PLAYER_CAPSULE_HEIGHT))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: color.0,
            unlit: false, // PBR lighting - only visible when lit
            ..default()
        })),
    ));
    debug!("Added player visuals at position: {:?}", position.0);
}

pub fn add_npc_visuals(
    trigger: On<Add, CharacterMarker>,
    npc_query: Query<(Entity, &Position), (With<CharacterMarker>, Without<PlayerId>, Without<Mesh3d>)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    let Ok((entity, position)) = npc_query.get(trigger.entity) else {
        return;
    };

    commands.entity(entity).insert((
        Mesh3d(meshes.add(Sphere::new(0.8))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: bevy::color::palettes::tailwind::RED_500.into(),
            unlit: false,
            ..default()
        })),
    ));

    debug!("Added NPC visuals at position: {:?}", position.0);
}
