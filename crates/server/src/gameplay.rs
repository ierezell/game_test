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

use shared::health::{Health, Respawnable};
use shared::input::{PLAYER_CAPSULE_HEIGHT, PlayerAction, shared_player_movement_with_stamina};
use shared::navigation_pathfinding::{NavigationMeshMarker, add_navigation_agent_with_speed};
use shared::protocol::{PlayerColor, PlayerId};
use shared::scene::{
    FLOOR_THICKNESS, FloorMarker, ROOM_SIZE, WALL_HEIGHT, WALL_THICKNESS, WallMarker, color_from_id,
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
        app.add_systems(bevy::prelude::Startup, spawn_enemies_on_server_start);
        app.add_systems(bevy::prelude::Startup, setup_navigation_on_server_start);
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
            Name::new(format!("Player_{}", client_id.to_bits())),
            PlayerId(peer_id),
        ))
        .insert(LinearVelocity::default())
        .insert(Position(Vec3::new(x, y, z)))
        .insert(Rotation::default())
        .insert(PlayerColor(color))
        .insert(Health::with_regeneration(100.0, 10.0, 5.0))
        .insert(Respawnable::new(5.0))
        .insert(ControlledBy {
            owner: trigger.entity,
            lifetime: Default::default(),
        })
        .insert(Replicate::to_clients(NetworkTarget::All))
        .insert(PredictionTarget::to_clients(NetworkTarget::Single(peer_id)))
        .insert(InterpolationTarget::to_clients(
            NetworkTarget::AllExceptSingle(peer_id),
        ))
        .insert(shared::scene::PlayerPhysicsBundle::default())
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
    for (entity, mut rotation, mut velocity, action_state, _stamina_effects) in
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

        shared_player_movement_with_stamina(action_state, &mut rotation, &mut velocity);
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

fn spawn_enemies_on_server_start(mut commands: Commands) {
    info!("Spawning enemies on server start");
    let enemy_positions = [Vec3::new(8.0, 1.0, 8.0)];

    for position in &enemy_positions {
        let entity_id = commands
            .spawn((
                Position(*position),
                Name::new("Enemy"),
                Health::no_regeneration(80.0),
                Replicate::to_clients(NetworkTarget::All),
                InterpolationTarget::to_clients(NetworkTarget::All),
            ))
            .id();

        add_navigation_agent_with_speed(&mut commands, entity_id, 5.0);
    }

    info!("Spawned {} enemies", 5);
}

fn setup_navigation_on_server_start(mut commands: Commands) {
    info!("Setting up navigation system on server start");

    // Spawn the navigation mesh marker to trigger mesh building
    let mesh_entity = commands
        .spawn((Name::new("NavigationMesh"), NavigationMeshMarker))
        .id();

    info!("Navigation mesh marker spawned: {:?}", mesh_entity);
}
