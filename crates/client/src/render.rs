use bevy::prelude::{App, Plugin, Startup, Update};
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};
use shared::render::{add_floor_visuals, add_wall_visuals, setup_lighting};

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_lighting);
        app.add_systems(Update, (add_floor_visuals, add_wall_visuals));
        app.add_plugins((EguiPlugin::default(), WorldInspectorPlugin::new()));
    }
}
