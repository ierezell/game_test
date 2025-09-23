use avian3d::prelude::Position;
use bevy::{
    color::palettes::css::{GREEN, WHITE},
    prelude::{
        Assets, Commands, Cuboid, DirectionalLight, Entity, Mesh, Mesh3d, MeshMaterial3d, Name,
        OnAdd, Query, ResMut, StandardMaterial, Transform, Trigger, Vec3, Without, debug, default,
    },
};

use crate::scene::{
    FLOOR_THICKNESS, FloorMarker, ROOM_SIZE, WALL_HEIGHT, WALL_THICKNESS, WallMarker,
};

pub fn add_floor_visuals(
    trigger: Trigger<OnAdd, FloorMarker>,
    floor_query: Query<(Entity, &Position)>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Ok((entity, position)) = floor_query.get(trigger.target()) else {
        debug!("Failed to get floor entity for visual addition.");
        return;
    };
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
    trigger: Trigger<OnAdd, WallMarker>,
    wall_query: Query<(Entity, &Position, &Name), Without<Mesh3d>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Ok((entity, position, name)) = wall_query.get(trigger.target()) else {
        debug!("Failed to get wall entity for visual addition.");
        return;
    };
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
