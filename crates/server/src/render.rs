use avian3d::prelude::PhysicsDebugPlugin;
use avian3d::prelude::Position;
use bevy::{
    color::palettes::css::{GRAY, GREEN, WHITE},
    prelude::{
        Added, App, Assets, Camera, Camera3d, Capsule3d, Commands, Cuboid, DirectionalLight,
        Entity, Mesh, Mesh3d, MeshMaterial3d, Name, Plugin, Query, ResMut, StandardMaterial,
        Startup, Transform, Update, Vec3, With, Without, default, info,
    },
};
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};

use shared::scene::{
    FLOOR_THICKNESS, FloorMarker, PlayerMarker, ROOM_SIZE, WALL_HEIGHT, WALL_THICKNESS, WallMarker,
};

pub struct RenderPlugin;

fn spawn_camera_if_none_exists(
    mut commands: Commands,
    existing_cameras: Query<Entity, With<Camera3d>>,
) {
    if existing_cameras.is_empty() {
        commands.spawn((
            Camera3d::default(),
            Camera {
                order: 0,
                ..default()
            },
            Transform::from_xyz(-50.0, 50.0, 50.0).looking_at(Vec3::ZERO, Vec3::Y),
            Name::new("RenderCamera"),
        ));
    }
}

fn setup_lighting(mut commands: Commands) {
    commands.spawn((
        DirectionalLight {
            color: WHITE.into(),
            illuminance: 10000.0,
            ..default()
        },
        Transform::from_xyz(10.0, 10.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
        Name::new("Sun"),
    ));
}

fn add_floor_visuals(
    mut commands: Commands,
    floor_query: Query<(Entity, &Position), (Added<FloorMarker>, Without<Mesh3d>)>,
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
                base_color: GRAY.into(),
                ..default()
            })),
        ));
        info!("Added floor visuals at position: {:?}", position.0);
    }
}

fn add_wall_visuals(
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
        info!(
            "Added wall visuals for {} at position: {:?}",
            name.as_str(),
            position.0
        );
    }
}

fn add_player_visuals(
    mut commands: Commands,
    player_query: Query<(Entity, &Position), (Added<PlayerMarker>, Without<Mesh3d>)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, position) in &player_query {
        commands.entity(entity).insert((
            Mesh3d(meshes.add(Capsule3d::new(
                shared::input::PLAYER_CAPSULE_RADIUS,
                shared::input::PLAYER_CAPSULE_HEIGHT,
            ))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: GREEN.into(),
                ..default()
            })),
        ));
        info!("Added player visuals at position: {:?}", position.0);
    }
}

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (spawn_camera_if_none_exists, setup_lighting));
        app.add_systems(
            Update,
            (add_floor_visuals, add_wall_visuals, add_player_visuals),
        );
        app.add_plugins((
            PhysicsDebugPlugin::default(),
            EguiPlugin::default(),
            WorldInspectorPlugin::default(),
        ));
    }
}
