use avian3d::prelude::{LinearVelocity, Position, Rotation};
use bevy::{
    ecs::schedule::IntoScheduleConfigs,
    prelude::{
        App, Commands, Entity, FixedUpdate, Name, Plugin, Query, Vec2, With, debug, info, warn,
    },
    state::{condition::in_state, state::OnEnter},
};
use leafwing_input_manager::prelude::ActionState;
use lightyear::prelude::{
    ControlledBy, InterpolationTarget, NetworkTarget, PeerId, PredictionTarget, RemoteId,
    Replicate, server::ClientOf,
};
use shared::{
    entities::player::color_from_id,
    input::{PlayerAction, shared_player_movement},
    protocol::{LobbyState, PlayerColor, PlayerId},
};

use crate::ServerGameState;

pub struct ServerEntitiesPlugin;
impl Plugin for ServerEntitiesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            server_player_movement.run_if(in_state(ServerGameState::Playing)),
        );
        app.add_systems(OnEnter(ServerGameState::Playing), spawn_player_entities);
    }
}

fn spawn_player_entities(
    mut commands: Commands,
    lobby_state: Query<&LobbyState>,
    client_query: Query<(Entity, &RemoteId), With<ClientOf>>,
) {
    let lobby_data = lobby_state.single().unwrap();
    info!(
        "🚀 SERVER: Spawning player entities for {} players",
        lobby_data.players.len()
    );

    for player_id in &lobby_data.players {
        // Find the client entity for this player
        if let Some((client_entity, remote_id)) =
            client_query
                .iter()
                .find(|(_, remote_id)| match remote_id.0 {
                    lightyear::prelude::PeerId::Netcode(id) => id == *player_id,
                    _ => false,
                })
        {
            let color = color_from_id(*player_id);

            info!(
                "🎯 SERVER: Spawning player for remote_id: {:?} (client_id: {})",
                remote_id, player_id
            );

            let player = commands
                .spawn((
                    Name::new(format!("Player_{}", player_id)),
                    ActionState::<PlayerAction>::default(),
                    PlayerId(PeerId::Netcode(*player_id)),
                    PlayerColor(color),
                    Position::default(),
                    Rotation::default(),
                    LinearVelocity::default(),
                    ControlledBy {
                        owner: client_entity,
                        lifetime: Default::default(),
                    },
                    Replicate::to_clients(NetworkTarget::All),
                    PredictionTarget::to_clients(NetworkTarget::Single(remote_id.0)),
                    InterpolationTarget::to_clients(NetworkTarget::AllExceptSingle(remote_id.0)),
                ))
                .id();

            info!(
                "🌐 SERVER: Player entity {:?} created for client_id: {}",
                player, player_id
            );

            // Add physics bundle
            commands
                .entity(player)
                .insert(shared::entities::player::PlayerPhysicsBundle::default());

            info!(
                "✅ SERVER: Player entity {:?} fully configured for client_id: {}",
                player, player_id
            );
        } else {
            warn!(
                "❌ SERVER: Could not find client entity for player_id: {}",
                player_id
            );
        }
    }

    info!("🎮 SERVER: All players spawned, game is ready!");
}

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
