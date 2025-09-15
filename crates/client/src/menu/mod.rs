use bevy::prelude::{App, Plugin};

mod main_menu;

pub struct MenuPlugin;
impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(main_menu::MainMenuPlugin);
    }
}
