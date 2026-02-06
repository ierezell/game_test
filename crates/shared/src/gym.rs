use crate::GymMode;
use crate::components::health::Health;
use crate::entities::NpcPhysicsBundle;
use crate::navigation::{NavigationObstacle, setup_patrol, validate_spawn_position};
use crate::protocol::CharacterMarker;
use avian3d::prelude::{Collider, LinearVelocity, Position, RigidBody, Rotation};
use bevy::prelude::Color;
use bevy::prelude::{
    Assets, Commands, Component, Cuboid, Dir3, Mesh, Mesh3d, MeshMaterial3d, Name, Plane3d, Query,
    Res, ResMut, StandardMaterial, Vec2, Vec3, With, default, info,
};

use lightyear::prelude::{InterpolationTarget, NetworkTarget, Replicate};
use serde::{Deserialize, Serialize};
use vleue_navigator::prelude::*;

pub const FLOOR_THICKNESS: f32 = 1.0;
pub const WALL_THICKNESS: f32 = 1.0;
pub const WALL_HEIGHT: f32 = 10.0;
pub const ROOM_SIZE: f32 = 50.0;

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct LevelDoneMarker;

pub fn setup_gym_level(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: Option<ResMut<Assets<StandardMaterial>>>, // Option for tests as there is no render
) {
    let mut floor_entity = commands.spawn((
        Name::new("Floor"),
        Position::from(Vec3::new(0.0, -FLOOR_THICKNESS / 2.0, 0.0)),
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

    let mut ceiling_entity = commands.spawn((
        Name::new("Ceiling"),
        Position::from(Vec3::new(0.0, WALL_HEIGHT + FLOOR_THICKNESS / 2.0, 0.0)),
        Mesh3d(meshes.add(Plane3d {
            normal: Dir3::NEG_Y,
            half_size: Vec2::splat(ROOM_SIZE),
        })),
        RigidBody::Static,
        Collider::cuboid(ROOM_SIZE * 2.0, FLOOR_THICKNESS, ROOM_SIZE * 2.0),
    ));

    if let Some(ref mut mats) = materials {
        ceiling_entity.insert(MeshMaterial3d(mats.add(StandardMaterial { ..default() })));
    }

    let walls = [
        (
            Vec3::new(ROOM_SIZE / 2.0, WALL_HEIGHT / 2.0, 0.0),
            Vec3::new(WALL_THICKNESS, WALL_HEIGHT, ROOM_SIZE),
            "Wall East",
        ),
        (
            Vec3::new(-ROOM_SIZE / 2.0, WALL_HEIGHT / 2.0, 0.0),
            Vec3::new(WALL_THICKNESS, WALL_HEIGHT, ROOM_SIZE),
            "Wall West",
        ),
        (
            Vec3::new(0.0, WALL_HEIGHT / 2.0, ROOM_SIZE / 2.0),
            Vec3::new(ROOM_SIZE, WALL_HEIGHT, WALL_THICKNESS),
            "Wall North",
        ),
        (
            Vec3::new(0.0, WALL_HEIGHT / 2.0, -ROOM_SIZE / 2.0),
            Vec3::new(ROOM_SIZE, WALL_HEIGHT, WALL_THICKNESS),
            "Wall South",
        ),
    ];

    for (position, size, name) in walls {
        let mut wall_entity = commands.spawn((
            Name::new(name),
            Position::from(position),
            Mesh3d(meshes.add(Cuboid {
                half_size: size / 2.0,
            })),
            RigidBody::Static,
            Collider::cuboid(size.x, size.y, size.z),
            NavigationObstacle,
        ));

        if let Some(ref mut mats) = materials {
            wall_entity.insert(MeshMaterial3d(mats.add(StandardMaterial { ..default() })));
        }
    }

    let obstacle_positions = [
        Vec3::new(15.0, 1.5, 10.0),
        Vec3::new(-10.0, 1.5, -15.0),
        Vec3::new(20.0, 1.5, -20.0),
        Vec3::new(-15.0, 1.5, 15.0),
    ];

    for (i, pos) in obstacle_positions.iter().enumerate() {
        let mut obstacle_entity = commands.spawn((
            Name::new(format!("Obstacle_{}", i + 1)),
            Position::from(*pos),
            Mesh3d(meshes.add(Cuboid::new(3.0, 3.0, 3.0))),
            RigidBody::Static,
            Collider::cuboid(1.5, 1.5, 1.5),
            NavigationObstacle,
        ));

        if let Some(ref mut mats) = materials {
            obstacle_entity.insert(MeshMaterial3d(mats.add(StandardMaterial {
                base_color: Color::srgb(0.7, 0.4, 0.2),
                ..default()
            })));
        }
    }

    let nav_area = ROOM_SIZE - 2.0; // Leave more margin from walls for safety
    commands.spawn((
        ManagedNavMesh::single(),
        NavMeshSettings {
            fixed: Triangulation::from_outer_edges(&[
                Vec2::new(-nav_area, -nav_area),
                Vec2::new(nav_area, -nav_area),
                Vec2::new(nav_area, nav_area),
                Vec2::new(-nav_area, nav_area),
            ]),
            simplify: 0.1,
            merge_steps: 1,
            build_timeout: Some(10.0),
            agent_radius: 1.0,
            ..default()
        },
        NavMeshUpdateMode::Direct,
        Name::new("NavMesh"),
    ));

    commands.spawn((LevelDoneMarker, Name::new("Gym")));
}

pub fn spawn_gym_patrolling_npc_entities(
    mut commands: Commands,
    obstacles: Query<&Position, With<NavigationObstacle>>,
    gym_mode: Option<Res<GymMode>>,
) {
    let is_gym_mode = gym_mode.map(|gm| gm.0).unwrap_or(false);

    if !is_gym_mode {
        info!("⏭️  Skipping NPC spawn in normal mode (not implemented yet)");
        return;
    }

    info!("🤖 Spawning test NPC for gym mode");

    let initial_spawn = Vec3::new(-18.0, 1.0, -8.0);
    // Obstacles are created at create_static_level, so it's before this system runs
    let validated_spawn = validate_spawn_position(initial_spawn, &obstacles, 0.5);
    let enemy = commands
        .spawn((
            Name::new("Patrol_Enemy_1"),
            Position::new(validated_spawn),
            Rotation::default(),
            LinearVelocity::default(),
            Health::basic(),
            Replicate::to_clients(NetworkTarget::All),
            InterpolationTarget::to_clients(NetworkTarget::All),
            CharacterMarker,
            NpcPhysicsBundle::default(),
        ))
        .id();

    let original_patrol_points = [
        Vec3::new(-20.0, 1.0, -10.0),
        Vec3::new(-5.0, 1.0, -10.0),
        Vec3::new(-5.0, 1.0, 5.0),
        Vec3::new(-20.0, 1.0, 5.0),
    ];

    let validated_patrol_points: Vec<Vec3> = original_patrol_points
        .iter()
        .map(|&point| validate_spawn_position(point, &obstacles, 0.5))
        .collect();

    setup_patrol(&mut commands, enemy, validated_patrol_points, 3.0);
}
