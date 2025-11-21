use crate::input::server_player_movement;
use avian3d::prelude::{LinearVelocity, Position, Rotation};
use bevy::{
    ecs::schedule::IntoScheduleConfigs,
    prelude::{App, Commands, Entity, FixedUpdate, Name, Plugin, Query, Vec3, With, info},
    state::{condition::in_state, state::OnEnter},
};

use lightyear::prelude::{
    ControlledBy, InterpolationTarget, NetworkTarget, PeerId, PredictionTarget, RemoteId,
    Replicate, server::ClientOf,
};
use shared::{
    entities::{NpcPhysicsBundle, PlayerPhysicsBundle, color_from_id},
    navigation::{NavigationObstacle, setup_patrol, validate_spawn_position},
    protocol::{CharacterMarker, LobbyState, PlayerColor, PlayerId},
};

use crate::ServerGameState;

pub struct ServerEntitiesPlugin;
impl Plugin for ServerEntitiesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            server_player_movement.run_if(in_state(ServerGameState::Playing)),
        );
        app.add_systems(
            OnEnter(ServerGameState::Playing),
            (spawn_player_entities, spawn_patrolling_npc_entities),
        );
    }
}

fn spawn_player_entities(
    mut commands: Commands,
    lobby_state: Query<&LobbyState>,
    client_query: Query<(Entity, &RemoteId), With<ClientOf>>,
) {
    let lobby_data = lobby_state.single().unwrap();
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

            let _player = commands
                .spawn((
                    Name::new(format!("Player_{}", player_id)),
                    PlayerId(PeerId::Netcode(*player_id)),
                    PlayerColor(color_from_id(*player_id)),
                    Rotation::default(),
                    Position::new(spawn_position),
                    LinearVelocity::default(),
                    ControlledBy {
                        owner: client_entity,
                        lifetime: Default::default(),
                    },
                    Replicate::to_clients(NetworkTarget::All),
                    PredictionTarget::to_clients(NetworkTarget::Single(remote_id.0)),
                    InterpolationTarget::to_clients(NetworkTarget::AllExceptSingle(remote_id.0)),
                    CharacterMarker,
                    PlayerPhysicsBundle::default(),
                ))
                .id();
        }
    }
}

fn spawn_patrolling_npc_entities(
    mut commands: Commands,
    obstacles: Query<&Position, With<NavigationObstacle>>,
) {
    let initial_spawn = Vec3::new(-18.0, 1.0, -8.0);
    // Obstacles are created at create_static_level, so it's before this system runs
    let validated_spawn = validate_spawn_position(initial_spawn, &obstacles, 0.5);
    let enemy = commands
        .spawn((
            Name::new("Patrol_Enemy_1"),
            Position::new(validated_spawn),
            Rotation::default(),
            LinearVelocity::default(),
            Replicate::to_clients(NetworkTarget::All),
            InterpolationTarget::to_clients(NetworkTarget::All),
            CharacterMarker,
            NpcPhysicsBundle::default(),
        ))
        .id();

    let original_patrol_points = vec![
        Vec3::new(-20.0, 1.0, -10.0),
        Vec3::new(-5.0, 1.0, -10.0),
        Vec3::new(-5.0, 1.0, 5.0),
        Vec3::new(-20.0, 1.0, 5.0),
    ];

    let validated_patrol_points: Vec<Vec3> = original_patrol_points
        .iter()
        .map(|&point| validate_spawn_position(point, &obstacles, 0.5))
        .collect();

    setup_patrol(&mut commands, enemy, validated_patrol_points, 3.0);
}
