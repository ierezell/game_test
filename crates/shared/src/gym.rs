use crate::GymMode;
use crate::components::health::{Health, Respawnable};
use crate::debug::{GymWanderDiagnostics, gym_debug_info, gym_debug_warn};
use crate::entities::NpcPhysicsBundle;
use crate::navigation::{
    NavigationObstacle, NavigationPathState, SimpleNavigationAgent, validate_spawn_position,
};
use crate::protocol::{CharacterMarker, PlayerId};
use avian3d::prelude::{Collider, LinearVelocity, Position, RigidBody, Rotation};
use bevy::prelude::Color;
use bevy::prelude::{
    Assets, Commands, Component, Cuboid, Dir3, Mesh, Mesh3d, MeshMaterial3d, Name, Plane3d, Query,
    Ref, Res, ResMut, StandardMaterial, Vec2, Vec3, With, Without, default,
};
use rand::Rng;
use std::ops::Deref;

use lightyear::prelude::{InterpolationTarget, NetworkTarget, Replicate};
use serde::{Deserialize, Serialize};
use vleue_navigator::prelude::*;

pub const FLOOR_THICKNESS: f32 = 1.0;
pub const WALL_THICKNESS: f32 = 1.0;
pub const WALL_HEIGHT: f32 = 10.0;
pub const ROOM_SIZE: f32 = 50.0;
pub const ROOM_HALF_EXTENT: f32 = ROOM_SIZE * 0.5;
pub const OBSTACLE_SIZE: f32 = 3.0;
pub const OBSTACLE_HALF_EXTENT: f32 = OBSTACLE_SIZE * 0.5;
const GYM_TARGET_MARGIN: f32 = 3.0;
const GYM_MIN_TARGET_DISTANCE: f32 = 6.0;
const GYM_TARGET_SAMPLE_ATTEMPTS: usize = 32;

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct LevelDoneMarker;

#[derive(Component, Clone, Debug, Default)]
pub struct GymRandomWanderer;

fn random_gym_floor_point(rng: &mut impl rand::Rng) -> Vec3 {
    let sample_extent = ROOM_HALF_EXTENT - GYM_TARGET_MARGIN;
    Vec3::new(
        rng.random_range(-sample_extent..sample_extent),
        1.0,
        rng.random_range(-sample_extent..sample_extent),
    )
}

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
            Vec3::new(ROOM_HALF_EXTENT, WALL_HEIGHT / 2.0, 0.0),
            Vec3::new(WALL_THICKNESS, WALL_HEIGHT, ROOM_SIZE),
            "Wall East",
        ),
        (
            Vec3::new(-ROOM_HALF_EXTENT, WALL_HEIGHT / 2.0, 0.0),
            Vec3::new(WALL_THICKNESS, WALL_HEIGHT, ROOM_SIZE),
            "Wall West",
        ),
        (
            Vec3::new(0.0, WALL_HEIGHT / 2.0, ROOM_HALF_EXTENT),
            Vec3::new(ROOM_SIZE, WALL_HEIGHT, WALL_THICKNESS),
            "Wall North",
        ),
        (
            Vec3::new(0.0, WALL_HEIGHT / 2.0, -ROOM_HALF_EXTENT),
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
            Mesh3d(meshes.add(Cuboid::new(OBSTACLE_SIZE, OBSTACLE_SIZE, OBSTACLE_SIZE))),
            RigidBody::Static,
            Collider::cuboid(
                OBSTACLE_HALF_EXTENT,
                OBSTACLE_HALF_EXTENT,
                OBSTACLE_HALF_EXTENT,
            ),
            NavigationObstacle,
        ));

        if let Some(ref mut mats) = materials {
            obstacle_entity.insert(MeshMaterial3d(mats.add(StandardMaterial {
                base_color: Color::srgb(0.7, 0.4, 0.2),
                ..default()
            })));
        }
    }

    let nav_area = ROOM_HALF_EXTENT - 2.0;
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
        return;
    }

    let npc_specs: Vec<(&str, Vec3, f32)> =
        vec![("Gym_Wander_Enemy_1", Vec3::new(-18.0, 1.0, -8.0), 3.0)];

    gym_debug_info(format_args!(
        "Spawning {} patrolling NPC(s) for gym mode",
        npc_specs.len()
    ));

    let mut rng = rand::rng();

    for (name, spawn_position, speed) in npc_specs {
        let validated_spawn = validate_spawn_position(spawn_position, &obstacles, 0.5);
        let mut nav_agent = SimpleNavigationAgent::new(speed);
        nav_agent.arrival_threshold = 2.0;
        nav_agent.current_target = Some(random_gym_floor_point(&mut rng));

        let enemy = commands
            .spawn((
                Name::new(name),
                Position::new(validated_spawn),
                Rotation::default(),
                LinearVelocity::default(),
                Health::basic(),
                Respawnable::with_position(2.0, validated_spawn),
                Replicate::to_clients(NetworkTarget::All),
                InterpolationTarget::to_clients(NetworkTarget::All),
                CharacterMarker,
                NpcPhysicsBundle::default(),
                nav_agent,
                NavigationPathState::default(),
                GymRandomWanderer,
                GymWanderDiagnostics::new(validated_spawn),
            ))
            .id();

        // Gym NPC movement is driven directly by nav Position updates.
        // Keep body kinematic to avoid dynamic solver jitter/fighting.
        commands.entity(enemy).insert(RigidBody::Kinematic);
    }
}

pub fn update_gym_wandering_npc_targets(
    gym_mode: Option<Res<GymMode>>,
    navmesh_query: Query<(&ManagedNavMesh, Ref<NavMeshStatus>)>,
    navmeshes: Res<Assets<NavMesh>>,
    gym_obstacles: Query<&Position, With<NavigationObstacle>>,
    mut npc_query: Query<
        (&Position, &mut SimpleNavigationAgent),
        (
            With<GymRandomWanderer>,
            With<CharacterMarker>,
            Without<PlayerId>,
        ),
    >,
) {
    let is_gym_mode = gym_mode.map(|gm| gm.0).unwrap_or(false);
    if !is_gym_mode {
        return;
    }

    let navmesh = navmesh_query
        .single()
        .ok()
        .and_then(|(navmesh_handle, status)| {
            if *status == NavMeshStatus::Built {
                navmeshes.get(navmesh_handle.deref())
            } else {
                None
            }
        });

    let mut rng = rand::rng();
    let sample_extent = ROOM_HALF_EXTENT - GYM_TARGET_MARGIN;

    for (position, mut nav_agent) in &mut npc_query {
        let nav_position = to_navmesh_plane(position.0);
        let reached_target = nav_agent.current_target.is_some_and(|target| {
            Vec2::new(position.0.x, position.0.z).distance(Vec2::new(target.x, target.z))
                <= nav_agent.arrival_threshold.max(0.75)
        });

        if nav_agent.current_target.is_some() && !reached_target {
            continue;
        }

        let mut selected_target = None;
        let mut rejected_out_of_mesh = 0usize;
        let mut rejected_too_close = 0usize;
        let mut rejected_no_path = 0usize;
        for _ in 0..GYM_TARGET_SAMPLE_ATTEMPTS {
            let raw_candidate = Vec3::new(
                rng.random_range(-sample_extent..sample_extent),
                1.0,
                rng.random_range(-sample_extent..sample_extent),
            );
            let candidate = validate_spawn_position(raw_candidate, &gym_obstacles, 1.0);
            let nav_candidate = to_navmesh_plane(candidate);

            if let Some(navmesh) = navmesh
                && !navmesh.transformed_is_in_mesh(nav_candidate)
            {
                rejected_out_of_mesh += 1;
                continue;
            }

            let distance =
                Vec2::new(position.0.x, position.0.z).distance(Vec2::new(candidate.x, candidate.z));
            if distance < GYM_MIN_TARGET_DISTANCE {
                rejected_too_close += 1;
                continue;
            }

            if let Some(navmesh) = navmesh
                && navmesh
                    .transformed_path(nav_position, nav_candidate)
                    .is_none()
            {
                rejected_no_path += 1;
                continue;
            }

            selected_target = Some(candidate);
            break;
        }

        if let Some(target) = selected_target {
            gym_debug_info(format_args!(
                "Gym NPC re-targeted: pos={:?} target={:?} dist={:.2}",
                position.0,
                target,
                Vec2::new(position.0.x, position.0.z).distance(Vec2::new(target.x, target.z))
            ));
            nav_agent.current_target = Some(target);
        } else {
            gym_debug_warn(format_args!(
                "Gym NPC failed to find target this tick: pos={:?} attempts={} rejected_out_of_mesh={} rejected_too_close={} rejected_no_path={}",
                position.0,
                GYM_TARGET_SAMPLE_ATTEMPTS,
                rejected_out_of_mesh,
                rejected_too_close,
                rejected_no_path,
            ));
        }
    }
}

fn to_navmesh_plane(point: Vec3) -> Vec3 {
    Vec3::new(point.x, point.z, 0.0)
}
