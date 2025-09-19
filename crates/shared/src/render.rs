use avian3d::prelude::Position;
use bevy::{
    color::palettes::css::{GREEN, WHITE},
    prelude::{
        Added, Assets, Capsule3d, Commands, Cuboid, DirectionalLight, Entity, Mesh, Mesh3d,
        MeshMaterial3d, Name, Query, ResMut, StandardMaterial, Transform, Vec3, Without, debug,
        default,
    },
};

use crate::input::{PLAYER_CAPSULE_HEIGHT, PLAYER_CAPSULE_RADIUS};
use crate::protocol::PlayerId;
use crate::scene::{
    FLOOR_THICKNESS, FloorMarker, ROOM_SIZE, WALL_HEIGHT, WALL_THICKNESS, WallMarker,
};

pub fn add_floor_visuals(
    mut commands: Commands,
    floor_query: Query<(Entity, &Position), Added<FloorMarker>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, position) in &floor_query {
        commands.entity(entity).insert((
            Mesh3d(meshes.add(Cuboid::new(
                ROOM_SIZE * 2.0,
                FLOOR_THICKNESS,
                ROOM_SIZE * 2.0,
            ))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: GREEN.into(), // Changed from GRAY to GREEN for visibility
                ..default()
            })),
        ));
        debug!("Added floor visuals at position: {:?}", position.0);
    }
}

pub fn setup_lighting(mut commands: Commands) {
    // Add ambient lighting for better visibility
    commands.insert_resource(bevy::pbr::AmbientLight {
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

pub fn add_wall_visuals(
    mut commands: Commands,
    wall_query: Query<(Entity, &Position, &Name), (Added<WallMarker>, Without<Mesh3d>)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, position, name) in &wall_query {
        let (width, height, depth) =
            if name.as_str().contains("North") || name.as_str().contains("South") {
                (ROOM_SIZE, WALL_HEIGHT, WALL_THICKNESS)
            } else {
                (WALL_THICKNESS, WALL_HEIGHT, ROOM_SIZE)
            };

        commands.entity(entity).insert((
            Mesh3d(meshes.add(Cuboid::new(width, height, depth))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: WHITE.into(),
                ..default()
            })),
        ));
        debug!(
            "Added wall visuals for {} at position: {:?}",
            name.as_str(),
            position.0
        );
    }
}

pub fn add_player_visuals(
    mut commands: Commands,
    player_query: Query<(Entity, &Position), (Added<PlayerId>, Without<Mesh3d>)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, position) in &player_query {
        commands.entity(entity).insert((
            Mesh3d(meshes.add(Capsule3d::new(PLAYER_CAPSULE_RADIUS, PLAYER_CAPSULE_HEIGHT))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: GREEN.into(),
                ..default()
            })),
        ));
        debug!("Added player visuals at position: {:?}", position.0);
    }
}
