pub mod entity;
pub mod game_state;
pub mod input;

pub mod network;
pub mod render;
use bevy::prelude::Resource;

#[derive(Resource)]
pub struct LocalPlayerId(pub u64);
