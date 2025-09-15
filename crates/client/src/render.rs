use avian3d::prelude::PhysicsDebugPlugin;
use bevy::{
    color::palettes::css::WHITE,
    prelude::{
        App, Camera, Camera3d, Commands, Entity, Name, Plugin, Query, Startup, Transform, Vec3,
        With, default,
    },
};
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};

pub struct RenderPlugin;

fn spawn_camera_if_none_exists(
    mut commands: Commands,
    existing_cameras: Query<Entity, With<Camera3d>>,
) {
    if existing_cameras.is_empty() {
        commands.spawn((
            Camera3d::default(),
            Camera {
                order: 0, // Set order 0 for render camera
                ..default()
            },
            Transform::from_xyz(-50.0, 50.0, 50.0).looking_at(Vec3::ZERO, Vec3::Y),
            Name::new("RenderCamera"),
        ));
    }
}

// If the headless server can't run it or doesn't need it
// It goes in this plugin
impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_camera_if_none_exists);
        app.add_plugins((
            //PhysicsDebugPlugin::default(),
            EguiPlugin::default(),
            WorldInspectorPlugin::default(),
        ));
    }
}
