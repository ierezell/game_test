use avian3d::prelude::Rotation;
use bevy::prelude::{EulerRot, Quat, Query, Vec2, With};
use leafwing_input_manager::prelude::ActionState;

use crate::{
    inputs::input::{PITCH_LIMIT_RADIANS, PlayerAction},
    protocol::{CharacterMarker, PlayerId},
};
const LOOK_DEADZONE_SQUARED: f32 = 0.000001;
pub const MOUSE_SENSIVITY: f32 = 0.002;

pub fn get_mouse_look_delta(action_state: &ActionState<PlayerAction>) -> Vec2 {
    let look_input = action_state.axis_pair(&PlayerAction::Look);
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

pub fn update_player_rotation_from_input(
    mut player_query: Query<
        (&ActionState<PlayerAction>, &mut Rotation),
        (With<CharacterMarker>, With<PlayerId>),
    >,
) {
    for (action_state, mut rotation) in player_query.iter_mut() {
        if action_state.disabled() {
            continue;
        }

        let mouse_delta = get_mouse_look_delta(action_state);
        if mouse_delta != Vec2::ZERO {
            rotation.0 = apply_look_delta(rotation.0, mouse_delta);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{apply_look_delta, get_mouse_look_delta};
    use crate::inputs::input::{PITCH_LIMIT_RADIANS, PlayerAction};
    use crate::protocol::{CharacterMarker, PlayerId};
    use avian3d::prelude::Rotation;
    use bevy::prelude::{App, Update, Vec2};
    use lightyear::prelude::{Controlled, PeerId, Predicted};
    use leafwing_input_manager::prelude::ActionState;

    #[test]
    fn look_delta_applies_deadzone() {
        let mut action_state = ActionState::<PlayerAction>::default();
        action_state.set_axis_pair(&PlayerAction::Look, Vec2::new(0.0001, 0.0001));

        let delta = get_mouse_look_delta(&action_state);
        assert_eq!(delta, Vec2::ZERO);
    }

    #[test]
    fn look_delta_preserves_valid_input() {
        let mut action_state = ActionState::<PlayerAction>::default();
        let expected = Vec2::new(0.25, -0.75);
        action_state.set_axis_pair(&PlayerAction::Look, expected);

        let delta = get_mouse_look_delta(&action_state);
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
        let rotation = apply_look_delta(bevy::prelude::Quat::IDENTITY, Vec2::new(0.0, -1_000_000.0));
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

        assert!(pitch.abs() > 0.0001, "Vertical mouse movement should affect pitch");
        assert!(yaw.abs() < 0.0001, "Pure vertical mouse movement should not change yaw");
    }

    #[test]
    fn look_updates_server_style_entity_without_predicted_controlled_markers() {
        let mut app = App::new();
        app.add_systems(Update, super::update_player_rotation_from_input);

        let mut action_state = ActionState::<PlayerAction>::default();
        action_state.enable();
        action_state.set_axis_pair(&PlayerAction::Look, Vec2::new(120.0, 0.0));

        let player = app
            .world_mut()
            .spawn((
                PlayerId(PeerId::Netcode(1)),
                CharacterMarker,
                Rotation::default(),
                action_state,
            ))
            .id();

        app.update();

        let updated_rotation = app
            .world()
            .get::<Rotation>(player)
            .expect("player should still have a rotation")
            .0;

        let dot = updated_rotation.dot(bevy::prelude::Quat::IDENTITY).abs();
        assert!(
            dot < 0.999,
            "Rotation should change for server-style entity without prediction markers, dot={}",
            dot
        );
    }

    #[test]
    fn look_updates_each_entity_from_its_own_action_state() {
        let mut app = App::new();
        app.add_systems(Update, super::update_player_rotation_from_input);

        let mut turning = ActionState::<PlayerAction>::default();
        turning.enable();
        turning.set_axis_pair(&PlayerAction::Look, Vec2::new(80.0, 0.0));

        let mut idle = ActionState::<PlayerAction>::default();
        idle.enable();
        idle.set_axis_pair(&PlayerAction::Look, Vec2::ZERO);

        let turning_player = app
            .world_mut()
            .spawn((
                PlayerId(PeerId::Netcode(10)),
                Predicted,
                Controlled,
                CharacterMarker,
                Rotation::default(),
                turning,
            ))
            .id();

        let idle_player = app
            .world_mut()
            .spawn((
                PlayerId(PeerId::Netcode(11)),
                Predicted,
                Controlled,
                CharacterMarker,
                Rotation::default(),
                idle,
            ))
            .id();

        app.update();

        let turning_rotation = app
            .world()
            .get::<Rotation>(turning_player)
            .expect("turning player should have rotation")
            .0;
        let idle_rotation = app
            .world()
            .get::<Rotation>(idle_player)
            .expect("idle player should have rotation")
            .0;

        let turning_dot = turning_rotation.dot(bevy::prelude::Quat::IDENTITY).abs();
        let idle_dot = idle_rotation.dot(bevy::prelude::Quat::IDENTITY).abs();

        assert!(
            turning_dot < 0.999,
            "Turning player should rotate, dot={}",
            turning_dot
        );
        assert!(
            idle_dot > 0.999,
            "Idle player should remain near identity rotation, dot={}",
            idle_dot
        );
    }
}
