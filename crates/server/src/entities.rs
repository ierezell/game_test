use crate::input::ServerInputPlugin;
use avian3d::prelude::{LinearVelocity, Position, Rotation};
use bevy::{
    ecs::schedule::IntoScheduleConfigs,
    prelude::{
        AmbientLight, App, Assets, Color, Commands, Cuboid, default, Entity,
        FixedUpdate, Mesh, Mesh3d, MeshMaterial3d, Name, Plane3d, Plugin, Query,
        ResMut, StandardMaterial, Transform, Vec2, Vec3, With, info,
    },
    state::{condition::in_state, state::OnEnter},
};
use leafwing_input_manager::prelude::ActionState;
use shared::input::PlayerAction;
use shared::movement::{GroundState, MovementConfig};
use shared::camera::FpsCamera;

use lightyear::prelude::{
    Connected, ControlledBy, InterpolationTarget, NetworkTarget, PeerId, PredictionTarget,
    RemoteId, Replicate, server::ClientOf,
};
use shared::{
    components::{
        health::{Health, Respawnable},
        weapons::Gun,
    },
    entities::{NpcPhysicsBundle, PlayerPhysicsBundle, color_from_id},
    navigation::{NavigationObstacle, setup_patrol, validate_spawn_position},
    protocol::{CharacterMarker, LobbyState, PlayerColor, PlayerId},
    components::flashlight::PlayerFlashlight,
};

use crate::ServerGameState;

pub struct ServerEntitiesPlugin;
impl Plugin for ServerEntitiesPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ServerInputPlugin);
        app.add_systems(
            FixedUpdate,
            (
                spawn_late_joining_players,
                handle_player_death,
                handle_player_respawn,
            )
                .run_if(in_state(ServerGameState::Playing)),
        );
        app.add_systems(
            OnEnter(ServerGameState::Playing),
            (
                generate_and_build_level,
                (spawn_player_entities, spawn_patrolling_npc_entities),
            )
                .chain(),
        );
    }
}

/// Generate the procedural level and build its visual representation
///
/// This runs FIRST when entering the Playing state, before spawning players
fn generate_and_build_level(
    mut commands: Commands,
    meshes: Option<ResMut<Assets<Mesh>>>,
    materials: Option<ResMut<Assets<StandardMaterial>>>,
) {
    info!("🎮 SIMPLIFIED MODE: Spawning basic test environment");

    // Only add visual components if rendering is available (not headless)
    if let (Some(mut meshes), Some(mut materials)) = (meshes, materials) {
        // Simple ground plane
        commands.spawn((
            Mesh3d(meshes.add(Mesh::from(Plane3d::new(Vec3::Y, Vec2::new(50.0, 50.0))))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.3, 0.3, 0.35),
                perceptual_roughness: 0.9,
                ..default()
            })),
            Transform::from_xyz(0.0, 0.0, 0.0),
            Replicate::to_clients(NetworkTarget::All),
            Name::new("Ground"),
        ));

        // Simple cube as reference object
        commands.spawn((
            Mesh3d(meshes.add(Mesh::from(Cuboid::new(2.0, 2.0, 2.0)))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.8, 0.2, 0.2),
                unlit: false,  // PBR lighting - only visible when lit
                ..default()
            })),
            Transform::from_xyz(5.0, 1.0, 5.0),
            Replicate::to_clients(NetworkTarget::All),
            Name::new("Test Cube"),
        ));

        // NO LIGHTS - Pure darkness except for player flashlights
        commands.insert_resource(AmbientLight {
            color: Color::BLACK,
            brightness: 0.0,
            affects_lightmapped_meshes: false,
        });

        info!("✅ Simple test environment ready with visuals (flashlight-only lighting)");
    } else {
        info!("✅ Simple test environment ready (headless mode)");
    }
}

fn spawn_player_entities(
    mut commands: Commands,
    lobby_state: Query<&LobbyState>,
    client_query: Query<(Entity, &RemoteId), With<ClientOf>>,
) {
    let lobby_data = lobby_state.single().unwrap();
    let player_count = lobby_data.players.len() as f32;

    // Spawn players in a circle
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
            let spawn_position = Vec3::new(
                spawn_radius * angle.cos(),
                3.5,
                spawn_radius * angle.sin(),
            );

            println!(
                "DEBUG: Spawning player entity for ID: {} at {:?}",
                player_id, spawn_position
            );

            let _player = commands
                .spawn((
                    Name::new(format!("Player_{}", player_id)),
                    PlayerId(PeerId::Netcode(*player_id)),
                    PlayerColor(color_from_id(*player_id)),
                    Rotation::default(),
                    Position::new(spawn_position),
                    LinearVelocity::default(),
                    Health::basic(),
                    Respawnable::new(3.0), // 3 second respawn delay
                    Gun::default(),
                    PlayerFlashlight::new(), // Add flashlight to player
                    ControlledBy {
                        owner: client_entity,
                        lifetime: Default::default(),
                    },
                    Replicate::to_clients(NetworkTarget::All),
                    PredictionTarget::to_clients(NetworkTarget::Single(remote_id.0)),
                    InterpolationTarget::to_clients(NetworkTarget::AllExceptSingle(remote_id.0)),
                ))
                .insert((
                    // Refactored modular movement components
                    MovementConfig::default(),
                    // FpsCamera: Server spawns default, client replicates actual values
                    // Server needs this for apply_movement system to calculate wish direction
                    FpsCamera::default(),
                    GroundState::default(),
                ))
                .insert((
                    CharacterMarker,
                    PlayerPhysicsBundle::default(),
                    ActionState::<PlayerAction>::default(),
                    leafwing_input_manager::prelude::InputMap::<PlayerAction>::default(),
                ))
                .id();
        } else {
            println!(
                "DEBUG: Could not find client entity for player ID: {}",
                player_id
            );
            // Dump available clients
            for (e, r) in client_query.iter() {
                println!("DEBUG: Available Client: {:?} with RemoteId: {:?}", e, r);
            }
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
            Health::basic(),
            Replicate::to_clients(NetworkTarget::All),
            InterpolationTarget::to_clients(NetworkTarget::All),
            CharacterMarker,
            NpcPhysicsBundle::default(),
        ))
        .id();

    let original_patrol_points = [
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

/// Spawn player entities for clients that join after the game has already started
fn spawn_late_joining_players(
    mut commands: Commands,
    lobby_state: Query<&LobbyState>,
    client_query: Query<(Entity, &RemoteId), (With<ClientOf>, With<Connected>)>,
    existing_players: Query<&PlayerId>,
) {
    let Ok(lobby_data) = lobby_state.single() else {
        return;
    };

    // Check each connected client to see if they need a player entity
    for (client_entity, remote_id) in client_query.iter() {
        let player_id_bits = match remote_id.0 {
            PeerId::Netcode(id) => id,
            _ => continue,
        };

        // Check if this client is in the lobby
        if !lobby_data.players.contains(&player_id_bits) {
            continue;
        }

        // Check if this player already has an entity
        let player_exists = existing_players.iter().any(|pid| match pid.0 {
            PeerId::Netcode(id) => id == player_id_bits,
            _ => false,
        });

        if !player_exists {
            // Find a spawn position - use index based on when they joined
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

            println!(
                "DEBUG: Spawning late-joining player entity for ID: {} at {:?}",
                player_id_bits, spawn_position
            );

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
                    PlayerFlashlight::new(), // Add flashlight to late-joining player
                    ControlledBy {
                        owner: client_entity,
                        lifetime: Default::default(),
                    },
                    Replicate::to_clients(NetworkTarget::All),
                    PredictionTarget::to_clients(NetworkTarget::Single(remote_id.0)),
                    InterpolationTarget::to_clients(NetworkTarget::AllExceptSingle(remote_id.0)),
                ))
                .insert((
                    // Refactored modular movement components
                    MovementConfig::default(),
                    // FpsCamera: Server spawns default, client replicates actual values
                    FpsCamera::default(),
                    GroundState::default(),
                ))
                .insert((
                    CharacterMarker,
                    PlayerPhysicsBundle::default(),
                    ActionState::<PlayerAction>::default(),
                    leafwing_input_manager::prelude::InputMap::<PlayerAction>::default(),
                ));
        }
    }
}

/// Handle player death - despawn the entity when health reaches 0
fn handle_player_death(
    mut commands: Commands,
    player_query: Query<(Entity, &Health, &PlayerId), With<CharacterMarker>>,
) {
    for (entity, health, player_id) in player_query.iter() {
        if health.is_dead {
            info!(
                "Player {:?} has died, despawning entity {:?}",
                player_id, entity
            );
            // Despawn the player entity - they'll respawn after the delay
            commands.entity(entity).despawn();
        }
    }
}

/// Handle player respawn - respawn players after their respawn delay
fn handle_player_respawn() {
    // This system will spawn players who are in the lobby but don't have entities
    // The spawn_late_joining_players system already handles this logic
    // So dead players will automatically respawn through that system
}
