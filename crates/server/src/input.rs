use bevy::prelude::Plugin;

use shared::inputs::SharedInputPlugin;

pub struct ServerInputPlugin;

impl Plugin for ServerInputPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins(SharedInputPlugin);
    }
}
