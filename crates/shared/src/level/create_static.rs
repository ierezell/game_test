use avian3d::prelude::{Position, Rotation};
use bevy::prelude::Color;
use bevy::prelude::{
    AmbientLight, Assets, Commands, Component, Cuboid, Dir3, DirectionalLight, Mesh, Mesh3d,
    MeshMaterial3d, Name, Plane3d, ResMut, StandardMaterial, Transform, Vec2, Vec3, default, info,
};

use rand::SeedableRng;
use rand::rngs::StdRng;
use serde::{Deserialize, Serialize};

pub const FLOOR_THICKNESS: f32 = 1.0;
pub const WALL_THICKNESS: f32 = 1.0;
pub const WALL_HEIGHT: f32 = 10.0;
pub const ROOM_SIZE: f32 = 50.0;

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct LevelDoneMarker;

pub fn setup_static_level(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: Option<ResMut<Assets<StandardMaterial>>>,
    seed: Option<u64>,
) {
    let seed = seed.unwrap_or(42); // Default seed if none provided
    info!("Setting up static level with seed: {}", seed);

    // Use seed for procedural generation
    let mut _rng = StdRng::seed_from_u64(seed);
    if materials.is_some() {
        commands.insert_resource(AmbientLight {
            color: Color::WHITE.into(),
            brightness: 0.3,
            affects_lightmapped_meshes: true,
        });

        commands.spawn((
            DirectionalLight {
                color: Color::WHITE.into(),
                illuminance: 10000.0,
                ..default()
            },
            Transform::from_xyz(10.0, 10.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
            Name::new("Sun"),
        ));
    }

    // Floor
    let mut floor_entity = commands.spawn((
        Name::new("Floor"),
        Position(Vec3::new(0.0, -FLOOR_THICKNESS / 2.0, 0.0)),
        Mesh3d(meshes.add(Plane3d {
            normal: Dir3::Y,
            half_size: Vec2::splat(50.0),
        })),
        Rotation::default(),
    ));

    if let Some(ref mut materials) = materials {
        floor_entity.insert(MeshMaterial3d(
            materials.add(StandardMaterial { ..default() }),
        ));
    }

    // Walls - could be procedurally varied based on seed
    let wall_positions = [
        (
            Vec3::new(
                ROOM_SIZE / 2.0 + WALL_THICKNESS / 2.0,
                WALL_HEIGHT / 2.0,
                0.0,
            ),
            "Wall East",
        ),
        (
            Vec3::new(
                -ROOM_SIZE / 2.0 - WALL_THICKNESS / 2.0,
                WALL_HEIGHT / 2.0,
                0.0,
            ),
            "Wall West",
        ),
        (
            Vec3::new(
                0.0,
                WALL_HEIGHT / 2.0,
                ROOM_SIZE / 2.0 + WALL_THICKNESS / 2.0,
            ),
            "Wall North",
        ),
        (
            Vec3::new(
                0.0,
                WALL_HEIGHT / 2.0,
                -ROOM_SIZE / 2.0 - WALL_THICKNESS / 2.0,
            ),
            "Wall South",
        ),
    ];

    for (position, name) in wall_positions {
        let size = if name.contains("North") || name.contains("South") {
            Vec3::new(ROOM_SIZE, WALL_HEIGHT, WALL_THICKNESS)
        } else {
            Vec3::new(WALL_THICKNESS, WALL_HEIGHT, ROOM_SIZE)
        };
        let mut wall_entity = commands.spawn((
            Name::new(name),
            Position(position),
            Rotation::default(),
            Mesh3d(meshes.add(Cuboid { half_size: size })),
        ));

        if let Some(ref mut materials) = materials {
            wall_entity.insert(MeshMaterial3d(
                materials.add(StandardMaterial { ..default() }),
            ));
        }
    }

    info!("Scene setup complete with seed: {}", seed);
    commands.spawn((LevelDoneMarker, Name::new("Level")));
}
