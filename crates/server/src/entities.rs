use avian3d::prelude::{LinearVelocity, Position, Rotation};
use bevy::{
    ecs::schedule::IntoScheduleConfigs,
    prelude::{
        App, Commands, Entity, FixedUpdate, Name, Plugin, Query, Transform, Vec2, Vec3, With,
        debug, info, warn,
    },
    state::{condition::in_state, state::OnEnter},
};
use leafwing_input_manager::prelude::ActionState;
use lightyear::prelude::{
    ControlledBy, InterpolationTarget, NetworkTarget, PeerId, PredictionTarget, RemoteId,
    Replicate, server::ClientOf,
};
use shared::{
    entities::PhysicsBundle,
    entities::color_from_id,
    input::{PlayerAction, shared_player_movement},
    navigation::setup_patrol,
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
            let spawn_position = Vec3::new(spawn_x, 5.0, spawn_z);

            info!(
                "🎯 SERVER: Spawning player for remote_id: {:?} (client_id: {}) at position ({:.2}, {:.2}, {:.2})",
                remote_id, player_id, spawn_x, 5.0, spawn_z
            );

            let player = commands
                .spawn((
                    Name::new(format!("Player_{}", player_id)),
                    PlayerId(PeerId::Netcode(*player_id)),
                    PlayerColor(color),
                    Position::default(),
                    Rotation::default(),
                    Transform::from_translation(spawn_position),
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
            commands.entity(player).insert(PhysicsBundle::default());

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

fn spawn_patrolling_npc_entities(mut commands: Commands) {
    info!("🚀 SERVER: Spawning patrolling NPCs with navigation");
    
    // Spawn a patrolling enemy that moves in a rectangle
    let enemy = commands
        .spawn((
            Name::new("Patrol_Enemy_1"),
            Position::default(),
            Rotation::default(),
            Transform::from_translation(Vec3::new(0.0, 1.0, -10.0)),
            LinearVelocity::default(),
            Replicate::to_clients(NetworkTarget::All),
            InterpolationTarget::to_clients(NetworkTarget::All),
        ))
        .id();

    commands
        .entity(enemy)
        .insert(PhysicsBundle::default());

    // Setup patrol route for the enemy - rectangular path
    let patrol_points = vec![
        Vec3::new(-20.0, 1.0, -10.0),
        Vec3::new(20.0, 1.0, -10.0),
        Vec3::new(20.0, 1.0, 10.0),
        Vec3::new(-20.0, 1.0, 10.0),
    ];
    
    setup_patrol(&mut commands, enemy, patrol_points, 3.0, false);
    
    info!("✅ SERVER: Patrol enemy entity {:?} configured with navigation", enemy);
    
    // Spawn a guard enemy that patrols back and forth
    let guard = commands
        .spawn((
            Name::new("Guard_Enemy_1"),
            Position::default(),
            Rotation::default(),
            Transform::from_translation(Vec3::new(15.0, 1.0, 0.0)),
            LinearVelocity::default(),
            Replicate::to_clients(NetworkTarget::All),
            InterpolationTarget::to_clients(NetworkTarget::All),
        ))
        .id();

    commands
        .entity(guard)
        .insert(PhysicsBundle::default());

    // Setup ping-pong patrol for the guard
    let guard_patrol_points = vec![
        Vec3::new(10.0, 1.0, -5.0),
        Vec3::new(25.0, 1.0, -5.0),
    ];
    
    setup_patrol(&mut commands, guard, guard_patrol_points, 2.0, true); // ping-pong = true
    
    info!("✅ SERVER: Guard enemy entity {:?} configured with ping-pong patrol", guard);
    
    // Spawn a wandering bot with complex patrol route
    let bot = commands
        .spawn((
            Name::new("Wandering_Bot_1"),
            Position::default(),
            Rotation::default(),
            Transform::from_translation(Vec3::new(10.0, 1.0, 5.0)),
            LinearVelocity::default(),
            Replicate::to_clients(NetworkTarget::All),
            InterpolationTarget::to_clients(NetworkTarget::All),
        ))
        .id();

    commands
        .entity(bot)
        .insert(PhysicsBundle::default());

    // Setup a complex patrol route for the bot
    let bot_patrol_points = vec![
        Vec3::new(5.0, 1.0, 5.0),
        Vec3::new(-5.0, 1.0, 5.0),
        Vec3::new(-5.0, 1.0, -5.0),
        Vec3::new(5.0, 1.0, -5.0),
        Vec3::new(0.0, 1.0, 0.0), // Visit center
    ];
    
    setup_patrol(&mut commands, bot, bot_patrol_points, 2.5, false);
    
    info!("✅ SERVER: Wandering bot entity {:?} configured with complex patrol", bot);
    
    // Spawn a scout bot that patrols the perimeter
    let scout = commands
        .spawn((
            Name::new("Scout_Bot_1"),
            Position::default(),
            Rotation::default(),
            Transform::from_translation(Vec3::new(-15.0, 1.0, 10.0)),
            LinearVelocity::default(),
            Replicate::to_clients(NetworkTarget::All),
            InterpolationTarget::to_clients(NetworkTarget::All),
        ))
        .id();

    commands
        .entity(scout)
        .insert(PhysicsBundle::default());

    // Setup perimeter patrol for scout bot
    let scout_patrol_points = vec![
        Vec3::new(-30.0, 1.0, 30.0),
        Vec3::new(30.0, 1.0, 30.0),
        Vec3::new(30.0, 1.0, -30.0),
        Vec3::new(-30.0, 1.0, -30.0),
    ];
    
    setup_patrol(&mut commands, scout, scout_patrol_points, 4.0, false);
    
    info!("✅ SERVER: Scout bot entity {:?} configured with perimeter patrol", scout);
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
