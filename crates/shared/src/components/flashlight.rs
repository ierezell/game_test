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
    pub const ON_INTENSITY: f32 = 1400000.0;
    pub const ON_RANGE: f32 = 100.0;
    pub const ON_INNER_ANGLE: f32 = 0.11;
    pub const ON_OUTER_ANGLE: f32 = 0.38;

    pub fn new() -> Self {
        Self {
            is_on: true,
            intensity: Self::ON_INTENSITY,
            range: Self::ON_RANGE,
            inner_angle: Self::ON_INNER_ANGLE,
            outer_angle: Self::ON_OUTER_ANGLE,
        }
    }

    pub fn toggle(&mut self) {
        self.is_on = !self.is_on;
        if self.is_on {
            self.intensity = Self::ON_INTENSITY;
            self.range = Self::ON_RANGE;
            self.inner_angle = Self::ON_INNER_ANGLE;
            self.outer_angle = Self::ON_OUTER_ANGLE;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flashlight_new_starts_on() {
        let flashlight = PlayerFlashlight::new();
        assert!(flashlight.is_on, "Flashlight should start ON");
        assert_eq!(
            flashlight.intensity, 1400000.0,
            "Flashlight should have high intensity"
        );
        assert_eq!(flashlight.range, 100.0, "Flashlight should have long range");
        assert!(
            flashlight.inner_angle > 0.0,
            "Inner angle should be positive"
        );
        assert!(
            flashlight.outer_angle > flashlight.inner_angle,
            "Outer angle should be larger than inner"
        );
    }

    #[test]
    fn flashlight_toggle_switches_state() {
        let mut flashlight = PlayerFlashlight::new();
        assert!(flashlight.is_on, "Should start ON");

        flashlight.toggle();
        assert!(!flashlight.is_on, "Should be OFF after first toggle");

        flashlight.toggle();
        assert!(flashlight.is_on, "Should be ON after second toggle");
    }

    #[test]
    fn flashlight_default_starts_off() {
        let flashlight = PlayerFlashlight::default();
        assert!(!flashlight.is_on, "Default flashlight should start OFF");
        assert_eq!(flashlight.intensity, 0.0, "Default intensity should be 0");
        assert_eq!(flashlight.range, 0.0, "Default range should be 0");
    }

    #[test]
    fn flashlight_angles_are_sensible() {
        let flashlight = PlayerFlashlight::new();
        // Inner angle ~6.3 degrees, outer angle ~21.8 degrees
        assert!(
            flashlight.inner_angle < flashlight.outer_angle,
            "Inner < Outer"
        );
        assert!(
            flashlight.outer_angle < std::f32::consts::FRAC_PI_2,
            "Outer < 90 degrees"
        );
    }
}
