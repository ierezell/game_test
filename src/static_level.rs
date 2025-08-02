use crate::combat::Health;
use crate::enemies::{Enemy, EnemyAI, EnemyState, EnemyType};
use avian3d::prelude::*;
use bevy::prelude::*;

pub struct StaticLevelPlugin;

impl Plugin for StaticLevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_simple_test_level);
    }
}

fn spawn_simple_test_level(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    info!("Creating simple test level...");

    // Basic materials - much brighter
    let wall_material_north = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.0, 0.0), // RED - North wall
        ..default()
    });

    let wall_material_south = materials.add(StandardMaterial {
        base_color: Color::srgb(0.0, 1.0, 0.0), // GREEN - South wall
        ..default()
    });

    let wall_material_east = materials.add(StandardMaterial {
        base_color: Color::srgb(0.0, 0.0, 1.0), // BLUE - East wall
        ..default()
    });

    let wall_material_west = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 1.0, 0.0), // YELLOW - West wall
        ..default()
    });

    let floor_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.7, 0.7, 0.7),
        ..default()
    });

    // Simple room dimensions
    let room_size = 15.0;
    let wall_height = 8.0;
    let wall_thickness = 1.0;

    // Floor - COMPLETELY SOLID at ground level
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(room_size, 0.2, room_size))),
        MeshMaterial3d(floor_material.clone()),
        Transform::from_xyz(0.0, 0.0, 0.0),
        RigidBody::Static,
        Collider::cuboid(room_size, 0.1, room_size),
        Name::new("Floor"),
    ));

    // North wall - RED
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(room_size, wall_height, wall_thickness))),
        MeshMaterial3d(wall_material_north),
        Transform::from_xyz(
            0.0,
            wall_height / 2.0,
            room_size / 2.0 + wall_thickness / 2.0,
        ),
        RigidBody::Static,
        Collider::cuboid(room_size, wall_height, wall_thickness),
        Name::new("North Wall - RED"),
    ));

    // South wall - GREEN
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(
            room_size + wall_thickness * 2.0,
            wall_height,
            wall_thickness,
        ))),
        MeshMaterial3d(wall_material_south),
        Transform::from_xyz(
            0.0,
            wall_height / 2.0,
            -room_size / 2.0 - wall_thickness / 2.0,
        ),
        RigidBody::Static,
        Collider::cuboid(
            room_size + wall_thickness * 2.0,
            wall_height,
            wall_thickness,
        ),
        Name::new("South Wall - GREEN"),
    ));

    // East wall - BLUE
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(
            wall_thickness,
            wall_height,
            room_size + wall_thickness * 2.0,
        ))),
        MeshMaterial3d(wall_material_east),
        Transform::from_xyz(
            room_size / 2.0 + wall_thickness / 2.0,
            wall_height / 2.0,
            0.0,
        ),
        RigidBody::Static,
        Collider::cuboid(
            wall_thickness,
            wall_height,
            room_size + wall_thickness * 2.0,
        ),
        Name::new("East Wall - BLUE"),
    ));

    // West wall - YELLOW
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(
            wall_thickness,
            wall_height,
            room_size + wall_thickness * 2.0,
        ))),
        MeshMaterial3d(wall_material_west),
        Transform::from_xyz(
            -room_size / 2.0 - wall_thickness / 2.0,
            wall_height / 2.0,
            0.0,
        ),
        RigidBody::Static,
        Collider::cuboid(
            wall_thickness,
            wall_height,
            room_size + wall_thickness * 2.0,
        ),
        Name::new("West Wall - YELLOW"),
    ));

    // Multiple bright lights for excellent visibility
    commands.spawn((
        PointLight {
            intensity: 10000.0,
            color: Color::WHITE,
            shadows_enabled: true,
            range: 30.0,
            ..default()
        },
        Transform::from_xyz(0.0, wall_height - 0.2, 0.0),
        Name::new("Main Ceiling Light"),
    ));

    // Corner lights for even better coverage
    commands.spawn((
        PointLight {
            intensity: 5000.0,
            color: Color::WHITE,
            shadows_enabled: false,
            range: 20.0,
            ..default()
        },
        Transform::from_xyz(-5.0, wall_height - 0.5, -5.0),
        Name::new("Corner Light 1"),
    ));

    commands.spawn((
        PointLight {
            intensity: 5000.0,
            color: Color::WHITE,
            shadows_enabled: false,
            range: 20.0,
            ..default()
        },
        Transform::from_xyz(5.0, wall_height - 0.5, 5.0),
        Name::new("Corner Light 2"),
    ));

    // One test enemy - bright red for visibility
    let enemy_material = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.0, 0.0),
        emissive: Color::srgb(0.3, 0.0, 0.0).into(),
        ..default()
    });

    commands.spawn((
        Mesh3d(meshes.add(Capsule3d::new(0.4, 1.6))),
        MeshMaterial3d(enemy_material),
        Transform::from_xyz(4.0, 0.8, 0.0),
        RigidBody::Dynamic,
        Collider::capsule(0.4, 1.6),
        Mass(50.0),
        LockedAxes::ROTATION_LOCKED,
        Enemy {
            enemy_type: EnemyType::Striker,
            aggro_range: 7.0,
            attack_range: 1.5,
            move_speed: 1.0,
            attack_damage: 20.0,
            attack_cooldown: 1.0,
            last_attack_time: 0.0,
            state: EnemyState::Patrolling,
            target: None,
        },
        EnemyAI {
            detection_radius: 7.0,
            patrol_points: vec![
                Vec3::new(4.0, 0.8, 0.0),
                Vec3::new(-3.0, 0.8, 3.0),
                Vec3::new(-3.0, 0.8, -3.0),
            ],
            current_patrol_index: 0,
            last_known_player_position: None,
            search_timer: 0.0,
            reaction_time: 0.5,
        },
        Health {
            current: 80.0,
            maximum: 80.0,
            regeneration_rate: 0.0,
            last_damage_time: 0.0,
        },
        Name::new("Test Enemy"),
    ));

    info!("Simple test level created: 15x15 room with basic lighting and one enemy");
}
