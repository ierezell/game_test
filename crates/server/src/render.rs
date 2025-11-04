use bevy::prelude::{
    App, Camera, Camera3d, Commands, Entity, Name, Plugin, Query, Startup, Transform, Vec3, With,
};
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};
use shared::render::add_player_visuals;
pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_camera_if_none_exists);
        app.add_observer(add_player_visuals);
        app.add_plugins((EguiPlugin::default(), WorldInspectorPlugin::default()));
    }
}

fn spawn_camera_if_none_exists(
    mut commands: Commands,
    existing_cameras: Query<Entity, With<Camera3d>>,
) {
    if existing_cameras.is_empty() {
        commands.spawn((
            Camera3d::default(),
            Camera {
                order: 0,
                ..Default::default()
            },
            Transform::from_xyz(-50.0, 50.0, 50.0).looking_at(Vec3::ZERO, Vec3::Y),
            Name::new("RenderCamera"),
        ));
    }
}
