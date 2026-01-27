/// Input Management - Client-Side Only
///
/// Separates concerns into focused modules:
/// - Cursor grab management
/// - Focus change handling  
/// - Stuck input detection
/// - Input state synchronization
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow, WindowFocused};
use leafwing_input_manager::prelude::ActionState;

use shared::input::PlayerAction;

// ============================================================================
// CURSOR MANAGEMENT - Single Responsibility
// ============================================================================

pub struct CursorManager;

impl CursorManager {
    /// Check if cursor is currently locked
    pub fn is_locked(cursor_options: &CursorOptions) -> bool {
        cursor_options.grab_mode == CursorGrabMode::Locked
    }

    /// Lock cursor (for gameplay)
    pub fn lock(cursor_options: &mut CursorOptions) {
        cursor_options.grab_mode = CursorGrabMode::Locked;
        cursor_options.visible = false;
    }

    /// Unlock cursor (for menus)
    pub fn unlock(cursor_options: &mut CursorOptions) {
        cursor_options.grab_mode = CursorGrabMode::None;
        cursor_options.visible = true;
    }

    /// Toggle cursor lock state
    pub fn toggle(cursor_options: &mut CursorOptions) {
        if Self::is_locked(cursor_options) {
            Self::unlock(cursor_options);
        } else {
            Self::lock(cursor_options);
        }
    }
}

// ============================================================================
// INPUT STATE MANAGER - Clears inputs without affecting camera
// ============================================================================

pub struct InputStateManager;

impl InputStateManager {
    /// Clear all movement/action inputs, preserve camera look
    pub fn clear_movement_inputs(action_state: &mut ActionState<PlayerAction>) {
        action_state.set_axis_pair(&PlayerAction::Move, Vec2::ZERO);
        action_state.release(&PlayerAction::Jump);
        action_state.release(&PlayerAction::Sprint);
        action_state.release(&PlayerAction::Shoot);
        action_state.release(&PlayerAction::Aim);
        // Note: PlayerAction::Look is intentionally NOT cleared to prevent camera jump
    }

    /// Enable input processing
    pub fn enable(action_state: &mut ActionState<PlayerAction>) {
        action_state.enable();
    }

    /// Disable input processing
    pub fn disable(action_state: &mut ActionState<PlayerAction>) {
        action_state.disable();
    }

    /// Reset inputs and disable (for menu/pause)
    pub fn reset_and_disable(action_state: &mut ActionState<PlayerAction>) {
        Self::clear_movement_inputs(action_state);
        Self::disable(action_state);
    }

    /// Reset inputs and enable (for gameplay)
    pub fn reset_and_enable(action_state: &mut ActionState<PlayerAction>) {
        Self::clear_movement_inputs(action_state);
        Self::enable(action_state);
    }
}

// ============================================================================
// STUCK INPUT DETECTOR - Detects input/keyboard desync
// ============================================================================

pub struct StuckInputDetector;

impl StuckInputDetector {
    /// Check if movement keys are actually pressed
    pub fn are_movement_keys_pressed(keyboard: &ButtonInput<KeyCode>) -> bool {
        // WASD keys
        let wasd = keyboard.pressed(KeyCode::KeyW)
            || keyboard.pressed(KeyCode::KeyA)
            || keyboard.pressed(KeyCode::KeyS)
            || keyboard.pressed(KeyCode::KeyD);

        // Arrow keys
        let arrows = keyboard.pressed(KeyCode::ArrowUp)
            || keyboard.pressed(KeyCode::ArrowDown)
            || keyboard.pressed(KeyCode::ArrowLeft)
            || keyboard.pressed(KeyCode::ArrowRight);

        wasd || arrows
    }

    /// Detect if ActionState shows movement but keyboard doesn't
    pub fn has_stuck_movement(
        action_state: &ActionState<PlayerAction>,
        keyboard: &ButtonInput<KeyCode>,
    ) -> bool {
        let has_input = action_state.axis_pair(&PlayerAction::Move).length_squared() > 0.0;
        let keys_pressed = Self::are_movement_keys_pressed(keyboard);

        has_input && !keys_pressed
    }

    /// Clear stuck movement input
    pub fn clear_stuck_movement(action_state: &mut ActionState<PlayerAction>) {
        action_state.set_axis_pair(&PlayerAction::Move, Vec2::ZERO);
    }
}

// ============================================================================
// SYSTEMS - Each with Single Responsibility
// ============================================================================

/// System: Toggle cursor grab on ESC key
pub fn toggle_cursor_on_escape<T: Component>(
    keys: Res<ButtonInput<KeyCode>>,
    mut cursor_query: Query<&mut CursorOptions, With<PrimaryWindow>>,
    mut action_query: Query<&mut ActionState<PlayerAction>, With<T>>,
) {
    if !keys.just_pressed(KeyCode::Escape) {
        return;
    }

    let Ok(mut cursor_options) = cursor_query.single_mut() else {
        return;
    };

    CursorManager::toggle(&mut cursor_options);

    // Update input state based on new cursor state
    if let Ok(mut action_state) = action_query.single_mut() {
        if CursorManager::is_locked(&cursor_options) {
            InputStateManager::reset_and_enable(&mut action_state);
        } else {
            InputStateManager::reset_and_disable(&mut action_state);
        }
    }
}

/// System: Handle window focus changes
pub fn handle_focus_changes<T: Component>(
    mut focus_events: MessageReader<WindowFocused>,
    mut cursor_query: Query<&mut CursorOptions, With<PrimaryWindow>>,
    mut action_query: Query<&mut ActionState<PlayerAction>, With<T>>,
) {
    for event in focus_events.read() {
        let Ok(mut cursor_options) = cursor_query.single_mut() else {
            continue;
        };
        let Ok(mut action_state) = action_query.single_mut() else {
            continue;
        };

        if event.focused {
            // Window regained focus - lock cursor and enable input
            CursorManager::lock(&mut cursor_options);
            InputStateManager::reset_and_enable(&mut action_state);
        } else {
            // Window lost focus - clear inputs to prevent stuck keys
            InputStateManager::clear_movement_inputs(&mut action_state);
        }
    }
}

/// System: Detect and clear stuck inputs
pub fn detect_stuck_inputs<T: Component>(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut action_query: Query<&mut ActionState<PlayerAction>, With<T>>,
) {
    let Ok(mut action_state) = action_query.single_mut() else {
        return;
    };

    if StuckInputDetector::has_stuck_movement(&action_state, &keyboard) {
        StuckInputDetector::clear_stuck_movement(&mut action_state);
    }
}

// ============================================================================
// TESTS - Verify State Changes Over Multiple Frames
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cursor_manager_toggle() {
        let mut options = CursorOptions {
            grab_mode: CursorGrabMode::Locked,
            visible: false,
            ..default()
        };

        assert!(CursorManager::is_locked(&options));

        CursorManager::toggle(&mut options);
        assert!(!CursorManager::is_locked(&options));
        assert!(options.visible);

        CursorManager::toggle(&mut options);
        assert!(CursorManager::is_locked(&options));
        assert!(!options.visible);
    }

    #[test]
    fn test_input_state_manager_clears_movement_only() {
        let mut action_state = ActionState::<PlayerAction>::default();

        // Set all inputs
        action_state.set_axis_pair(&PlayerAction::Move, Vec2::new(1.0, 1.0));
        action_state.set_axis_pair(&PlayerAction::Look, Vec2::new(0.5, 0.3));
        action_state.press(&PlayerAction::Jump);

        InputStateManager::clear_movement_inputs(&mut action_state);

        assert_eq!(action_state.axis_pair(&PlayerAction::Move), Vec2::ZERO);
        assert!(!action_state.pressed(&PlayerAction::Jump));
        // Look should be preserved
        assert_ne!(action_state.axis_pair(&PlayerAction::Look), Vec2::ZERO);
    }

    #[test]
    fn test_stuck_input_detector_detects_desync() {
        let mut action_state = ActionState::<PlayerAction>::default();
        action_state.set_axis_pair(&PlayerAction::Move, Vec2::new(1.0, 0.0));

        let keyboard = ButtonInput::<KeyCode>::default();

        assert!(
            StuckInputDetector::has_stuck_movement(&action_state, &keyboard),
            "Should detect stuck input when ActionState has movement but keyboard doesn't"
        );
    }

    #[test]
    fn test_stuck_input_detector_allows_valid_input() {
        let mut action_state = ActionState::<PlayerAction>::default();
        action_state.set_axis_pair(&PlayerAction::Move, Vec2::new(1.0, 0.0));

        let mut keyboard = ButtonInput::<KeyCode>::default();
        keyboard.press(KeyCode::KeyD);

        assert!(
            !StuckInputDetector::has_stuck_movement(&action_state, &keyboard),
            "Should not detect stuck input when keyboard matches ActionState"
        );
    }

    /// INTEGRATION TEST: ESC toggle over 3 frames
    #[test]
    fn test_cursor_toggle_system_integration() {
        #[derive(Component)]
        struct TestMarker;

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<ButtonInput<KeyCode>>();
        app.add_systems(Update, toggle_cursor_on_escape::<TestMarker>);

        // Spawn window and player
        app.world_mut().spawn((
            PrimaryWindow,
            CursorOptions {
                grab_mode: CursorGrabMode::Locked,
                visible: false,
                ..default()
            },
        ));

        let player = app
            .world_mut()
            .spawn((TestMarker, ActionState::<PlayerAction>::default()))
            .id();

        // Frame 1: Press ESC
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Escape);

        app.update();

        let cursor = app
            .world_mut()
            .query_filtered::<&CursorOptions, With<PrimaryWindow>>()
            .single(app.world())
            .unwrap();
        assert_eq!(cursor.grab_mode, CursorGrabMode::None);

        let action = app
            .world()
            .get::<ActionState<PlayerAction>>(player)
            .unwrap();
        assert!(action.disabled(), "ActionState should be disabled");

        // Frame 2: Release ESC
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .release(KeyCode::Escape);
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .clear_just_pressed(KeyCode::Escape);

        app.update();

        // Frame 3: Press ESC again
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Escape);

        app.update();

        let cursor = app
            .world_mut()
            .query_filtered::<&CursorOptions, With<PrimaryWindow>>()
            .single(app.world())
            .unwrap();
        assert_eq!(cursor.grab_mode, CursorGrabMode::Locked);

        let action = app
            .world()
            .get::<ActionState<PlayerAction>>(player)
            .unwrap();
        assert!(!action.disabled(), "ActionState should be enabled");
    }

    /// INTEGRATION TEST: Focus change handling over 3 frames
    #[test]
    fn test_focus_change_system_integration() {
        #[derive(Component)]
        struct TestMarker;

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<WindowFocused>();
        app.add_systems(Update, handle_focus_changes::<TestMarker>);

        app.world_mut().spawn((
            PrimaryWindow,
            CursorOptions {
                grab_mode: CursorGrabMode::Locked,
                visible: false,
                ..default()
            },
        ));

        let player = app
            .world_mut()
            .spawn((TestMarker, ActionState::<PlayerAction>::default()))
            .id();

        // Set some movement input
        app.world_mut()
            .get_mut::<ActionState<PlayerAction>>(player)
            .unwrap()
            .set_axis_pair(&PlayerAction::Move, Vec2::new(1.0, 1.0));

        // Frame 1: Lose focus
        app.world_mut().write_message(WindowFocused {
            window: Entity::PLACEHOLDER,
            focused: false,
        });

        app.update();

        let action = app
            .world()
            .get::<ActionState<PlayerAction>>(player)
            .unwrap();
        assert_eq!(
            action.axis_pair(&PlayerAction::Move),
            Vec2::ZERO,
            "Movement should be cleared on focus loss"
        );

        // Frame 2: Regain focus
        app.world_mut().write_message(WindowFocused {
            window: Entity::PLACEHOLDER,
            focused: true,
        });

        app.update();

        let cursor = app
            .world_mut()
            .query_filtered::<&CursorOptions, With<PrimaryWindow>>()
            .single(app.world())
            .unwrap();
        assert_eq!(cursor.grab_mode, CursorGrabMode::Locked);

        let action = app
            .world()
            .get::<ActionState<PlayerAction>>(player)
            .unwrap();
        assert!(!action.disabled());

        // Frame 3: Normal operation after regain
        app.world_mut()
            .get_mut::<ActionState<PlayerAction>>(player)
            .unwrap()
            .set_axis_pair(&PlayerAction::Move, Vec2::new(1.0, 0.0));

        app.update();

        let action = app
            .world()
            .get::<ActionState<PlayerAction>>(player)
            .unwrap();
        assert_ne!(action.axis_pair(&PlayerAction::Move), Vec2::ZERO);
    }

    /// INTEGRATION TEST: Stuck input detection over 3 frames
    #[test]
    fn test_stuck_input_detection_integration() {
        #[derive(Component)]
        struct TestMarker;

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<ButtonInput<KeyCode>>();
        app.add_systems(Update, detect_stuck_inputs::<TestMarker>);

        let player = app
            .world_mut()
            .spawn((TestMarker, ActionState::<PlayerAction>::default()))
            .id();

        // Frame 1: Set stuck input (ActionState has movement, keyboard doesn't)
        app.world_mut()
            .get_mut::<ActionState<PlayerAction>>(player)
            .unwrap()
            .set_axis_pair(&PlayerAction::Move, Vec2::new(1.0, 0.0));

        app.update();

        let action = app
            .world()
            .get::<ActionState<PlayerAction>>(player)
            .unwrap();
        assert_eq!(
            action.axis_pair(&PlayerAction::Move),
            Vec2::ZERO,
            "Stuck input should be cleared"
        );

        // Frame 2: Set valid input (keyboard matches)
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyW);
        app.world_mut()
            .get_mut::<ActionState<PlayerAction>>(player)
            .unwrap()
            .set_axis_pair(&PlayerAction::Move, Vec2::new(0.0, 1.0));

        app.update();

        let action = app
            .world()
            .get::<ActionState<PlayerAction>>(player)
            .unwrap();
        assert_ne!(
            action.axis_pair(&PlayerAction::Move),
            Vec2::ZERO,
            "Valid input should be preserved"
        );

        // Frame 3: Release key, input becomes stuck again
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .release(KeyCode::KeyW);

        app.update();

        let action = app
            .world()
            .get::<ActionState<PlayerAction>>(player)
            .unwrap();
        assert_eq!(
            action.axis_pair(&PlayerAction::Move),
            Vec2::ZERO,
            "Stuck input should be cleared again"
        );
    }
}
