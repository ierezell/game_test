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
        shared::input::shared_player_movement(
            time.clone(),
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
