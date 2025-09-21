use bevy::log::debug;
use bevy::prelude::{
    App, Commands, CommandsStatesExt, Entity, Name, OnAdd, OnEnter, OnRemove, Plugin, Query, Res,
    ResMut, Resource, Startup, State, Trigger, Update, With, error,
};

use lightyear::prelude::client::*;
use lightyear::prelude::*;
use shared::{SERVER_ADDR, SHARED_SETTINGS};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use crate::app::LocalPlayerId;
use crate::game_state::GameState;
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

        if !app.world().contains_resource::<AutoConnect>() {
            app.insert_resource(AutoConnect::default());
        }

        app.add_systems(OnEnter(GameState::ConnectingRemote), start_connection);
        app.add_systems(OnEnter(GameState::MainMenu), cleanup_client_connection);

        app.add_systems(Startup, conditional_auto_connect);

        app.add_systems(Update, monitor_connection_status);
        app.add_systems(Update, log_connection_events);

        app.add_observer(handle_client_connected);
        app.add_observer(handle_client_disconnected);
    }
}

fn cleanup_client_connection(mut commands: Commands, client_query: Query<Entity, With<Client>>) {
    for client_entity in client_query.iter() {
        debug!("🧹 Cleaning up client connection: {:?}", client_entity);
        commands.entity(client_entity).despawn();
    }
}

fn start_connection(
    mut commands: Commands,
    client_id: Res<LocalPlayerId>,
    existing_clients: Query<Entity, With<Client>>,
) {
    if !existing_clients.is_empty() {
        debug!("🔄 Client already exists, skipping connection creation");
        for client_entity in existing_clients.iter() {
            commands.trigger_targets(Connect, client_entity);
            debug!(
                "🚀 Re-triggering connection on existing client: {:?}",
                client_entity
            );
        }
        return;
    }

    debug!(
        "🔌 User requested connection - Starting client connection to server at {:?}",
        SERVER_ADDR
    );

    debug!("📋 Using client ID: {}", client_id.0);

    let client_port = 4000 + client_id.0 as u16;
    let client_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), client_port);
    debug!("🔌 Client binding to local address: {}", client_addr);

    let auth = Authentication::Manual {
        server_addr: SERVER_ADDR,
        client_id: client_id.0,
        private_key: SHARED_SETTINGS.private_key,
        protocol_id: SHARED_SETTINGS.protocol_id,
    };

    let netcode_config = NetcodeConfig {
        num_disconnect_packets: 10,
        keepalive_packet_send_rate: 1.0 / 10.0,
        client_timeout_secs: 3,
        token_expire_secs: 30,
    };

    let netcode_client = match NetcodeClient::new(auth, netcode_config) {
        Ok(client) => {
            debug!("✅ Netcode client created successfully");
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
            LocalAddr(client_addr),
            PeerAddr(SERVER_ADDR),
            Link::new(None),
            ReplicationReceiver::default(),
            // 🔧 FIX: Manually add PredictionManager like examples show
            PredictionManager::default(),
            InterpolationManager::default(),
            netcode_client,
            UdpIo::default(),
            Name::new(format!("Client {}", client_id.0)),
        ))
        .id();

    commands.trigger_targets(Connect, client);
    debug!("🚀 Client connection initiated - entity: {:?}", client);
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
                debug!("✅ Client successfully connected - entity: {:?}", entity);
            }
            connection_state.was_connected = true;
            connection_state.logged_waiting = false;
        } else if client_exists && !is_connected && !connection_state.logged_waiting {
            for entity in client_query.iter() {
                debug!(
                    "⏳ Client entity created, attempting connection - entity: {:?}",
                    entity
                );
            }
            connection_state.logged_waiting = true;
        }
    } else {
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
) {
    let current_state_value = current_state.get();

    match current_state_value {
        GameState::ConnectingRemote => {
            // Don't monitor disconnection while initially connecting
            // Let the connection attempt complete first
        }
        GameState::Loading | GameState::Playing => {
            // Only check for disconnection in these states after initial connection
            if connected_query.is_empty() && !client_query.is_empty() {
                debug!(
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
    debug!(
        "🎉 Client {:?} successfully connected to server! in state {:?}",
        trigger.target(),
        current_state
    );
    if *current_state.get() == GameState::ConnectingRemote {
        debug!("📥 Transitioning to Loading state");
        commands.set_state(GameState::Loading);
    }
}

fn handle_client_disconnected(
    trigger: Trigger<OnRemove, Connected>,
    mut commands: Commands,
    current_state: Res<State<GameState>>,
) {
    let current_state_value = current_state.get();
    debug!(
        "💔 Client {:?} disconnected from server while in state: {:?}",
        trigger.target(),
        current_state_value
    );

    if *current_state_value != GameState::MainMenu {
        debug!("🏠 Returning to main menu due to disconnection");
        commands.set_state(GameState::MainMenu);
    }
}

fn conditional_auto_connect(
    mut commands: Commands,
    current_state: Res<State<GameState>>,
    auto_connect: Res<AutoConnect>,
) {
    if auto_connect.0 && *current_state.get() == GameState::MainMenu {
        debug!("🤖 Auto-connecting (enabled via CLI)...");
        commands.set_state(GameState::ConnectingRemote);
    }
}
