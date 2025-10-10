use super::entity_implementations::*;
use super::entity_spawner::*;
use crate::enemy::Enemy;
use crate::scene::{FloorMarker, WallMarker};

use avian3d::prelude::Position;
use bevy::{
    color::palettes::css::WHITE,
    light::{AmbientLight, DirectionalLight},
    prelude::{
        Add, Assets, Commands, Entity, Mesh, Mesh3d, Name, On, Query, ResMut, StandardMaterial,
        Transform, Vec3, Without, debug, default,
    },
};

/// Observer function for adding floor visuals using entity system
pub fn add_floor_visuals(
    trigger: On<Add, FloorMarker>,
    floor_query: Query<(Entity, &Position)>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Ok((entity, _position)) = floor_query.get(trigger.entity) else {
        debug!("Failed to get floor entity for visual addition.");
        return;
    };

    let floor_entity = FloorEntity::default();
    add_entity_visuals(
        &floor_entity,
        &mut commands,
        entity,
        &mut meshes,
        &mut materials,
    );
}

/// Observer function for adding wall visuals using entity system
pub fn add_wall_visuals(
    trigger: On<Add, WallMarker>,
    wall_query: Query<(Entity, &Position, &Name), Without<Mesh3d>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Ok((entity, _position, name)) = wall_query.get(trigger.entity) else {
        debug!("Failed to get wall entity for visual addition.");
        return;
    };

    // Determine wall type from name
    let wall_type = if name.as_str().contains("North") {
        WallType::North
    } else if name.as_str().contains("South") {
        WallType::South
    } else if name.as_str().contains("East") {
        WallType::East
    } else {
        WallType::West
    };

    let wall_entity = WallEntity::new(wall_type);
    add_entity_visuals(
        &wall_entity,
        &mut commands,
        entity,
        &mut meshes,
        &mut materials,
    );
}

/// Observer function for adding enemy visuals using entity system
pub fn add_enemy_visuals(
    trigger: On<Add, Enemy>,
    enemy_query: Query<(Entity, &Position, &Name, &Enemy), Without<Mesh3d>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Ok((entity, _position, name, enemy_data)) = enemy_query.get(trigger.entity) else {
        debug!("Failed to get enemy entity for visual addition.");
        return;
    };

    // Determine enemy type from health/stats or name
    let enemy_type = if enemy_data.max_health > 120.0 {
        EnemyType::Heavy
    } else if enemy_data.move_speed > 4.0 {
        EnemyType::Fast
    } else {
        EnemyType::Basic
    };

    let enemy_entity = EnemyEntity::new(enemy_type);
    add_entity_visuals(
        &enemy_entity,
        &mut commands,
        entity,
        &mut meshes,
        &mut materials,
    );

    debug!("Added visuals for enemy: {}", name.as_str());
}

/// Setup basic lighting for the scene
pub fn setup_lighting(mut commands: Commands) {
    // Add ambient lighting for better visibility
    commands.insert_resource(AmbientLight {
        color: WHITE.into(),
        brightness: 0.3,
        affects_lightmapped_meshes: true,
    });

    // Main directional light (sun)
    commands.spawn((
        DirectionalLight {
            color: WHITE.into(),
            illuminance: 10000.0,
            ..default()
        },
        Transform::from_xyz(10.0, 10.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
        Name::new("Sun"),
    ));

    debug!("✅ Lighting setup complete with ambient and directional light");
}
