use bevy::prelude::Resource;

pub mod host;
pub mod local_menu;
pub mod native;

#[cfg(test)]
mod tests;

#[cfg(target_family = "wasm")]
pub mod wasm;

#[derive(Resource)]
pub struct AutoJoin(pub bool);
