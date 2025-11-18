use avian3d::prelude::{LinearVelocity, Rotation};
use bevy::prelude::{Entity, Query, Vec2, With, debug};
use leafwing_input_manager::prelude::ActionState;

use shared::{
    input::{PlayerAction, shared_player_movement},
    protocol::PlayerId,
};

pub fn server_player_movement(
    mut player_query: Query<
        (
            Entity,
            &mut Rotation,
            &mut LinearVelocity,
            &ActionState<PlayerAction>,
        ),
        With<PlayerId>,
    >,
) {
    for (entity, mut rotation, mut velocity, action_state) in player_query.iter_mut() {
        let axis_pair = action_state.axis_pair(&PlayerAction::Move);
        if axis_pair != Vec2::ZERO || !action_state.get_pressed().is_empty() {
            debug!(
                "🖥️ SERVER: Processing movement for entity {:?} with axis {:?} and actions {:?}",
                entity,
                axis_pair,
                action_state.get_pressed()
            );
        }

        shared_player_movement(action_state, &mut rotation, &mut velocity);
    }
}
