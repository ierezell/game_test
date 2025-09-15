use crate::protocol::PlayerId;

use avian3d::prelude::{LinearVelocity, Position, Rotation};
use bevy::prelude::{
    App, Commands, Component, Event, EventReader, EventWriter, Plugin, Quat, Query, Reflect, Res,
    ResMut, Time, Transform, Update, Vec2, Vec3, With, debug, default,
};

use leafwing_input_manager::Actionlike;
use leafwing_input_manager::InputControlKind::DualAxis;
use leafwing_input_manager::prelude::ActionState;
use leafwing_input_manager::prelude::InputManagerPlugin as LeafwingInputPlugin;
use lightyear::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy, Hash, Reflect, Actionlike)]
pub enum NetworkedInput {
    #[actionlike(DualAxis)]
    Move,
    PrimaryFire,
    Reload,
    Jump,
    Run,
    Look,
    Sprint,
    Crouch,
}

impl Default for NetworkedInput {
    fn default() -> Self {
        Self::Move
    }
}

pub const PLAYER_SPEED: f32 = 5.0;
pub const PLAYER_SPRINT_SPEED: f32 = 8.0;
pub const PLAYER_CROUCH_SPEED: f32 = 2.0;

pub const MOUSE_SENSITIVITY: f32 = 0.002;

pub const STAMINA_DRAIN_RATE: f32 = 20.0;
pub const STAMINA_REGEN_RATE: f32 = 15.0;
pub const MAX_STAMINA: f32 = 100.0;

#[derive(Component, Debug, Clone, Reflect)]
pub struct Stamina {
    pub current: f32,
}

impl Default for Stamina {
    fn default() -> Self {
        Self {
            current: MAX_STAMINA,
        }
    }
}

pub fn shared_player_movement(
    mut player_query: Query<(
        &mut Position,
        &mut Rotation,
        &ActionState<NetworkedInput>,
        &mut Stamina,
    )>,
    time: Res<Time>,
) {
    for (mut position, mut rotation, action_state, mut stamina) in player_query.iter_mut() {
        let mut movement = Vec3::ZERO;

        let before_pos = position.0;

        let move_data = action_state.axis_pair(&NetworkedInput::Move);
        debug!(
            "shared_player_movement: move_data={:?}, look_data={:?}",
            move_data,
            action_state.axis_pair(&NetworkedInput::Look)
        );
        if move_data != Vec2::ZERO {
            movement.x = move_data.x;
            movement.z = move_data.y;
        }

        let is_sprinting = action_state.pressed(&NetworkedInput::Sprint) && stamina.current > 0.0;
        let is_crouching = action_state.pressed(&NetworkedInput::Crouch);

        let speed = if is_sprinting {
            PLAYER_SPRINT_SPEED
        } else if is_crouching {
            PLAYER_CROUCH_SPEED
        } else {
            PLAYER_SPEED
        };

        if movement != Vec3::ZERO {
            let forward = rotation.0.mul_vec3(Vec3::NEG_Z);
            let right = rotation.0.mul_vec3(Vec3::X);
            let movement_world =
                (right * movement.x + forward * movement.z) * speed * time.delta_secs();
            position.0 += movement_world;
        }

        let look_delta = action_state.axis_pair(&NetworkedInput::Look);
        if look_delta != Vec2::ZERO {
            let yaw_delta = -look_delta.x * MOUSE_SENSITIVITY;
            let yaw_rotation = Quat::from_rotation_y(yaw_delta);
            rotation.0 = yaw_rotation * rotation.0;
        }

        if is_sprinting {
            stamina.current -= STAMINA_DRAIN_RATE * time.delta_secs();
            stamina.current = stamina.current.max(0.0);
        } else {
            stamina.current += STAMINA_REGEN_RATE * time.delta_secs();
            stamina.current = stamina.current.min(MAX_STAMINA);
        }

        debug!(
            "shared_player_movement: before_pos={:?} after_pos={:?}",
            before_pos, position.0
        );
    }
}
