use avian3d::prelude::{LinearVelocity, Position, Rotation};
use bevy::prelude::{
    Add, App, Commands, Entity, FixedUpdate, Name, On, Plugin, Query, Single, Vec2, Vec3, With,
    Without, debug, info,
};
use leafwing_input_manager::prelude::ActionState;
use lightyear::connection::client::Connected;
use lightyear::connection::server::Started;

use lightyear::prelude::server::{ClientOf, Server};
use lightyear::prelude::{
    Confirmed, ControlledBy, InterpolationTarget, LocalTimeline, NetworkTarget, NetworkTimeline,
    Predicted, PredictionTarget, RemoteId, Replicate,
};
use shared::entity_implementations::EnhancedPlayerPhysicsBundle;
use shared::entity_implementations::{EnemyEntity, EnemyType};
use shared::entity_traits::PhysicsProvider;
use shared::health::{Health, Respawnable};
use shared::input::{PLAYER_CAPSULE_HEIGHT, PlayerAction, shared_player_movement_with_stamina};
use shared::navigation_pathfinding::{NavigationMeshMarker, add_navigation_agent_with_speed};
use shared::protocol::{PlayerColor, PlayerId};
use shared::scene::{
    FLOOR_THICKNESS, FloorMarker, ROOM_SIZE, WALL_HEIGHT, WALL_THICKNESS, WallMarker,
    add_floor_physics, add_wall_physics, color_from_id,
};
use shared::stamina::{StaminaEffects, add_stamina_to_player};
use shared::weapons::add_weapon_holder;

pub struct ServerGameplayPlugin;

impl Plugin for ServerGameplayPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(setup_scene_on_server_start);
        app.add_observer(handle_connected);
        app.add_systems(FixedUpdate, server_player_movement);
        app.add_systems(FixedUpdate, debug_player_position);
        app.add_observer(add_floor_physics);
        app.add_observer(add_wall_physics);
        app.add_observer(spawn_enemies_on_server_start);
        app.add_observer(setup_navigation_on_server_start);
    }
}

fn debug_player_position(
    query: Query<(&Name, &Position, &LinearVelocity), With<PlayerId>>,
    timeline: Single<&LocalTimeline, With<Server>>,
) {
    for (name, pos, vel) in query.iter() {
        debug!(
            "S:{:?} pos:{:?} vel:{:?} tick:{:?}",
            name,
            pos,
            vel,
            timeline.tick()
        );
    }
}

fn handle_connected(
    trigger: On<Add, Connected>,
    query: Query<&RemoteId, With<ClientOf>>,
    mut commands: Commands,
) {
    let Ok(client_id) = query.get(trigger.entity) else {
        return;
    };
    let peer_id = client_id.0;
    info!("Client connected with client-id {client_id:?}. Spawning player entity.");

    let color = color_from_id(client_id.to_bits());
    let angle: f32 = client_id.to_bits() as f32 * 6.28 / 4.0; // Distribute around circle
    let x = 5.0 * angle.cos();
    let z = 5.0 * angle.sin();
    let y = PLAYER_CAPSULE_HEIGHT + 10.0;

    info!(
        "🎯 Setting up prediction target for client_id: {:?} (peer_id: {})",
        client_id, peer_id
    );
    info!(
        "🔍 Client entity: {:?}, RemoteId bits: {}",
        trigger.entity,
        client_id.to_bits()
    );

    let player = commands
        .spawn((
            // Replicated
            Name::new(format!("Player_{}", client_id.to_bits())),
            PlayerId(peer_id),
            LinearVelocity::default(),
            Position(Vec3::new(x, y, z)),
            Rotation::default(),
            PlayerColor(color),
            // Health system
            Health::with_regeneration(100.0, 10.0, 5.0), // 100 HP, 10 HP/s regen, 5s delay
            Respawnable::new(5.0),                       // 5 second respawn time
            // Lightyear config
            ControlledBy {
                owner: trigger.entity,
                lifetime: Default::default(),
            },
            Replicate::to_clients(NetworkTarget::All),
            PredictionTarget::to_clients(NetworkTarget::Single(peer_id)),
            InterpolationTarget::to_clients(NetworkTarget::AllExceptSingle(peer_id)),
            // Should not be replicated
            EnhancedPlayerPhysicsBundle::default(),
        ))
        .id();

    // Add weapon holder to player
    add_weapon_holder(&mut commands, player);

    // Add stamina system to player
    add_stamina_to_player(&mut commands, player);

    info!("Created player entity {player:?} for client {client_id:?}");
    info!(
        "🔍 ControlledBy owner set to client entity: {:?}",
        trigger.entity
    );
}

pub fn server_player_movement(
    mut player_query: Query<
        (
            Entity,
            &mut Rotation,
            &mut LinearVelocity,
            &ActionState<PlayerAction>,
            Option<&StaminaEffects>,
        ),
        // Based on lightyear examples - avoid applying movement to predicted/confirmed entities
        // to prevent conflicts in host-server mode
        (
            With<PlayerId>,
            Without<Predicted>,
            Without<Confirmed<Position>>,
        ),
    >,
) {
    for (entity, mut rotation, mut velocity, action_state, stamina_effects) in
        player_query.iter_mut()
    {
        let axis_pair = action_state.axis_pair(&PlayerAction::Move);
        if axis_pair != Vec2::ZERO || !action_state.get_pressed().is_empty() {
            debug!(
                "🖥️ SERVER: Processing movement for entity {:?} with axis {:?} and actions {:?}",
                entity,
                axis_pair,
                action_state.get_pressed()
            );
        }

        shared_player_movement_with_stamina(
            action_state,
            &mut rotation,
            &mut velocity,
            stamina_effects,
        );
    }
}

fn setup_scene_on_server_start(_trigger: On<Add, Started>, mut commands: Commands) {
    info!("Setting up scene on server (after server started)");

    commands.spawn((
        Name::new("Floor"),
        FloorMarker,
        Position(Vec3::new(0.0, -FLOOR_THICKNESS / 2.0, 0.0)),
        Rotation::default(),
        Replicate::to_clients(NetworkTarget::All),
    ));

    let wall_positions = [
        (
            Vec3::new(
                ROOM_SIZE / 2.0 + WALL_THICKNESS / 2.0,
                WALL_HEIGHT / 2.0,
                0.0,
            ),
            "Wall East",
        ),
        (
            Vec3::new(
                -ROOM_SIZE / 2.0 - WALL_THICKNESS / 2.0,
                WALL_HEIGHT / 2.0,
                0.0,
            ),
            "Wall West",
        ),
        (
            Vec3::new(
                0.0,
                WALL_HEIGHT / 2.0,
                ROOM_SIZE / 2.0 + WALL_THICKNESS / 2.0,
            ),
            "Wall North",
        ),
        (
            Vec3::new(
                0.0,
                WALL_HEIGHT / 2.0,
                -ROOM_SIZE / 2.0 - WALL_THICKNESS / 2.0,
            ),
            "Wall South",
        ),
    ];

    for (position, name) in wall_positions {
        commands.spawn((
            Name::new(name),
            WallMarker,
            Position(position),
            Rotation::default(),
            Replicate::to_clients(NetworkTarget::All),
        ));
    }

    info!("Scene setup complete");
}

fn spawn_enemies_on_server_start(_trigger: On<Add, Started>, mut commands: Commands) {
    info!("Spawning enemies on server start");

    // Define enemy spawn positions around the room
    let enemy_positions = [
        (Vec3::new(8.0, 1.0, 8.0), EnemyType::Basic),
        (Vec3::new(-8.0, 1.0, 8.0), EnemyType::Fast),
        (Vec3::new(8.0, 1.0, -8.0), EnemyType::Heavy),
        (Vec3::new(-8.0, 1.0, -8.0), EnemyType::Basic),
        (Vec3::new(0.0, 1.0, 12.0), EnemyType::Fast),
    ];

    for (position, enemy_type) in &enemy_positions {
        let enemy_entity = EnemyEntity::new(enemy_type.clone());

        // Define patrol points for each enemy
        let patrol_points = match enemy_type {
            EnemyType::Basic => vec![
                *position,
                *position + Vec3::new(4.0, 0.0, 0.0),
                *position + Vec3::new(4.0, 0.0, 4.0),
                *position + Vec3::new(0.0, 0.0, 4.0),
            ],
            EnemyType::Fast => vec![
                *position,
                *position + Vec3::new(6.0, 0.0, 2.0),
                *position + Vec3::new(2.0, 0.0, 6.0),
            ],
            EnemyType::Heavy => vec![
                *position,
                *position + Vec3::new(2.0, 0.0, 0.0),
                *position + Vec3::new(0.0, 0.0, 2.0),
            ],
        };

        // Create the physics bundle with default values
        let mut physics_bundle = enemy_entity.get_physics_bundle();

        // Update the position in the physics bundle
        physics_bundle.enemy_bundle.position = Position(*position);

        // Update behaviors with specific values
        physics_bundle.enemy_bundle.patrol.patrol_points = patrol_points;
        physics_bundle.enemy_bundle.patrol.patrol_speed = match enemy_type {
            EnemyType::Basic => 2.0,
            EnemyType::Fast => 3.5,
            EnemyType::Heavy => 1.5,
        };

        physics_bundle.enemy_bundle.chase.chase_speed = match enemy_type {
            EnemyType::Basic => 4.0,
            EnemyType::Fast => 6.0,
            EnemyType::Heavy => 3.0,
        };

        physics_bundle.enemy_bundle.attack.attack_damage = match enemy_type {
            EnemyType::Basic => 25.0,
            EnemyType::Fast => 15.0,
            EnemyType::Heavy => 40.0,
        };

        let entity_id = commands
            .spawn((
                Name::new(format!("{:?}_Enemy", enemy_type)),
                physics_bundle,
                // Health system - enemies don't regenerate health
                Health::no_regeneration(match enemy_type {
                    EnemyType::Basic => 80.0,
                    EnemyType::Fast => 50.0,
                    EnemyType::Heavy => 150.0,
                }),
                Replicate::to_clients(NetworkTarget::All),
                InterpolationTarget::to_clients(NetworkTarget::All),
            ))
            .id();

        // Add navigation agent with appropriate speed for enemy type
        let navigation_speed = match enemy_type {
            EnemyType::Basic => 3.0,
            EnemyType::Fast => 5.0,
            EnemyType::Heavy => 2.0,
        };
        add_navigation_agent_with_speed(&mut commands, entity_id, navigation_speed);
    }

    info!("Spawned {} enemies", 5);
}

fn setup_navigation_on_server_start(_trigger: On<Add, Started>, mut commands: Commands) {
    info!("Setting up navigation system on server start");

    // Spawn the navigation mesh marker to trigger mesh building
    let mesh_entity = commands
        .spawn((Name::new("NavigationMesh"), NavigationMeshMarker))
        .id();

    info!("Navigation mesh marker spawned: {:?}", mesh_entity);
}
