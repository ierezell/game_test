pub mod input_map;
pub mod window;

use bevy::prelude::{App, Plugin};

use crate::inputs::window::ClientWindowPlugin;

pub struct ClientInputPlugin;

impl Plugin for ClientInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ClientWindowPlugin);
    }
}
