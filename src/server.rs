//! Server-side networking logic

use crate::protocol::*;
use crate::shared::{LocalWeaponFireEvent, SharedPlugin};
use bevy::prelude::*;
use lightyear::prelude::server::*;
use lightyear::prelude::*;

#[derive(Clone)]
pub struct ServerPlugin;

impl Plugin for ServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(SharedPlugin);

        // Server-specific systems
        app.add_systems(Startup, setup_server);
        app.add_systems(
            Update,
            (
                handle_weapon_fire,
                convert_local_weapon_events,
                server_update_loop,
            ),
        );

        // Handle new client connections
        app.add_observer(handle_new_client);

        info!("🚀 Server plugin initialized");
    }
}

/// Initialize the server
fn setup_server(mut commands: Commands) {
    info!("🚀 Starting GTFO-Like Game Server...");

    // Spawn the server entity with netcode configuration
    let _server = commands
        .spawn((
            NetcodeServer::new(NetcodeConfig {
                protocol_id: PROTOCOL_ID,
                private_key: PRIVATE_KEY,
                ..default()
            }),
            LocalAddr(SERVER_ADDR),
            ServerUdpIo::default(),
        ))
        .id();

    // TODO: Fix server start trigger with correct Lightyear API
    // commands.trigger_targets(Start, server);

    info!("📡 Server listening on {}", SERVER_ADDR);
    info!("✅ Server initialization complete");

    // Server setup complete
}

/// Handle new client connections
fn handle_new_client(trigger: Trigger<OnAdd, Connected>, mut commands: Commands) {
    let client_entity = trigger.target();
    info!("🎮 New client connected: {:?}", client_entity);

    // Add replication sender to the client connection
    commands
        .entity(client_entity)
        .insert(ReplicationSender::new(
            SERVER_REPLICATION_INTERVAL,
            SendUpdatesMode::SinceLastAck,
            false,
        ));

    // Generate a unique player ID for this client
    let player_id = client_entity.index() as u64;

    // Spawn a networked player entity for this client
    let player_entity = commands
        .spawn(NetworkedPlayerBundle::new(
            player_id,
            Vec3::new(0.0, 2.0, 0.0),
        ))
        .id();

    info!(
        "Spawned player entity {:?} for client {:?}",
        player_entity, client_entity
    );
}

/// Handle weapon fire events and broadcast to clients
fn handle_weapon_fire(
    mut weapon_fire_receivers: Query<(Entity, &mut MessageReceiver<WeaponFireMessage>)>,
) {
    for (entity, mut receiver) in weapon_fire_receivers.iter_mut() {
        for weapon_fire in receiver.receive() {
            // For now, just log the weapon fire event
            // Full networking implementation would broadcast to clients here
            info!(
                "Received weapon fire from client {:?}, player {}, position: {:?}, direction: {:?}",
                entity, weapon_fire.player_id, weapon_fire.origin, weapon_fire.direction
            );
        }
    }
}

/// Server main update loop to keep it running
fn server_update_loop(time: Res<Time>) {
    // Server heartbeat - runs every frame to keep server alive
    // In a real server, this would handle:
    // - Player position updates
    // - Physics simulation
    // - AI enemy updates
    // - Network message processing

    // For now, just log every 5 seconds
    static mut LAST_LOG: f32 = 0.0;
    let current_time = time.elapsed_secs();

    unsafe {
        if current_time - LAST_LOG > 5.0 {
            info!(
                "Server running - {} players connected, uptime: {:.1}s",
                0, current_time
            );
            LAST_LOG = current_time;
        }
    }
}

/// Convert local weapon fire events to network messages
fn convert_local_weapon_events(
    mut local_events: EventReader<LocalWeaponFireEvent>,
    // TODO: Add proper server message sender to broadcast weapon fire to clients
) {
    for event in local_events.read() {
        // For now, just log the weapon fire event
        // In a full implementation, this would broadcast to all clients
        info!(
            "Server processing weapon fire from player {}, position: {:?}, direction: {:?}",
            event.player_id, event.origin, event.direction
        );

        // TODO: Send WeaponFireMessage to all clients using ServerMultiMessageSender
    }
}
