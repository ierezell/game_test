use bevy::prelude::{App, Component, Plugin, Reflect, ReflectComponent};
use serde::{Deserialize, Serialize};

pub const STAMINA_MAX: f32 = 100.0;
pub const STAMINA_DRAIN_RATE: f32 = 20.0;
pub const STAMINA_REGEN_RATE: f32 = 12.0;
pub const STAMINA_REGEN_DELAY: f32 = 1.0;

#[derive(Component, Reflect, Clone, Debug, Serialize, Deserialize, PartialEq)]
#[reflect(Component)]
pub struct Stamina {
    pub current: f32,
    pub max: f32,
    pub drain_rate: f32,
    pub regen_rate: f32,
    pub regen_delay: f32,
    pub last_drain_time: f32,
    pub exhausted: bool,
}

impl Default for Stamina {
    fn default() -> Self {
        Self {
            current: STAMINA_MAX,
            max: STAMINA_MAX,
            drain_rate: STAMINA_DRAIN_RATE,
            regen_rate: STAMINA_REGEN_RATE,
            regen_delay: STAMINA_REGEN_DELAY,
            last_drain_time: 0.0,
            exhausted: false,
        }
    }
}

impl Stamina {
    pub fn percentage(&self) -> f32 {
        if self.max <= 0.0 {
            0.0
        } else {
            (self.current / self.max).clamp(0.0, 1.0)
        }
    }

    pub fn can_sprint(&self) -> bool {
        self.current > 0.0
    }

    pub fn drain(&mut self, amount: f32, current_time: f32) {
        self.current = (self.current - amount).max(0.0);
        self.last_drain_time = current_time;
        if self.current <= 0.0 {
            self.exhausted = true;
        }
    }

    pub fn can_regenerate(&self, current_time: f32) -> bool {
        self.current < self.max && (current_time - self.last_drain_time) >= self.regen_delay
    }

    pub fn regenerate(&mut self, amount: f32, current_time: f32) {
        self.current = (self.current + amount).min(self.max);
        if self.current >= self.max {
            self.exhausted = false;
            self.last_drain_time = current_time;
        }
    }
}

pub struct StaminaPlugin;

impl Plugin for StaminaPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Stamina>();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stamina_drain_reduces_current_and_marks_exhausted_at_zero() {
        let mut stamina = Stamina {
            current: 5.0,
            max: STAMINA_MAX,
            drain_rate: STAMINA_DRAIN_RATE,
            regen_rate: STAMINA_REGEN_RATE,
            regen_delay: STAMINA_REGEN_DELAY,
            last_drain_time: 0.0,
            exhausted: false,
        };

        stamina.drain(5.0, 1.0);
        assert_eq!(stamina.current, 0.0);
        assert!(stamina.exhausted);
    }

    #[test]
    fn stamina_drain_clamps_at_zero() {
        let mut stamina = Stamina::default();
        stamina.drain(200.0, 1.0);
        assert_eq!(stamina.current, 0.0);
        assert!(stamina.exhausted);
    }

    #[test]
    fn stamina_draining_updates_last_drain_time() {
        let mut stamina = Stamina::default();
        stamina.drain(10.0, 5.0);
        assert_eq!(stamina.last_drain_time, 5.0);
    }

    #[test]
    fn stamina_can_regenerate_only_after_delay() {
        let stamina = Stamina {
            current: 50.0,
            last_drain_time: 10.0,
            ..Default::default()
        };

        assert!(!stamina.can_regenerate(10.5), "Too soon after drain");
        assert!(
            stamina.can_regenerate(11.1),
            "After regen_delay has elapsed"
        );
    }

    #[test]
    fn stamina_can_regenerate_when_not_full() {
        let stamina = Stamina {
            current: 50.0,
            max: STAMINA_MAX,
            last_drain_time: 0.0,
            ..Default::default()
        };
        assert!(stamina.can_regenerate(10.0));
    }

    #[test]
    fn stamina_can_not_regenerate_when_full() {
        let stamina = Stamina::default();
        assert!(!stamina.can_regenerate(10.0));
    }

    #[test]
    fn stamina_regen_restores_exhausted_flag_at_full() {
        let mut stamina = Stamina {
            current: 95.0,
            max: STAMINA_MAX,
            exhausted: true,
            last_drain_time: 0.0,
            ..Default::default()
        };

        stamina.regenerate(10.0, 5.0);
        assert_eq!(stamina.current, STAMINA_MAX);
        assert!(!stamina.exhausted);
        assert_eq!(stamina.last_drain_time, 5.0);
    }

    #[test]
    fn stamina_regen_clamps_at_max() {
        let mut stamina = Stamina::default();
        stamina.regenerate(500.0, 1.0);
        assert_eq!(stamina.current, STAMINA_MAX);
    }

    #[test]
    fn stamina_percentage_correct() {
        let stamina = Stamina {
            current: 25.0,
            max: 100.0,
            ..Default::default()
        };
        assert!((stamina.percentage() - 0.25).abs() < 0.001);
    }

    #[test]
    fn stamina_percentage_zero_when_empty() {
        let stamina = Stamina {
            current: 0.0,
            max: 100.0,
            ..Default::default()
        };
        assert_eq!(stamina.percentage(), 0.0);
    }

    #[test]
    fn can_sprint_when_stamina_above_zero() {
        let stamina = Stamina {
            current: 1.0,
            ..Default::default()
        };
        assert!(stamina.can_sprint());
    }

    #[test]
    fn can_not_spend_stamina_when_depleted() {
        let stamina = Stamina {
            current: 0.0,
            ..Default::default()
        };
        assert!(!stamina.can_sprint());
    }
}
