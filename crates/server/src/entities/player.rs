use avian3d::prelude::{LinearVelocity, Position, Rotation};
use bevy::prelude::{Commands, Entity, Name, Query, Vec3, With, info};
use leafwing_input_manager::prelude::ActionState;

use lightyear::prelude::{
    Connected, ControlledBy, InterpolationTarget, NetworkTarget, PeerId, PredictionTarget,
    RemoteId, Replicate, server::ClientOf,
};
use shared::debug::debug_println;
use shared::inputs::input::PlayerAction;
use shared::inputs::movement::GroundState;
use shared::{
    components::{
        flashlight::PlayerFlashlight,
        health::{Health, Respawnable},
        weapons::Gun,
    },
    entities::{PlayerPhysicsBundle, color_from_id},
    protocol::{CharacterMarker, LobbyState, PlayerColor, PlayerId},
};

pub fn spawn_player_entities(
    mut commands: Commands,
    lobby_state: &Query<&LobbyState>,
    client_query: &Query<(Entity, &RemoteId), With<ClientOf>>,
) {
    let Ok(lobby_data) = lobby_state.single() else {
        return;
    };

    let player_count = lobby_data.players.len() as f32;
    let spawn_radius = 3.0;

    for (index, player_id) in lobby_data.players.iter().enumerate() {
        if let Some((client_entity, remote_id)) =
            client_query
                .iter()
                .find(|(_, remote_id)| match remote_id.0 {
                    PeerId::Netcode(id) => id == *player_id,
                    _ => false,
                })
        {
            let angle = (index as f32) * 2.0 * std::f32::consts::PI / player_count;
            let spawn_position =
                Vec3::new(spawn_radius * angle.cos(), 3.5, spawn_radius * angle.sin());

            debug_println(format_args!(
                "DEBUG: Spawning player entity for ID: {} at {:?}",
                player_id, spawn_position
            ));

            commands
                .spawn((
                    Name::new(format!("Player_{}", player_id)),
                    PlayerId(PeerId::Netcode(*player_id)),
                    PlayerColor(color_from_id(*player_id)),
                    Rotation::default(),
                    Position::new(spawn_position),
                    LinearVelocity::default(),
                    Health::basic(),
                    Respawnable::new(3.0),
                    Gun::default(),
                    PlayerFlashlight::new(),
                    ControlledBy {
                        owner: client_entity,
                        lifetime: Default::default(),
                    },
                    Replicate::to_clients(NetworkTarget::All),
                    PredictionTarget::to_clients(NetworkTarget::Single(remote_id.0)),
                    InterpolationTarget::to_clients(NetworkTarget::AllExceptSingle(remote_id.0)),
                ))
                .insert(GroundState::default())
                .insert((
                    CharacterMarker,
                    PlayerPhysicsBundle::default(),
                    ActionState::<PlayerAction>::default(),
                    leafwing_input_manager::prelude::InputMap::<PlayerAction>::default(),
                ));
        } else {
            debug_println(format_args!(
                "DEBUG: Could not find client entity for player ID: {}",
                player_id
            ));
            for (entity, remote) in client_query.iter() {
                debug_println(format_args!(
                    "DEBUG: Available Client: {:?} with RemoteId: {:?}",
                    entity, remote
                ));
            }
        }
    }
}

/// Spawn player entities for clients that join after the game has already started.
pub fn spawn_late_joining_players(
    mut commands: Commands,
    lobby_state: Query<&LobbyState>,
    client_query: Query<(Entity, &RemoteId), (With<ClientOf>, With<Connected>)>,
    existing_players: Query<&PlayerId>,
) {
    let Ok(lobby_data) = lobby_state.single() else {
        return;
    };

    for (client_entity, remote_id) in client_query.iter() {
        let player_id_bits = match remote_id.0 {
            PeerId::Netcode(id) => id,
            _ => continue,
        };

        if !lobby_data.players.contains(&player_id_bits) {
            continue;
        }

        let player_exists = existing_players.iter().any(|pid| match pid.0 {
            PeerId::Netcode(id) => id == player_id_bits,
            _ => false,
        });

        if !player_exists {
            let index = lobby_data
                .players
                .iter()
                .position(|&id| id == player_id_bits)
                .unwrap_or(0);
            let player_count = lobby_data.players.len() as f32;
            let spawn_radius = 3.0;
            let angle = (index as f32) * 2.0 * std::f32::consts::PI / player_count;
            let spawn_position =
                Vec3::new(spawn_radius * angle.cos(), 3.5, spawn_radius * angle.sin());

            debug_println(format_args!(
                "DEBUG: Spawning late-joining player entity for ID: {} at {:?}",
                player_id_bits, spawn_position
            ));

            commands
                .spawn((
                    Name::new(format!("Player_{}", player_id_bits)),
                    PlayerId(PeerId::Netcode(player_id_bits)),
                    PlayerColor(color_from_id(player_id_bits)),
                    Rotation::default(),
                    Position::new(spawn_position),
                    LinearVelocity::default(),
                    Health::basic(),
                    Respawnable::new(3.0),
                    Gun::default(),
                    PlayerFlashlight::new(),
                    ControlledBy {
                        owner: client_entity,
                        lifetime: Default::default(),
                    },
                    Replicate::to_clients(NetworkTarget::All),
                    PredictionTarget::to_clients(NetworkTarget::Single(remote_id.0)),
                    InterpolationTarget::to_clients(NetworkTarget::AllExceptSingle(remote_id.0)),
                ))
                .insert(GroundState::default())
                .insert((
                    CharacterMarker,
                    PlayerPhysicsBundle::default(),
                    ActionState::<PlayerAction>::default(),
                    leafwing_input_manager::prelude::InputMap::<PlayerAction>::default(),
                ));
        }
    }
}

/// Handle player death by despawning entities with empty health.
pub fn handle_player_death(
    mut commands: Commands,
    player_query: Query<(Entity, &Health, &PlayerId), With<CharacterMarker>>,
) {
    for (entity, health, player_id) in player_query.iter() {
        if health.is_dead {
            info!(
                "Player {:?} has died, despawning entity {:?}",
                player_id, entity
            );
            commands.entity(entity).despawn();
        }
    }
}
