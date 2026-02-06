use bevy::prelude::{FixedUpdate, IntoScheduleConfigs, Plugin, Update};

use crate::inputs::{
    look::update_player_rotation_from_input,
    movement::{apply_movement, update_ground_detection},
};

pub mod input;
pub mod look;
pub mod movement;

pub struct SharedInputPlugin;

impl Plugin for SharedInputPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        // Movement systems (FixedUpdate for physics)
        app.add_systems(
            FixedUpdate,
            (update_ground_detection, apply_movement).chain(),
        );

        app.add_systems(Update, update_player_rotation_from_input);
    }
}
