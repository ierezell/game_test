mod flashlight;
mod gun;

use crate::vfx::flashlight::ClientFlashlightPlugin;
use crate::vfx::gun::GunEffectsPlugin;
use bevy::prelude::*;

pub struct ClientVFXPlugin;

impl Plugin for ClientVFXPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(GunEffectsPlugin);
        app.add_plugins(ClientFlashlightPlugin);
    }
}
