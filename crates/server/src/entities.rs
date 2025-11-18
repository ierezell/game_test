use crate::input::server_player_movement;
use avian3d::prelude::{LinearVelocity, Position, Rotation};
use bevy::{
    ecs::schedule::IntoScheduleConfigs,
    prelude::{App, Commands, Entity, FixedUpdate, Name, Plugin, Query, Vec3, With, info, warn},
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
    info!(
        "🚀 SERVER: Spawning player entities for {} players",
        lobby_data.players.len()
    );

    let player_count = lobby_data.players.len() as f32;
    let spawn_radius = 3.0; // Radius for circular spawn pattern

    for (index, player_id) in lobby_data.players.iter().enumerate() {
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

            // Calculate spawn position in a circle to avoid stacking
            let angle = (index as f32) * 2.0 * std::f32::consts::PI / player_count;
            let spawn_x = spawn_radius * angle.cos();
            let spawn_z = spawn_radius * angle.sin();
            // Spawn players closer to the navmesh/floor so navmesh sampling succeeds
            let spawn_position = Vec3::new(spawn_x, 1.5, spawn_z);

            info!(
                "🎯 SERVER: Spawning player for remote_id: {:?} (client_id: {}) at position ({:.2}, {:.2}, {:.2})",
                remote_id, player_id, spawn_x, 5.0, spawn_z
            );

            let player = commands
                .spawn((
                    Name::new(format!("Player_{}", player_id)),
                    PlayerId(PeerId::Netcode(*player_id)),
                    PlayerColor(color),
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
                    // Use player-specific physics bundle at spawn so replication includes tuned components
                    PlayerPhysicsBundle::default(),
                ))
                .id();

            info!(
                "🌐 SERVER: Player entity {:?} created for client_id: {}",
                player, player_id
            );
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

fn spawn_patrolling_npc_entities(
    mut commands: Commands,
    obstacles: Query<&Position, With<NavigationObstacle>>,
) {
    info!("🚀 SERVER: Spawning patrolling NPCs with navigation");

    // Choose a safe spawn position away from obstacles
    // Obstacle is at (-10.0, 1.5, -15.0), so spawn at a clear area
    // Spawn slightly offset from first patrol point (-20.0, 1.0, -10.0) to ensure movement starts
    let initial_spawn = Vec3::new(-18.0, 1.0, -8.0);

    // Validate spawn position to avoid obstacles
    let validated_spawn = validate_spawn_position(initial_spawn, &obstacles, 0.5);

    // Spawn a patrolling enemy that moves in a rectangle (different path)
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

    // Setup patrol route for the enemy - rectangular path in a clear area
    // Avoid the obstacles at (-10.0, 1.5, -15.0), (15.0, 1.5, 10.0), etc.
    let original_patrol_points = vec![
        Vec3::new(-20.0, 1.0, -10.0), // Safe starting position
        Vec3::new(-5.0, 1.0, -10.0),  // Move east, avoiding obstacles
        Vec3::new(-5.0, 1.0, 5.0),    // Move north, clear area
        Vec3::new(-20.0, 1.0, 5.0),   // Move west, completing rectangle
    ];

    let validated_patrol_points: Vec<Vec3> = original_patrol_points
        .iter()
        .map(|&point| validate_spawn_position(point, &obstacles, 0.5))
        .collect();

    setup_patrol(&mut commands, enemy, validated_patrol_points, 3.0);

    info!(
        "✅ SERVER: Patrol enemy entity {:?} configured with navigation",
        enemy
    );

    // // Spawn a guard enemy that patrols a different area (ping-pong)
    // let guard = commands
    //     .spawn((
    //         Name::new("Guard_Enemy_1"),
    //         Position::new(Vec3::new(15.0, 1.0, -10.0)), // Start at first patrol point
    //         Rotation::default(),
    //         LinearVelocity::default(),
    //         Replicate::to_clients(NetworkTarget::All),
    //         InterpolationTarget::to_clients(NetworkTarget::All),
    //         CharacterMarker,
    //         NpcPhysicsBundle::default(),
    //     ))
    //     .id();

    // // Setup ping-pong patrol for the guard in a different corridor
    // let guard_patrol_points = vec![Vec3::new(15.0, 1.0, -10.0), Vec3::new(15.0, 1.0, 10.0)];

    // setup_patrol(&mut commands, guard, guard_patrol_points, 2.5, true); // ping-pong = true

    // info!(
    //     "✅ SERVER: Guard enemy entity {:?} configured with ping-pong patrol",
    //     guard
    // );

    // // Spawn a wandering bot with circular patrol pattern
    // let bot = commands
    //     .spawn((
    //         Name::new("Wandering_Bot_1"),
    //         Position::new(Vec3::new(-15.0, 1.0, 5.0)), // Start at first patrol point
    //         Rotation::default(),
    //         LinearVelocity::default(),
    //         Replicate::to_clients(NetworkTarget::All),
    //         InterpolationTarget::to_clients(NetworkTarget::All),
    //         CharacterMarker,
    //         NpcPhysicsBundle::default(),
    //     ))
    //     .id();

    // // Setup a circular patrol route for the bot - 8 points in a circle
    // let center = Vec3::new(-15.0, 1.0, 5.0);
    // let radius = 8.0;
    // let mut bot_patrol_points = Vec::new();
    // for i in 0..8 {
    //     let angle = (i as f32) * 2.0 * std::f32::consts::PI / 8.0;
    //     let x = center.x + radius * angle.cos();
    //     let z = center.z + radius * angle.sin();
    //     bot_patrol_points.push(Vec3::new(x, 1.0, z));
    // }

    // setup_patrol(&mut commands, bot, bot_patrol_points, 3.0, false);

    // info!(
    //     "✅ SERVER: Wandering bot entity {:?} configured with complex patrol",
    //     bot
    // );

    // // Spawn a scout bot that does ping-pong diagonal patrol
    // let scout = commands
    //     .spawn((
    //         Name::new("Scout_Bot_1"),
    //         Position::new(Vec3::new(-20.0, 1.0, -20.0)), // Start at first patrol point
    //         Rotation::default(),
    //         LinearVelocity::default(),
    //         Replicate::to_clients(NetworkTarget::All),
    //         InterpolationTarget::to_clients(NetworkTarget::All),
    //         CharacterMarker,
    //         NpcPhysicsBundle::default(),
    //     ))
    //     .id();

    // // Setup diagonal ping-pong patrol for scout bot
    // let scout_patrol_points = vec![
    //     Vec3::new(-20.0, 1.0, -20.0),
    //     Vec3::new(20.0, 1.0, 20.0),
    // ];

    // setup_patrol(&mut commands, scout, scout_patrol_points, 4.0, true); // ping-pong = true

    // info!(
    //     "✅ SERVER: Scout bot entity {:?} configured with perimeter patrol",
    //     scout
    // );
}
