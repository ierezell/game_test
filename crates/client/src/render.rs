use bevy::prelude::*;
use bevy_inspector_egui::{
    bevy_egui::{EguiGlobalSettings, EguiPlugin, PrimaryEguiContext},
    quick::WorldInspectorPlugin,
};
use shared::render::{add_floor_visuals, add_wall_visuals, setup_lighting};

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (setup_lighting, spawn_debug_camera));
        app.add_systems(Update, (add_floor_visuals, add_wall_visuals));
        app.insert_resource(EguiGlobalSettings {
            auto_create_primary_context: false,
            ..Default::default()
        });
        app.add_plugins((EguiPlugin::default(), WorldInspectorPlugin::default()));
    }
}

#[derive(Component)]
struct DebugCamera;

fn spawn_debug_camera(mut commands: Commands) {
    commands.spawn((
        Camera {
            order: 100,
            ..default()
        },
        Camera2d::default(),
        DebugCamera,
        PrimaryEguiContext,
    ));
}
