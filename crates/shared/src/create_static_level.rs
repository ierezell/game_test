use crate::navigation::NavigationObstacle;
use avian3d::prelude::{Collider, RigidBody};
use bevy::prelude::Color;
use bevy::prelude::{
    AmbientLight, Assets, Commands, Component, Cuboid, Dir3, DirectionalLight, Mesh, Mesh3d,
    MeshMaterial3d, Name, Plane3d, Quat, ResMut, StandardMaterial, Transform, Vec2, Vec3, default,
    info,
};
use rand::SeedableRng;
use rand::rngs::StdRng;
use serde::{Deserialize, Serialize};
use vleue_navigator::prelude::*;

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

    // Floor with physics collision
    let mut floor_entity = commands.spawn((
        Name::new("Floor"),
        Transform::from_xyz(0.0, -FLOOR_THICKNESS / 2.0, 0.0),
        Mesh3d(meshes.add(Plane3d {
            normal: Dir3::Y,
            half_size: Vec2::splat(ROOM_SIZE),
        })),
        RigidBody::Static,
        Collider::cuboid(ROOM_SIZE * 2.0, FLOOR_THICKNESS, ROOM_SIZE * 2.0),
    ));

    if let Some(ref mut mats) = materials {
        floor_entity.insert(MeshMaterial3d(mats.add(StandardMaterial { ..default() })));
    }

    // Walls - could be procedurally varied based on seed
    let wall_positions = [
        (
            Vec3::new(ROOM_SIZE + WALL_THICKNESS, WALL_HEIGHT, 0.0),
            "Wall East",
        ),
        (
            Vec3::new(-ROOM_SIZE - WALL_THICKNESS, WALL_HEIGHT, 0.0),
            "Wall West",
        ),
        (
            Vec3::new(0.0, WALL_HEIGHT, ROOM_SIZE + WALL_THICKNESS),
            "Wall North",
        ),
        (
            Vec3::new(0.0, WALL_HEIGHT, -ROOM_SIZE - WALL_THICKNESS),
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
            Transform::from_translation(position / 2.0),
            Mesh3d(meshes.add(Cuboid {
                half_size: size / 2.0,
            })),
            RigidBody::Static,
            Collider::cuboid(size.x, size.y, size.z),
            NavigationObstacle, // Mark walls as navigation obstacles
        ));

        if let Some(ref mut mats) = materials {
            wall_entity.insert(MeshMaterial3d(mats.add(StandardMaterial { ..default() })));
        }
    }

    // Create some interior obstacles for more interesting pathfinding
    let obstacle_positions = [
        Vec3::new(15.0, 2.5, 10.0),
        Vec3::new(-10.0, 2.5, -15.0),
        Vec3::new(20.0, 2.5, -20.0),
        Vec3::new(-15.0, 2.5, 15.0),
    ];

    for (i, pos) in obstacle_positions.iter().enumerate() {
        let mut obstacle_entity = commands.spawn((
            Name::new(format!("Obstacle_{}", i + 1)),
            Transform::from_translation(*pos),
            Mesh3d(meshes.add(Cuboid::new(4.0, 5.0, 4.0))),
            RigidBody::Static,
            Collider::cuboid(2.0, 2.5, 2.0),
            NavigationObstacle, // Mark as navigation obstacle
        ));

        if let Some(ref mut mats) = materials {
            obstacle_entity.insert(MeshMaterial3d(mats.add(StandardMaterial {
                base_color: Color::srgb(0.7, 0.4, 0.2),
                ..default()
            })));
        }
    }

    // Setup navigation mesh for pathfinding
    // The navmesh covers the floor area minus the walls
    let nav_area = ROOM_SIZE - 1.0; // Leave some margin from walls
    commands.spawn((
        NavMeshSettings {
            // Define the outer borders of the navmesh (floor area)
            fixed: Triangulation::from_outer_edges(&[
                Vec2::new(-nav_area, -nav_area),
                Vec2::new(nav_area, -nav_area),
                Vec2::new(nav_area, nav_area),
                Vec2::new(-nav_area, nav_area),
            ]),
            simplify: 0.1,
            merge_steps: 1,
            build_timeout: Some(5.0), // Allow more time for complex levels
            agent_radius: 0.5,        // Match typical player/bot radius
            ..default()
        },
        // Position the navmesh slightly above the floor
        Transform::from_xyz(0.0, 0.1, 0.0)
            .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
        // Auto-update navmesh when obstacles change
        NavMeshUpdateMode::Direct,
        Name::new("NavMesh"),
    ));

    commands.spawn((LevelDoneMarker, Name::new("Level")));
    info!("Scene setup complete with navmesh and seed: {}", seed);
}
