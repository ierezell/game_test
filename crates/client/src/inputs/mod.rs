pub mod window;

use bevy::prelude::{App, Commands, Entity, Plugin, Query, Res, Without};
use shared::inputs::{PlayerActions, get_player_actions};
use shared::protocol::PlayerId;

use crate::inputs::window::ClientWindowPlugin;

pub struct ClientInputPlugin;

impl Plugin for ClientInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ClientWindowPlugin);
    }
}

/// Spawn the input `Actions` bundle for the local player.
///
/// The local player is identified by matching the `PlayerId` component on its
/// entity against the `LocalPlayerId` *resource* (the same rule the camera uses
/// in `spawn_camera_when_local_player_id_added`). `LocalPlayerId` is a resource,
/// not a component, so a `With<LocalPlayerId>` query would match nothing and the
/// player would never receive input bindings.
pub fn spawn_local_player_input_actions(
    mut commands: Commands,
    local_player_id: Res<crate::LocalPlayerId>,
    query: Query<(Entity, &PlayerId), Without<PlayerActions>>,
) {
    for (entity, player_id) in query.iter() {
        if player_id.0.to_bits() == local_player_id.0 {
            commands.entity(entity).insert(get_player_actions());
        }
    }
}
