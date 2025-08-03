#![allow(dead_code)]

use bevy::prelude::*;

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    #[default]
    MainMenu,
    Loading,
    InGame,
    Paused,
    GameOver,
}

#[derive(Resource, Default)]
pub struct GameSettings {
    pub master_volume: f32,
    pub effects_volume: f32,
    pub music_volume: f32,
    pub mouse_sensitivity: f32,
    pub field_of_view: f32,
}

impl GameSettings {
    pub fn new() -> Self {
        Self {
            master_volume: 1.0,
            effects_volume: 0.8,
            music_volume: 0.6,
            mouse_sensitivity: 1.0,
            field_of_view: 75.0,
        }
    }
}
