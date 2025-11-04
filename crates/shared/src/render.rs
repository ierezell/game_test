use crate::entities::enemy::Enemy;
use avian3d::prelude::Position;

use crate::input::{PLAYER_CAPSULE_HEIGHT, PLAYER_CAPSULE_RADIUS};
use crate::protocol::{PlayerColor, PlayerId};
use bevy::prelude::{
    Add, Assets, Capsule3d, Commands, Entity, Mesh, Mesh3d, MeshMaterial3d, Name, On, Query,
    ResMut, StandardMaterial, Without, debug, default,
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
            ..default()
        })),
    ));
    debug!("Added player visuals at position: {:?}", position.0);
}

/// Observer function for adding enemy visuals using entity system
pub fn add_enemy_visuals(
    trigger: On<Add, Enemy>,
    enemy_query: Query<(Entity, &Position, &Name, &Enemy), Without<Mesh3d>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Ok((_entity, _position, name, _enemy_data)) = enemy_query.get(trigger.entity) else {
        debug!("Failed to get enemy entity for visual addition.");
        return;
    };

    commands.spawn((
        Mesh3d(meshes.add(Capsule3d::new(PLAYER_CAPSULE_RADIUS, PLAYER_CAPSULE_HEIGHT))),
        MeshMaterial3d(materials.add(StandardMaterial { ..default() })),
    ));

    debug!("Added visuals for enemy: {}", name.as_str());
}
