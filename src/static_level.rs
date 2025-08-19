use crate::combat::Health;
use crate::enemies::{Enemy, EnemyAI, EnemyState, EnemyType};
// Events removed - not needed in static_level
use crate::menu::GameState;
use crate::scenes::*;
use avian3d::prelude::*;
use bevy::prelude::*;

/// Marker component to identify level entities that can be deleted
#[derive(Component)]
pub struct LevelEntity;

pub struct StaticLevelPlugin;

impl Plugin for StaticLevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_scene_based_level)
            .add_systems(
                OnEnter(GameState::InGame),
                (delete_level, spawn_scene_based_level).chain(),
            );
    }
}

fn spawn_scene_based_level(
    mut commands: Commands,
    _asset_server: Res<AssetServer>,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<StandardMaterial>>,
    // Level events removed - unused dead code
) {
    debug!("Creating scene-based level...");

    // Create level data
    let level_data = LevelData {
        name: "Test Level".to_string(),
        difficulty: 1,
        enemy_spawn_points: vec![
            Vec3::new(10.0, 1.0, 10.0),
            Vec3::new(-10.0, 1.0, -10.0),
            Vec3::new(20.0, 1.0, 0.0),
        ],
        player_spawn_point: Vec3::new(0.0, 1.7, 0.0),
        ambient_light_color: Color::srgb(0.1, 0.1, 0.2),
        fog_settings: FogSettings {
            color: Color::srgb(0.1, 0.1, 0.2),
            density: 0.02,
            start_distance: 10.0,
            end_distance: 50.0,
        },
    };

    // Create basic level entity without scene loading to avoid asset errors
    let _level_entity = commands.spawn((
        Transform::default(),
        GlobalTransform::default(),
        Name::new(format!("Level: {}", level_data.name)),
        LevelEntity,
    )).id();

    // Use procedural generation instead of scene loading
    spawn_simple_test_level_fallback(&mut commands, meshes, materials);

    // Level loaded event removed - unused dead code
}

fn spawn_simple_test_level_fallback(
    commands: &mut Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    debug!("Creating simple test level...");

    let wall_material_north = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.0, 0.0),
        ..default()
    });

    let wall_material_south = materials.add(StandardMaterial {
        base_color: Color::srgb(0.0, 1.0, 0.0),
        ..default()
    });

    let wall_material_east = materials.add(StandardMaterial {
        base_color: Color::srgb(0.0, 0.0, 1.0),
        ..default()
    });

    let wall_material_west = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 1.0, 0.0),
        ..default()
    });

    let floor_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.7, 0.7, 0.7),
        ..default()
    });

    let room_size = 15.0;
    let wall_height = 4.0;
    let wall_thickness = 1.0;

    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(room_size, 0.2, room_size))),
        MeshMaterial3d(floor_material.clone()),
        Transform::from_xyz(0.0, 0.0, 0.0),
        RigidBody::Static,
        Collider::cuboid(room_size, wall_thickness, room_size),
        CollisionLayers::default(),
        Name::new("Floor"),
        LevelEntity,
    ));

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
        CollisionLayers::default(),
        Name::new("North Wall - RED"),
        LevelEntity,
    ));

    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(room_size, wall_height, wall_thickness))),
        MeshMaterial3d(wall_material_south),
        Transform::from_xyz(
            0.0,
            wall_height / 2.0,
            -room_size / 2.0 - wall_thickness / 2.0,
        ),
        RigidBody::Static,
        Collider::cuboid(room_size, wall_height, wall_thickness),
        CollisionLayers::default(),
        Name::new("South Wall - GREEN"),
        LevelEntity,
    ));

    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(wall_thickness, wall_height, room_size))),
        MeshMaterial3d(wall_material_east),
        Transform::from_xyz(
            room_size / 2.0 + wall_thickness / 2.0,
            wall_height / 2.0,
            0.0,
        ),
        RigidBody::Static,
        Collider::cuboid(wall_thickness, wall_height, room_size),
        CollisionLayers::default(),
        Name::new("East Wall - BLUE"),
        LevelEntity,
    ));

    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(wall_thickness, wall_height, room_size))),
        MeshMaterial3d(wall_material_west),
        Transform::from_xyz(
            -room_size / 2.0 - wall_thickness / 2.0,
            wall_height / 2.0,
            0.0,
        ),
        RigidBody::Static,
        Collider::cuboid(wall_thickness, wall_height, room_size),
        CollisionLayers::default(),
        Name::new("West Wall - YELLOW"),
        LevelEntity,
    ));

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
        LevelEntity,
    ));

    commands.spawn((
        PointLight {
            intensity: 5000.0,
            color: Color::WHITE,
            shadows_enabled: false,
            range: 20.0,
            ..default()
        },
        Transform::from_xyz(room_size, wall_height + 0.5, room_size),
        Name::new("Corner Light 1"),
        LevelEntity,
    ));

    commands.spawn((
        PointLight {
            intensity: 5000.0,
            color: Color::WHITE,
            shadows_enabled: false,
            range: 20.0,
            ..default()
        },
        Transform::from_xyz(-room_size, wall_height + 0.5, -room_size),
        Name::new("Corner Light 2"),
        LevelEntity,
    ));

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
        CollisionLayers::default(),
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
            state: EnemyState::Dormant,
            target: None,
        },
        EnemyAI {
            detection_radius: 3.0,
            patrol_points: vec![
                Vec3::new(4.0, 0.8, 0.0),
                Vec3::new(-3.0, 0.8, 3.0),
                Vec3::new(-3.0, 0.8, -3.0),
            ],
            current_patrol_index: 0,
            last_known_player_position: None,
            search_timer: 0.0,
        },
        Health {
            current: 100.0,
            maximum: 100.0,
            last_damage_time: 0.0,
        },
        Name::new("Test Enemy"),
        LevelEntity,
    ));

    debug!("Simple test level created: 15x15 room with basic lighting and one enemy");
}

/// Deletes all level entities from the world
pub fn delete_level(mut commands: Commands, level_entities: Query<Entity, With<LevelEntity>>) {
    debug!("Deleting level entities...");

    let mut deleted_count = 0;
    for entity in level_entities.iter() {
        commands.entity(entity).despawn();
        deleted_count += 1;
    }

    debug!("Deleted {} level entities", deleted_count);
}
