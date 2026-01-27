/// Tests for client input handling, cursor management, and focus changes
/// These tests verify that input state is correctly managed to prevent stuck keys
/// and that camera state is preserved during cursor grab/ungrab operations.
#[cfg(test)]
mod input_handling_tests {
    use avian3d::prelude::SpatialQueryPipeline;
    use bevy::input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll};
    use bevy::prelude::*;
    use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow, WindowFocused};
    use leafwing_input_manager::plugin::InputManagerPlugin;
    use leafwing_input_manager::prelude::*;
    use lightyear::prelude::{Controlled, PeerId, Predicted};

    use ::client::input::ClientInputPlugin;
    use shared::input::PlayerAction;
    use shared::camera::FpsCamera;
    use shared::protocol::PlayerId;

    fn get_player_input_map() -> InputMap<PlayerAction> {
        InputMap::<PlayerAction>::default()
            .with(PlayerAction::Jump, KeyCode::Space)
            .with(PlayerAction::Shoot, MouseButton::Left)
            .with(PlayerAction::Aim, MouseButton::Right)
            .with(PlayerAction::Sprint, KeyCode::ShiftLeft)
            .with_dual_axis(PlayerAction::Move, VirtualDPad::wasd())
            .with_dual_axis(PlayerAction::Move, VirtualDPad::arrow_keys())
            .with_dual_axis(PlayerAction::Look, MouseMove::default())
    }

    fn setup_test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<ButtonInput<KeyCode>>();
        app.init_resource::<ButtonInput<MouseButton>>();
        app.init_resource::<AccumulatedMouseMotion>();
        app.init_resource::<AccumulatedMouseScroll>();
        app.init_resource::<SpatialQueryPipeline>(); // Add physics resource needed by client_player_movement
        app.add_message::<WindowFocused>();
        app.add_plugins(InputManagerPlugin::<PlayerAction>::default());
        app.add_plugins(ClientInputPlugin);

        // Spawn a primary window
        app.world_mut().spawn((
            PrimaryWindow,
            CursorOptions {
                grab_mode: CursorGrabMode::Locked,
                visible: false,
                ..default()
            },
        ));

        app
    }

    fn spawn_test_player(world: &mut World) -> Entity {
        let input_map = get_player_input_map();
        let mut action_state = ActionState::<PlayerAction>::default();
        action_state.enable();

        world
            .spawn((
                PlayerId(PeerId::Netcode(0)),
                Predicted,
                Controlled,
                input_map,
                action_state,
                FpsCamera {
                    pitch: 0.5, // Non-zero pitch to verify preservation
                    yaw: 1.0,   // Non-zero yaw to verify preservation
                    ..default()
                },
            ))
            .id()
    }

    #[test]
    fn test_escape_toggles_cursor_grab() {
        let mut app = setup_test_app();
        let _player = spawn_test_player(app.world_mut());

        // Initial state: cursor locked
        let initial_grab = {
            let mut query = app.world_mut().query::<&CursorOptions>();
            query.single(app.world()).unwrap().grab_mode
        };
        assert_eq!(initial_grab, CursorGrabMode::Locked);

        // First ESC toggle: Locked -> None
        {
            let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            keys.clear();
            keys.press(KeyCode::Escape);
        }
        app.update();

        // Verify cursor unlocked
        {
            let mut query = app.world_mut().query::<&CursorOptions>();
            let opts = query.single(app.world()).unwrap();
            assert_eq!(
                opts.grab_mode,
                CursorGrabMode::None,
                "After first ESC, should be None"
            );
            assert!(opts.visible);
        }

        // Clear just_pressed state and release
        {
            let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            keys.clear_just_pressed(KeyCode::Escape);
            keys.release(KeyCode::Escape);
        }
        app.update();

        // Second ESC toggle: None -> Locked
        {
            let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            keys.clear_just_pressed(KeyCode::Escape);
            keys.press(KeyCode::Escape);
        }
        app.update();

        // Verify cursor locked again
        {
            let mut query = app.world_mut().query::<&CursorOptions>();
            let opts = query.single(app.world()).unwrap();
            assert_eq!(
                opts.grab_mode,
                CursorGrabMode::Locked,
                "After second ESC, should be Locked"
            );
            assert!(!opts.visible);
        }
    }

    #[test]
    fn test_escape_resets_movement_but_preserves_camera() {
        let mut app = setup_test_app();
        let player = spawn_test_player(app.world_mut());

        // Set movement and camera state
        {
            let mut action_state = app
                .world_mut()
                .get_mut::<ActionState<PlayerAction>>(player)
                .unwrap();
            action_state.set_axis_pair(&PlayerAction::Move, Vec2::new(1.0, 1.0));
            action_state.press(&PlayerAction::Jump);
            action_state.set_axis_pair(&PlayerAction::Look, Vec2::new(0.5, -0.3));
        }

        let controller = app.world().get::<FpsCamera>(player).unwrap();
        let initial_pitch = controller.pitch;
        let initial_yaw = controller.yaw;

        // Press ESC to ungrab cursor
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Escape);
        app.update();

        // Verify movement inputs cleared
        let action_state = app
            .world()
            .get::<ActionState<PlayerAction>>(player)
            .unwrap();
        assert_eq!(action_state.axis_pair(&PlayerAction::Move), Vec2::ZERO);
        assert!(!action_state.pressed(&PlayerAction::Jump));

        // Verify ActionState disabled when cursor ungrabbed
        assert!(
            action_state.disabled(),
            "ActionState should be disabled when cursor ungrabbed"
        );

        // Verify camera rotation preserved in FpsCamera
        let controller = app.world().get::<FpsCamera>(player).unwrap();
        assert_eq!(
            controller.pitch, initial_pitch,
            "Camera pitch should be preserved"
        );
        assert_eq!(
            controller.yaw, initial_yaw,
            "Camera yaw should be preserved"
        );
    }

    #[test]
    fn test_focus_lost_clears_movement_inputs() {
        let mut app = setup_test_app();
        let player = spawn_test_player(app.world_mut());

        // Set movement inputs
        {
            let mut action_state = app
                .world_mut()
                .get_mut::<ActionState<PlayerAction>>(player)
                .unwrap();
            action_state.set_axis_pair(&PlayerAction::Move, Vec2::new(1.0, 0.0));
            action_state.press(&PlayerAction::Sprint);
        }

        // Simulate window focus loss
        app.world_mut().write_message(WindowFocused {
            window: Entity::PLACEHOLDER,
            focused: false,
        });
        app.update();

        // Verify movement cleared
        let action_state = app
            .world()
            .get::<ActionState<PlayerAction>>(player)
            .unwrap();
        assert_eq!(action_state.axis_pair(&PlayerAction::Move), Vec2::ZERO);
        assert!(!action_state.pressed(&PlayerAction::Sprint));
    }

    #[test]
    fn test_focus_regained_resets_movement_and_relocks_cursor() {
        let mut app = setup_test_app();
        let player = spawn_test_player(app.world_mut());

        // Simulate focus loss first
        app.world_mut().write_message(WindowFocused {
            window: Entity::PLACEHOLDER,
            focused: false,
        });
        app.update();

        // Unlock cursor (simulating user action during focus loss)
        app.world_mut()
            .query::<&mut CursorOptions>()
            .single_mut(app.world_mut())
            .unwrap()
            .grab_mode = CursorGrabMode::None;

        // Set some stuck movement input
        {
            let mut action_state = app
                .world_mut()
                .get_mut::<ActionState<PlayerAction>>(player)
                .unwrap();
            action_state.set_axis_pair(&PlayerAction::Move, Vec2::new(1.0, 1.0));
        }

        let controller = app.world().get::<FpsCamera>(player).unwrap();
        let initial_pitch = controller.pitch;

        // Simulate focus regain
        app.world_mut().write_message(WindowFocused {
            window: Entity::PLACEHOLDER,
            focused: true,
        });
        app.update();

        // Verify movement cleared
        let action_state = app
            .world()
            .get::<ActionState<PlayerAction>>(player)
            .unwrap();
        assert_eq!(action_state.axis_pair(&PlayerAction::Move), Vec2::ZERO);

        // Verify ActionState enabled
        let action_disabled = action_state.disabled();

        // Verify cursor re-locked
        let cursor = app
            .world_mut()
            .query::<&CursorOptions>()
            .single(app.world())
            .unwrap();
        assert_eq!(cursor.grab_mode, CursorGrabMode::Locked);
        assert!(!cursor.visible);

        assert!(!action_disabled);

        // Verify camera rotation preserved
        let controller = app.world().get::<FpsCamera>(player).unwrap();
        assert_eq!(controller.pitch, initial_pitch);
    }

    #[test]
    fn test_stuck_input_detection_clears_movement() {
        let mut app = setup_test_app();
        let player = spawn_test_player(app.world_mut());

        // Simulate stuck movement input (ActionState shows movement but keyboard doesn't)
        {
            let mut action_state = app
                .world_mut()
                .get_mut::<ActionState<PlayerAction>>(player)
                .unwrap();
            action_state.set_axis_pair(&PlayerAction::Move, Vec2::new(1.0, 0.0));
        }

        // Ensure no keyboard keys are pressed
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .clear();

        // Run update (detect_stuck_inputs runs in Update)
        app.update();

        // Verify movement cleared by stuck input detection
        let action_state = app
            .world()
            .get::<ActionState<PlayerAction>>(player)
            .unwrap();
        assert_eq!(
            action_state.axis_pair(&PlayerAction::Move),
            Vec2::ZERO,
            "Stuck input detection should clear movement when no keys pressed"
        );
    }

    #[test]
    fn test_stuck_input_detection_preserves_valid_input() {
        let mut app = setup_test_app();
        let player = spawn_test_player(app.world_mut());

        // Set movement input matching keyboard state
        {
            let mut action_state = app
                .world_mut()
                .get_mut::<ActionState<PlayerAction>>(player)
                .unwrap();
            action_state.set_axis_pair(&PlayerAction::Move, Vec2::new(0.0, 1.0));
        }

        // Press W key to match the movement input
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyW);
        app.update();

        // Verify movement NOT cleared (valid input)
        let action_state = app
            .world()
            .get::<ActionState<PlayerAction>>(player)
            .unwrap();
        let move_input = action_state.axis_pair(&PlayerAction::Move);
        assert!(
            move_input.length_squared() > 0.0,
            "Valid input should be preserved when matching keyboard state"
        );
    }

    #[test]
    fn test_action_state_disabled_when_cursor_ungrabbed() {
        let mut app = setup_test_app();
        let player = spawn_test_player(app.world_mut());

        // Initial state: enabled
        let action_state = app
            .world()
            .get::<ActionState<PlayerAction>>(player)
            .unwrap();
        assert!(!action_state.disabled());

        // Press ESC to ungrab
        {
            let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            keys.clear();
            keys.press(KeyCode::Escape);
        }
        app.update();

        // Verify disabled
        let action_state = app
            .world()
            .get::<ActionState<PlayerAction>>(player)
            .unwrap();
        assert!(action_state.disabled());

        // Clear and press ESC again to grab
        {
            let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            keys.clear_just_pressed(KeyCode::Escape);
            keys.release(KeyCode::Escape);
        }
        app.update();
        {
            let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            keys.clear_just_pressed(KeyCode::Escape);
            keys.press(KeyCode::Escape);
        }
        app.update();

        // Verify enabled
        let action_state = app
            .world()
            .get::<ActionState<PlayerAction>>(player)
            .unwrap();
        assert!(
            !action_state.disabled(),
            "ActionState should be enabled when cursor re-grabbed"
        );
    }

    #[test]
    fn test_multiple_focus_changes_preserve_camera() {
        let mut app = setup_test_app();
        let player = spawn_test_player(app.world_mut());

        let controller = app.world().get::<FpsCamera>(player).unwrap();
        let initial_pitch = controller.pitch;
        let initial_yaw = controller.yaw;

        // Multiple focus changes
        for _ in 0..5 {
            app.world_mut().write_message(WindowFocused {
                window: Entity::PLACEHOLDER,
                focused: false,
            });
            app.update();

            app.world_mut().write_message(WindowFocused {
                window: Entity::PLACEHOLDER,
                focused: true,
            });
            app.update();
        }

        // Verify camera rotation still preserved
        let controller = app.world().get::<FpsCamera>(player).unwrap();
        assert_eq!(controller.pitch, initial_pitch);
        assert_eq!(controller.yaw, initial_yaw);
    }

    #[test]
    fn test_cursor_grab_mode_toggle_cycle() {
        let mut app = setup_test_app();
        let _player = spawn_test_player(app.world_mut());

        // Initial: locked
        let cursor = app
            .world_mut()
            .query::<&CursorOptions>()
            .single(app.world())
            .unwrap();
        assert_eq!(cursor.grab_mode, CursorGrabMode::Locked);

        // Unlock
        app.world_mut()
            .query::<&mut CursorOptions>()
            .single_mut(app.world_mut())
            .unwrap()
            .grab_mode = CursorGrabMode::None;

        let cursor = app
            .world_mut()
            .query::<&CursorOptions>()
            .single(app.world())
            .unwrap();
        assert_eq!(cursor.grab_mode, CursorGrabMode::None);
    }
}
