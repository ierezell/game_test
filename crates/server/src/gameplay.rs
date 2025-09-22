use avian3d::prelude::*;
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;
use lightyear::connection::client::Connected;
use lightyear::connection::server::Started;

use lightyear::prelude::server::*;
use lightyear::prelude::*;
use shared::input::{PLAYER_CAPSULE_HEIGHT, PlayerAction, shared_player_movement};
use shared::protocol::{PlayerColor, PlayerId};
use shared::scene::*;

pub struct ServerGameplayPlugin;

impl Plugin for ServerGameplayPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(setup_scene_on_server_start);
        app.add_observer(handle_connected);
        app.add_systems(FixedUpdate, server_player_movement);
    }
}

fn handle_connected(
    trigger: Trigger<OnAdd, Connected>,
    query: Query<&RemoteId, With<ClientOf>>,
    mut commands: Commands,
) {
    let Ok(client_id) = query.get(trigger.target()) else {
        return;
    };
    let peer_id = client_id.0;
    info!("Client connected with client-id {client_id:?}. Spawning player entity.");

    let color = color_from_id(client_id.to_bits());
    let angle: f32 = client_id.to_bits() as f32 * 6.28 / 4.0; // Distribute around circle
    let x = 5.0 * angle.cos();
    let z = 5.0 * angle.sin();

    let y = (PLAYER_CAPSULE_HEIGHT / 2.0) + 1.0; // Capsule center at half-height above ground

    info!(
        "🎯 Setting up prediction target for client_id: {:?} (peer_id: {})",
        client_id, peer_id
    );
    info!(
        "🔍 Client entity: {:?}, RemoteId bits: {}",
        trigger.target(),
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
            // Lightyear config
            ControlledBy {
                owner: trigger.target(),
                lifetime: Default::default(),
            },
            Replicate::to_clients(NetworkTarget::All),
            PredictionTarget::to_clients(NetworkTarget::Single(peer_id)),
            InterpolationTarget::to_clients(NetworkTarget::AllExceptSingle(peer_id)),
            // Should not be replicated
            PlayerPhysicsBundle::default(),
        ))
        .id();

    info!("Created player entity {player:?} for client {client_id:?}");
    info!(
        "🔍 ControlledBy owner set to client entity: {:?}",
        trigger.target()
    );
}

pub fn server_player_movement(
    time: Res<Time>,
    mut player_query: Query<
        (
            Entity,
            &mut Rotation, // Position is now controlled by the physics engine
            &mut LinearVelocity,
            &ActionState<PlayerAction>,
        ),
        // Based on lightyear examples - avoid applying movement to predicted/confirmed entities
        // to prevent conflicts in host-server mode
        (With<PlayerId>, Without<Predicted>, Without<Confirmed>),
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

        shared_player_movement(&time, action_state, &mut rotation, &mut velocity);
    }
}

fn setup_scene_on_server_start(_trigger: Trigger<OnAdd, Started>, mut commands: Commands) {
    info!("Setting up scene on server (after server started)");

    commands.spawn((
        Name::new("Floor"),
        FloorMarker,
        Position(Vec3::new(0.0, -FLOOR_THICKNESS / 2.0, 0.0)),
        Rotation::default(),
        FloorPhysicsBundle::default(),
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
        let mut wall_bundle = WallPhysicsBundle::default();

        // Adjust collider for north/south walls (rotate 90 degrees)
        if name.contains("North") || name.contains("South") {
            wall_bundle.collider = Collider::cuboid(ROOM_SIZE, WALL_HEIGHT, WALL_THICKNESS);
        }

        commands.spawn((
            Name::new(name),
            WallMarker,
            Position(position),
            Rotation::default(),
            wall_bundle,
            Replicate::to_clients(NetworkTarget::All),
        ));
    }

    info!("Scene setup complete");
}
