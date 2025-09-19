use avian3d::prelude::PhysicsDebugPlugin;

use bevy::prelude::{
    App, Camera, Camera3d, Commands, Entity, Name, Plugin, Query, Startup, Transform, Update, Vec3,
    With, default,
};
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};
use shared::render::{add_floor_visuals, add_player_visuals, add_wall_visuals, setup_lighting};
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
