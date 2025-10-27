use crate::protocol::PlayerId;
use avian3d::prelude::{LinearVelocity, Position};
use bevy::prelude::*;
/// Minimal enemy component
#[derive(Component, Clone, Debug)]
pub struct Enemy;

/// System for basic enemy patrol and attack
pub fn enemy_behavior(
    mut query: Query<(&mut Position, &mut LinearVelocity), With<Enemy>>,
    player_query: Query<&Position, With<PlayerId>>,
) {
    for (pos, mut vel) in query.iter_mut() {
        // Find the first player (for simplicity)
        if let Some(player_pos) = player_query.iter().next() {
            let to_player = player_pos.0 - pos.0;
            let dist = to_player.length();
            if dist > 2.0 {
                // Move toward player
                vel.0 = to_player.normalize() * 2.0;
            } else {
                // Attack: stop and (in a real system) deal damage
                vel.0 = Vec3::ZERO;
                // Damage logic would go here
            }
        } else {
            // No player: stand still
            vel.0 = Vec3::ZERO;
        }
    }
}
