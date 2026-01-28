use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Component indicating the player has a flashlight attached
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Reflect, Serialize, Deserialize)]
#[reflect(Component)]
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
            is_on: true,           // Start ON so player can see immediately
            intensity: 1000000.0,  // Much brighter - increased from 500000.0
            range: 80.0,           // Longer range - increased from 60.0
            inner_angle: 0.12,     // Very tight core - reduced from 0.15
            outer_angle: 0.35,     // Tighter cone - reduced from 0.4
        }
    }

    pub fn toggle(&mut self) {
        self.is_on = !self.is_on;
    }
}

/// Marker component for the flashlight's SpotLight entity
#[derive(Component, Clone, Copy, Debug, Default, PartialEq)]
pub struct FlashlightBeam;
