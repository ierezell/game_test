use avian3d::prelude::{Collider, LinearVelocity, SpatialQueryPipeline};
use bevy::prelude::{Entity, Query, Res, Time, Transform, With};
use leafwing_input_manager::prelude::ActionState;

use shared::{
    input::{FpsController, PlayerAction},
    protocol::PlayerId,
};

pub fn server_player_movement(
    time: Res<Time>,
    spatial_query: Res<SpatialQueryPipeline>,
    mut player_query: Query<
        (
            Entity,
            &ActionState<PlayerAction>,
            &mut FpsController,
            &mut Transform,
            &mut LinearVelocity,
            &Collider,
        ),
        With<PlayerId>,
    >,
) {
    for (entity, action_state, mut controller, mut transform, mut velocity, collider) in
        player_query.iter_mut()
    {
        let move_input = action_state.axis_pair(&PlayerAction::Move);
        if move_input.length_squared() > 0.0 {
            println!("Server received move input: {:?}", move_input);
        }
        shared::input::shared_player_movement(
            *time,
            spatial_query.clone(),
            entity,
            action_state,
            &mut controller,
            &mut transform,
            &mut velocity,
            collider,
        );
    }
}
