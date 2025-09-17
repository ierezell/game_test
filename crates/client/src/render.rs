use avian3d::prelude::Position;
use bevy::{
    color::palettes::css::{BROWN, GRAY},
    prelude::{
        Added, App, Assets, Camera, Camera3d, Commands, Cuboid, Entity, Mesh, Mesh3d,
        MeshMaterial3d, Name, Or, Plugin, PointLight, Query, ResMut, StandardMaterial, Startup,
        Transform, Update, Vec3, With, default, info,
    },
};
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};
use lightyear::prelude::*;
use shared::scene::{
    FLOOR_THICKNESS, FloorMarker, ROOM_SIZE, WALL_HEIGHT, WALL_THICKNESS, WallMarker,
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
            Transform::from_xyz(-15.0, 10.0, 15.0).looking_at(Vec3::ZERO, Vec3::Y),
            Name::new("RenderCamera"),
        ));

        commands.spawn((
            PointLight {
                intensity: 2000.0,
                shadows_enabled: true,
                ..default()
            },
            Transform::from_xyz(4.0, 8.0, 4.0),
            Name::new("MainLight"),
        ));
    }
}

fn add_floor_visuals(
    mut commands: Commands,
    floor_query: Query<Entity, (Or<(Added<Replicated>, Added<Predicted>)>, With<FloorMarker>)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for entity in &floor_query {
        info!(?entity, "Adding visuals to floor");
        commands.entity(entity).insert((
            Mesh3d(meshes.add(Cuboid::new(ROOM_SIZE, FLOOR_THICKNESS, ROOM_SIZE))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: GRAY.into(),
                ..default()
            })),
        ));
    }
}

fn add_wall_visuals(
    mut commands: Commands,
    wall_query: Query<
        (Entity, &Position),
        (Or<(Added<Replicated>, Added<Predicted>)>, With<WallMarker>),
    >,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, position) in &wall_query {
        info!(
            ?entity,
            "Adding visuals to wall at position: {:?}", position.0
        );

        // Determine wall orientation based on position like the server does
        // North/South walls have non-zero Z positions, East/West walls have non-zero X positions
        let (width, height, depth) = if position.0.z.abs() > position.0.x.abs() {
            // North/South wall (positioned along Z-axis)
            (ROOM_SIZE, WALL_HEIGHT, WALL_THICKNESS)
        } else {
            // East/West wall (positioned along X-axis)
            (WALL_THICKNESS, WALL_HEIGHT, ROOM_SIZE)
        };

        commands.entity(entity).insert((
            Mesh3d(meshes.add(Cuboid::new(width, height, depth))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: BROWN.into(),
                ..default()
            })),
        ));
    }
}

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_camera_if_none_exists);
        app.add_systems(Update, (add_floor_visuals, add_wall_visuals));
        app.add_plugins((EguiPlugin::default(), WorldInspectorPlugin::default()));
    }
}
