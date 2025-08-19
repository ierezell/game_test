//! Client-side networking logic

use crate::protocol::*;
use crate::shared::SharedPlugin;

use bevy::prelude::*;
use lightyear::prelude::client::*;
use lightyear::prelude::*;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

#[derive(Clone)]
pub struct ClientPlugin;

impl Plugin for ClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(SharedPlugin);

        // Client-specific systems
        app.add_systems(Startup, setup_client);
        app.add_systems(
            Update,
            (handle_weapon_fire_messages, handle_player_messages),
        );

        // Handle connection events
        app.add_observer(handle_connection_events);

        info!("🎮 Client plugin initialized");
    }
}

/// Initialize the client
fn setup_client(mut commands: Commands) {
    info!("🎮 Starting GTFO-Like Game Client...");

    // Get client ID from args (for now, hardcode as 1)
    let client_id = 1;
    let client_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), CLIENT_PORT);

    let auth = Authentication::Manual {
        server_addr: SERVER_ADDR,
        client_id,
        private_key: PRIVATE_KEY,
        protocol_id: PROTOCOL_ID,
    };

    // Spawn the client entity
    let client = commands
        .spawn((
            Client::default(),
            LocalAddr(client_addr),
            PeerAddr(SERVER_ADDR),
            Link::new(None),
            ReplicationReceiver::default(),
            NetcodeClient::new(auth, NetcodeConfig::default()).unwrap(),
            UdpIo::default(),
        ))
        .id();

    // Connect to server
    commands.trigger_targets(Connect, client);

    info!("🔌 Attempting to connect to server at {}", SERVER_ADDR);
    info!("✅ Client initialization complete");

    // Client setup complete
    info!("✅ Client setup complete");
}

/// Handle connection events
fn handle_connection_events(trigger: Trigger<OnAdd, Connected>, mut commands: Commands) {
    info!("✅ Successfully connected to server!");

    // The client is now connected, we can add additional components if needed
    let client_entity = trigger.target();

    // Add any client-specific components here
    commands
        .entity(client_entity)
        .insert(Name::new("Connected Client"));
}

/// Handle player-related messages from server
fn handle_player_messages(// TODO: Fix message handling with correct Lightyear API
    // mut connect_messages: EventReader<ReceivedMessage<PlayerConnectedMessage>>,
    // mut disconnect_messages: EventReader<ReceivedMessage<PlayerDisconnectedMessage>>,
) {
    // TODO: Implement message handling with correct Lightyear 0.23 API
    // for message_event in connect_messages.read() {
    //     let message = &message_event.message;
    //     info!("🎮 Player {} ({}) joined the game", message.player_id, message.name);
    // }

    // for message_event in disconnect_messages.read() {
    //     let message = &message_event.message;
    //     info!("👋 Player {} left the game", message.player_id);
    // }
}

/// Handle weapon fire messages from the server
fn handle_weapon_fire_messages(
    mut weapon_fire_receivers: Query<(Entity, &mut MessageReceiver<WeaponFireMessage>)>,
) {
    for (entity, mut receiver) in weapon_fire_receivers.iter_mut() {
        for weapon_fire in receiver.receive() {
            // Play weapon fire effects, spawn projectiles, etc.
            info!(
                "Client received weapon fire from server {:?}: Player {} fired weapon from {:?} in direction {:?}",
                entity, weapon_fire.player_id, weapon_fire.origin, weapon_fire.direction
            );

            // Here you would typically:
            // - Play weapon fire sound
            // - Spawn muzzle flash effect
            // - Spawn bullet tracer
            // - Apply screen shake
        }
    }
}
