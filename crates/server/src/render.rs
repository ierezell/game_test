use avian3d::prelude::{PhysicsDebugPlugin, Position};

use bevy::prelude::{
    App, Assets, Camera, Camera3d, Capsule3d, Commands, Entity, Mesh, Mesh3d, MeshMaterial3d, Name,
    OnAdd, Plugin, Query, ResMut, StandardMaterial, Startup, Transform, Trigger, Vec3, With,
    Without, debug, default,
};
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};
use shared::input::{PLAYER_CAPSULE_HEIGHT, PLAYER_CAPSULE_RADIUS};
use shared::{
    protocol::{PlayerColor, PlayerId},
    render::{add_floor_visuals, add_wall_visuals, setup_lighting},
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

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (spawn_camera_if_none_exists, setup_lighting));
        app.add_observer(add_floor_visuals);
        app.add_observer(add_wall_visuals);
        app.add_observer(add_player_visuals);
        app.add_plugins((
            PhysicsDebugPlugin::default(),
            EguiPlugin::default(),
            WorldInspectorPlugin::default(),
        ));
    }
}

fn add_player_visuals(
    trigger: Trigger<OnAdd, PlayerId>,
    player_query: Query<(Entity, &Position, &PlayerColor), Without<Mesh3d>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    let Ok((entity, position, color)) = player_query.get(trigger.target()) else {
        debug!("Failed to get player entity for visual addition.");
        return;
    };

    commands.entity(entity).insert((
        Mesh3d(meshes.add(Capsule3d::new(PLAYER_CAPSULE_RADIUS, PLAYER_CAPSULE_HEIGHT))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: color.0,
            ..default()
        })),
    ));
    debug!("Added player visuals at position: {:?}", position.0);
}
