use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Component indicating the player has a flashlight attached
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct PlayerFlashlight {
    /// Whether the flashlight is currently on
    pub is_on: bool,
    /// Intensity of the flashlight
    pub intensity: f32,
    /// Range of the flashlight beam
    pub range: f32,
    /// Inner angle of the spotlight cone (in radians)
    pub inner_angle: f32,
    /// Outer angle of the spotlight cone (in radians)
    pub outer_angle: f32,
}

impl PlayerFlashlight {
    pub fn new() -> Self {
        Self {
            is_on: true,          // Start ON so player can see immediately
            intensity: 1400000.0, // Brighter beam for dark procedural levels
            range: 100.0,         // Longer throw distance
            inner_angle: 0.11,
            outer_angle: 0.38,
        }
    }

    pub fn toggle(&mut self) {
        self.is_on = !self.is_on;
    }
}
