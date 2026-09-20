use avian3d::prelude::Rotation;
use bevy::prelude::{Entity, EulerRot, Quat, Query, Vec2, With};
use bevy_enhanced_input::action::Action;
use bevy_enhanced_input::prelude::Actions;

use crate::{
    inputs::{Look, PITCH_LIMIT_RADIANS, PlayerActions},
    protocol::{CharacterMarker, PlayerId},
};

const LOOK_DEADZONE_SQUARED: f32 = 0.000001;
pub const MOUSE_SENSIVITY: f32 = 0.0007;

pub fn get_mouse_look_delta(action: &Action<Look>) -> Vec2 {
    let look_input: Vec2 = **action;
    if look_input.length_squared() < LOOK_DEADZONE_SQUARED {
        Vec2::ZERO
    } else {
        look_input
    }
}

pub fn apply_look_delta(current_rotation: Quat, mouse_delta: Vec2) -> Quat {
    let (mut yaw, mut pitch, _) = current_rotation.to_euler(EulerRot::YXZ);

    yaw += -mouse_delta.x * MOUSE_SENSIVITY;
    pitch = (pitch + (-mouse_delta.y * MOUSE_SENSIVITY))
        .clamp(-PITCH_LIMIT_RADIANS, PITCH_LIMIT_RADIANS);

    Quat::from_euler(EulerRot::YXZ, yaw, pitch, 0.0)
}

/// Update the player's rotation based on mouse look input.
///
/// Supports two Action<T> access patterns:
/// 1. Production: `Action<Look>` spawned as a related child of `Actions<PlayerActions>`
///    via the `actions!` macro — resolved by iterating the relationship.
/// 2. Legacy/test: `Action<Look>` placed directly as a component on the entity
///    — resolved via a fallback query on the entity itself.
pub fn update_player_rotation_from_input(
    mut player_query: Query<
        (Option<&Actions<PlayerActions>>, Entity, &mut Rotation),
        (With<PlayerActions>, With<CharacterMarker>, With<PlayerId>),
    >,
    look_query: Query<&Action<Look>>,
) {
    for (actions, entity, mut rotation) in player_query.iter_mut() {
        let look_action: Option<&Action<Look>> = if let Some(actions) = actions {
            // Production path: Action<Look> is a related child of Actions<PlayerActions>
            actions
                .iter()
                .find_map(|action_entity| look_query.get(*action_entity).ok())
        } else {
            // Legacy/test path: Action<Look> is directly on the entity
            look_query.get(entity).ok()
        };

        if let Some(look_action) = look_action {
            let mouse_delta = get_mouse_look_delta(look_action);
            if mouse_delta != Vec2::ZERO {
                rotation.0 = apply_look_delta(rotation.0, mouse_delta);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{apply_look_delta, get_mouse_look_delta};
    use crate::inputs::{Look, PITCH_LIMIT_RADIANS};
    use bevy::prelude::Vec2;
    use bevy_enhanced_input::action::Action;

    #[test]
    fn look_delta_applies_deadzone() {
        let mut look_action = Action::<Look>::default();
        *look_action = Vec2::new(0.0001, 0.0001);
        let delta = get_mouse_look_delta(&look_action);
        assert_eq!(delta, Vec2::ZERO);
    }

    #[test]
    fn look_delta_preserves_valid_input() {
        let mut look_action = Action::<Look>::default();
        let expected = Vec2::new(0.25, -0.75);
        *look_action = expected;
        let delta = get_mouse_look_delta(&look_action);
        assert_eq!(delta, expected);
    }

    #[test]
    fn apply_look_delta_accumulates_rotation() {
        let first = apply_look_delta(bevy::prelude::Quat::IDENTITY, Vec2::new(100.0, 0.0));
        let second = apply_look_delta(first, Vec2::new(100.0, 0.0));

        let (yaw1, _, _) = first.to_euler(bevy::prelude::EulerRot::YXZ);
        let (yaw2, _, _) = second.to_euler(bevy::prelude::EulerRot::YXZ);

        assert!(
            yaw2.abs() > yaw1.abs(),
            "Yaw should accumulate over consecutive look inputs"
        );
    }

    #[test]
    fn apply_look_delta_clamps_pitch() {
        let rotation =
            apply_look_delta(bevy::prelude::Quat::IDENTITY, Vec2::new(0.0, -1_000_000.0));
        let (_, pitch, _) = rotation.to_euler(bevy::prelude::EulerRot::YXZ);

        assert!(
            (-PITCH_LIMIT_RADIANS..=PITCH_LIMIT_RADIANS).contains(&pitch),
            "Pitch should be clamped within configured limits"
        );
    }

    #[test]
    fn vertical_mouse_input_changes_pitch() {
        let rotation = apply_look_delta(bevy::prelude::Quat::IDENTITY, Vec2::new(0.0, 120.0));
        let (yaw, pitch, _) = rotation.to_euler(bevy::prelude::EulerRot::YXZ);

        assert!(
            pitch.abs() > 0.0001,
            "Vertical mouse movement should affect pitch"
        );
        assert!(
            yaw.abs() < 0.0001,
            "Pure vertical mouse movement should not change yaw"
        );
    }

    // Regression: the production `get_player_actions()` bundle must bind `Look`
    // to mouse motion, otherwise mouse look is dead at runtime. The unit tests
    // above set `Action<Look>` by hand and therefore never exercised the
    // binding — this one feeds a real `MouseMotion` event through the production
    // bundle and asserts the `Look` action becomes non-zero.
    #[test]
    fn production_look_binding_consumes_mouse_motion() {
        use crate::inputs::{Look, PlayerActions, get_player_actions};
        use bevy::ecs::message::Messages;
        use bevy::input::InputPlugin;
        use bevy::input::mouse::MouseMotion;
        use bevy::prelude::{App, MinimalPlugins, Vec2};
        use bevy_enhanced_input::action::relationship::Actions;
        use bevy_enhanced_input::prelude::*;

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(InputPlugin);
        app.add_plugins(EnhancedInputPlugin)
            .add_input_context::<PlayerActions>()
            .finish();

        let entity = app.world_mut().spawn(get_player_actions()).id();

        app.world_mut()
            .resource_mut::<Messages<MouseMotion>>()
            .write(MouseMotion {
                delta: Vec2::new(20.0, 8.0),
            });

        app.update();

        // Read the Look action from its relationship child entity, not the player
        let value: Vec2 = {
            let actions = app.world().get::<Actions<PlayerActions>>(entity).unwrap();
            let mut found = None;
            for action_entity in actions.iter() {
                if let Some(action) = app.world().get::<Action<Look>>(*action_entity) {
                    found = Some(**action);
                    break;
                }
            }
            found.expect("Look action should exist as a relationship child")
        };
        assert!(
            value.length() > 0.0,
            "production Look binding should pick up mouse motion; got {:?}",
            value
        );
    }
}
