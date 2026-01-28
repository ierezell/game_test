/// Camera Control Module - Separated from Movement for Single Responsibility
///
/// Handles first-person camera rotation and look controls.
/// Camera state is independent of movement to avoid coupling.
use bevy::prelude::*;
use leafwing_input_manager::prelude::ActionState;
use serde::{Deserialize, Serialize};

use crate::input::PlayerAction;

// ============================================================================
// COMPONENTS
// ============================================================================

/// First-person camera controller - only handles look/rotation
#[derive(Component, Reflect, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct FpsCamera {
    pub pitch: f32,
    pub yaw: f32,
    pub sensitivity: f32,
}

impl Default for FpsCamera {
    fn default() -> Self {
        Self {
            pitch: 0.0,
            yaw: 0.0,
            sensitivity: 0.0005,  // Reduced from 0.001 for less sensitive movement
        }
    }
}

// ============================================================================
// CONSTANTS
// ============================================================================

pub const PITCH_LIMIT_RADIANS: f32 = std::f32::consts::FRAC_PI_2 - 0.01;
const LOOK_DEADZONE_SQUARED: f32 = 0.000001;

// ============================================================================
// PURE FUNCTIONS
// ============================================================================

/// Get mouse look delta from input, with deadzone filtering
pub fn get_mouse_look_delta(action_state: &ActionState<PlayerAction>) -> Vec2 {
    let look_input = action_state.axis_pair(&PlayerAction::Look);
    if look_input.length_squared() < LOOK_DEADZONE_SQUARED {
        Vec2::ZERO
    } else {
        look_input
    }
}

/// Update camera rotation from mouse input
pub fn update_camera_rotation(camera: &mut FpsCamera, mouse_delta: Vec2) {
    camera.yaw -= mouse_delta.x * camera.sensitivity;
    camera.pitch -= mouse_delta.y * camera.sensitivity;
    camera.pitch = camera
        .pitch
        .clamp(-PITCH_LIMIT_RADIANS, PITCH_LIMIT_RADIANS);
}

/// Convert camera yaw/pitch to world rotation quaternion
pub fn camera_to_rotation(camera: &FpsCamera) -> Quat {
    Quat::from_euler(EulerRot::YXZ, camera.yaw, 0.0, 0.0)
}

// ============================================================================
// SYSTEM
// ============================================================================

/// System: Update camera rotation from player input
pub fn update_camera_from_input(mut query: Query<(&ActionState<PlayerAction>, &mut FpsCamera)>) {
    for (action_state, mut camera) in query.iter_mut() {
        if !action_state.disabled() {
            let mouse_delta = get_mouse_look_delta(action_state);
            update_camera_rotation(&mut camera, mouse_delta);
        }
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_camera_rotation_updates_from_mouse() {
        let mut camera = FpsCamera::default();
        let mouse_delta = Vec2::new(10.0, 5.0);

        let initial_yaw = camera.yaw;
        let initial_pitch = camera.pitch;

        update_camera_rotation(&mut camera, mouse_delta);

        assert_ne!(camera.yaw, initial_yaw, "Yaw should change");
        assert_ne!(camera.pitch, initial_pitch, "Pitch should change");
    }

    #[test]
    fn test_pitch_clamping() {
        let mut camera = FpsCamera {
            pitch: PITCH_LIMIT_RADIANS + 1.0,
            ..Default::default()
        };

        update_camera_rotation(&mut camera, Vec2::ZERO);

        assert!(
            camera.pitch <= PITCH_LIMIT_RADIANS,
            "Pitch should be clamped to limit"
        );

        // Try negative limit
        camera.pitch = -PITCH_LIMIT_RADIANS - 1.0;
        update_camera_rotation(&mut camera, Vec2::ZERO);

        assert!(
            camera.pitch >= -PITCH_LIMIT_RADIANS,
            "Pitch should be clamped to negative limit"
        );
    }

    #[test]
    fn test_look_deadzone_filters_small_input() {
        let mut action_state = ActionState::<PlayerAction>::default();
        action_state.set_axis_pair(&PlayerAction::Look, Vec2::new(0.0001, 0.0001));

        let delta = get_mouse_look_delta(&action_state);

        assert_eq!(delta, Vec2::ZERO, "Small inputs should be filtered");
    }

    #[test]
    fn test_look_deadzone_allows_large_input() {
        let mut action_state = ActionState::<PlayerAction>::default();
        action_state.set_axis_pair(&PlayerAction::Look, Vec2::new(1.0, 1.0));

        let delta = get_mouse_look_delta(&action_state);

        assert_ne!(delta, Vec2::ZERO, "Large inputs should pass through");
    }

    #[test]
    fn test_camera_to_rotation_produces_valid_quaternion() {
        let camera = FpsCamera {
            yaw: std::f32::consts::FRAC_PI_2,
            pitch: 0.0,
            sensitivity: 0.001,
        };

        let rotation = camera_to_rotation(&camera);

        assert!(rotation.is_normalized(), "Quaternion should be normalized");
    }

    /// INTEGRATION TEST: Camera system updates over 3 frames
    #[test]
    fn test_camera_system_integration() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, update_camera_from_input);

        let entity = app
            .world_mut()
            .spawn((FpsCamera::default(), ActionState::<PlayerAction>::default()))
            .id();

        // Frame 1: No input
        app.update();
        let cam1 = app.world().get::<FpsCamera>(entity).unwrap().clone();
        assert_eq!(cam1.yaw, 0.0);
        assert_eq!(cam1.pitch, 0.0);

        // Frame 2: Apply mouse input
        app.world_mut()
            .get_mut::<ActionState<PlayerAction>>(entity)
            .unwrap()
            .set_axis_pair(&PlayerAction::Look, Vec2::new(100.0, 50.0));

        app.update();
        let cam2 = app.world().get::<FpsCamera>(entity).unwrap().clone();
        assert_ne!(cam2.yaw, cam1.yaw, "Yaw should change");
        assert_ne!(cam2.pitch, cam1.pitch, "Pitch should change");

        // Frame 3: Continue input (cumulative)
        app.update();
        let cam3 = app.world().get::<FpsCamera>(entity).unwrap().clone();
        assert_ne!(cam3.yaw, cam2.yaw, "Yaw should continue changing");
    }

    /// INTEGRATION TEST: Camera system respects disabled ActionState
    #[test]
    fn test_camera_system_respects_disabled_input() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, update_camera_from_input);

        let entity = app
            .world_mut()
            .spawn((FpsCamera::default(), ActionState::<PlayerAction>::default()))
            .id();

        // Disable input
        app.world_mut()
            .get_mut::<ActionState<PlayerAction>>(entity)
            .unwrap()
            .disable();

        // Set look input (should be ignored)
        app.world_mut()
            .get_mut::<ActionState<PlayerAction>>(entity)
            .unwrap()
            .set_axis_pair(&PlayerAction::Look, Vec2::new(100.0, 100.0));

        app.update();
        let camera = app.world().get::<FpsCamera>(entity).unwrap();
        assert_eq!(
            camera.yaw, 0.0,
            "Camera should not update when input disabled"
        );
        assert_eq!(camera.pitch, 0.0);
    }
}
