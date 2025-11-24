use bevy::prelude::Resource;

pub mod native;
pub mod local_menu;

#[cfg(target_family = "wasm")]
pub mod wasm;

#[derive(Resource)]
pub struct AutoHost(pub bool);

#[derive(Resource)]
pub struct AutoJoin(pub bool);
