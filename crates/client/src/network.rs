use bevy::prelude::{
    App, Commands, CommandsStatesExt, Entity, Name, OnAdd, OnEnter, OnRemove, Plugin, Query, Res,
    ResMut, Resource, Startup, State, Trigger, Update, With, default, error, info, warn,
};
use common::{CLIENT_ADDR, PRIVATE_KEY, PROTOCOL_ID, SERVER_ADDR};
use lightyear::netcode::Key;
use lightyear::prelude::client::*;
use lightyear::prelude::*;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use crate::game_state::GameState;
use common::NetTransport;
use common::protocol::PlayerId;
pub struct NetworkPlugin;

#[derive(Resource, Default)]
pub struct ConnectionState {
    pub was_connected: bool,
    pub logged_waiting: bool,
}

#[derive(Resource)]
pub struct AutoConnect(pub bool);

impl Default for AutoConnect {
    fn default() -> Self {
        Self(false)
    }
}

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ConnectionState::default());

        // Initialize AutoConnect resource if not already present
        if !app.world().contains_resource::<AutoConnect>() {
            app.insert_resource(AutoConnect::default());
        }

        app.add_systems(OnEnter(GameState::ConnectingRemote), start_connection);
        app.add_systems(OnEnter(GameState::MainMenu), cleanup_client_connection);

        // Conditionally add auto-connect system based on resource
        app.add_systems(Startup, conditional_auto_connect);

        app.add_systems(Update, monitor_connection_status);
        app.add_systems(Update, log_connection_events);

        app.add_observer(handle_client_connected);
        app.add_observer(handle_client_disconnected);
    }
}

fn cleanup_client_connection(mut commands: Commands, client_query: Query<Entity, With<Client>>) {
    for client_entity in client_query.iter() {
        info!("🧹 Cleaning up client connection: {:?}", client_entity);
        commands.entity(client_entity).despawn();
    }
}

fn start_connection(
    mut commands: Commands,
    client_id: Res<PlayerId>,
    existing_clients: Query<Entity, With<Client>>,
) {
    if !existing_clients.is_empty() {
        info!("🔄 Client already exists, skipping connection creation");
        for client_entity in existing_clients.iter() {
            commands.trigger_targets(Connect, client_entity);
            info!(
                "🚀 Re-triggering connection on existing client: {:?}",
                client_entity
            );
        }
        return;
    }

    info!(
        "🔌 User requested connection - Starting client connection to server at {:?}",
        SERVER_ADDR
    );

    info!("📋 Using client ID: {}", client_id.0);

    let auth = Authentication::Manual {
        server_addr: SERVER_ADDR,
        client_id: client_id.0,
        private_key: PRIVATE_KEY,
        protocol_id: PROTOCOL_ID,
    };

    let netcode_config = NetcodeConfig {
        num_disconnect_packets: 10,
        keepalive_packet_send_rate: 1.0 / 10.0,
        client_timeout_secs: 15,
        token_expire_secs: 30,
    };

    let netcode_client = match NetcodeClient::new(auth, netcode_config) {
        Ok(client) => {
            info!("✅ Netcode client created successfully");
            client
        }
        Err(e) => {
            error!("❌ Failed to create netcode client: {:?}", e);
            return;
        }
    };

    let client = commands
        .spawn((
            Client::default(),
            LocalAddr(CLIENT_ADDR),
            PeerAddr(SERVER_ADDR),
            Link::new(None),
            ReplicationReceiver::default(),
            netcode_client,
            UdpIo::default(),
            Name::new(format!("Client {}", client_id.0)),
        ))
        .id();

    commands.trigger_targets(Connect, client);
    info!("🚀 Client connection initiated - entity: {:?}", client);
}

fn log_connection_events(
    connected_query: Query<(Entity, &Connected)>,
    client_query: Query<Entity, With<Client>>,
    mut connection_state: ResMut<ConnectionState>,
    current_game_state: Res<State<GameState>>,
) {
    let is_connected = !connected_query.is_empty();
    let client_exists = !client_query.is_empty();

    if *current_game_state.get() == GameState::ConnectingRemote {
        if is_connected && !connection_state.was_connected {
            for (entity, _) in connected_query.iter() {
                info!("✅ Client successfully connected - entity: {:?}", entity);
            }
            connection_state.was_connected = true;
            connection_state.logged_waiting = false;
        } else if client_exists && !is_connected && !connection_state.logged_waiting {
            for entity in client_query.iter() {
                info!(
                    "⏳ Client entity created, attempting connection - entity: {:?}",
                    entity
                );
            }
            connection_state.logged_waiting = true;
        }
    } else {
        // Reset state when not connecting
        if connection_state.was_connected || connection_state.logged_waiting {
            connection_state.was_connected = false;
            connection_state.logged_waiting = false;
        }
    }
}

fn monitor_connection_status(
    connected_query: Query<&Connected>,
    client_query: Query<Entity, With<Client>>,
    mut commands: Commands,
    current_state: Res<State<GameState>>,
    connection_state: Res<ConnectionState>,
) {
    let current_state_value = current_state.get();

    match current_state_value {
        GameState::ConnectingRemote => {
            // Don't monitor disconnection while initially connecting
            // Let the connection attempt complete first
        }
        GameState::Loading | GameState::Spawning | GameState::Playing => {
            // Only check for disconnection in these states after initial connection
            if connected_query.is_empty() && !client_query.is_empty() {
                info!(
                    "❌ Connection lost while in state {:?}, returning to main menu",
                    current_state_value
                );
                commands.set_state(GameState::MainMenu);
            }
        }
        _ => {
            // Don't monitor connection in menu states
        }
    }
}

fn handle_client_connected(
    trigger: Trigger<OnAdd, Connected>,
    mut commands: Commands,
    current_state: Res<State<GameState>>,
) {
    info!("🎉 Client successfully connected to server!");
    if *current_state.get() == GameState::ConnectingRemote {
        info!("📥 Transitioning to Loading state");
        commands.set_state(GameState::Loading);
    }
}

fn handle_client_disconnected(
    trigger: Trigger<OnRemove, Connected>,
    mut commands: Commands,
    current_state: Res<State<GameState>>,
) {
    let current_state_value = current_state.get();
    info!(
        "💔 Client disconnected from server while in state: {:?}",
        current_state_value
    );

    if *current_state_value != GameState::MainMenu {
        info!("🏠 Returning to main menu due to disconnection");
        commands.set_state(GameState::MainMenu);
    }
}

// Conditional auto-connect system that respects CLI flag
fn conditional_auto_connect(
    mut commands: Commands,
    current_state: Res<State<GameState>>,
    auto_connect: Res<AutoConnect>,
) {
    if auto_connect.0 && *current_state.get() == GameState::MainMenu {
        info!("🤖 Auto-connecting (enabled via CLI)...");
        commands.set_state(GameState::ConnectingRemote);
    }
}
