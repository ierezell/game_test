pub mod input;
pub mod window;

use bevy::prelude::{App, Plugin, Update};

use crate::inputs::window::{grab_cursor, handle_focus_change, toggle_cursor_grab};

pub struct ClientInputPlugin;

impl Plugin for ClientInputPlugin {
    fn build(&self, app: &mut App) {
        // Client specific input systems
        app.add_systems(Update, (toggle_cursor_grab, handle_focus_change));
        app.add_observer(grab_cursor);
    }
}
