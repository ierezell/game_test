use crate::{ClientGameState, LocalPlayerId};
use bevy::log::debug;
use bevy::prelude::{
    Add, App, Commands, CommandsStatesExt, Entity, Name, On, OnEnter, Plugin, Query, Remove, Res,
    Resource, State, With, error, info,
};
use lightyear::prelude::{
    Authentication, Client, Connect, Connected, Link, LocalAddr, PeerAddr, PredictionManager,
    ReplicationReceiver, UdpIo,
    client::{NetcodeClient, NetcodeConfig},
};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use shared::{SERVER_ADDR, SHARED_SETTINGS};

#[derive(Resource)]
pub struct AutoJoin(pub bool);

impl Default for AutoJoin {
    fn default() -> Self {
        Self(false)
    }
}

pub struct ClientNetworkPlugin;
impl Plugin for ClientNetworkPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(ClientGameState::Lobby), start_connection);
        app.add_observer(handle_client_connected);
        app.add_observer(handle_client_disconnected);
    }
}

fn start_connection(
    mut commands: Commands,
    client_id: Res<LocalPlayerId>,
    existing_clients: Query<Entity, With<Client>>,
) {
    if !existing_clients.is_empty() {
        info!("🔄 Client already exists, skipping connection creation");
        for client_entity in existing_clients.iter() {
            commands.trigger(Connect {
                entity: client_entity,
            });
            info!(
                "🚀 Re-triggering connection on existing client: {:?}",
                client_entity
            );
        }
        return;
    }

    // Use a different port range to avoid conflicts with server
    let client_port = 5000 + client_id.0 as u16;
    let client_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), client_port);

    let auth = Authentication::Manual {
        server_addr: SERVER_ADDR,
        client_id: client_id.0,
        private_key: SHARED_SETTINGS.private_key,
        protocol_id: SHARED_SETTINGS.protocol_id,
    };

    let netcode_config = NetcodeConfig {
        num_disconnect_packets: 10,
        keepalive_packet_send_rate: 1.0 / 10.0,
        client_timeout_secs: 10,
        token_expire_secs: 30,
    };

    match NetcodeClient::new(auth, netcode_config) {
        Ok(netcode_client) => {
            debug!("✅ Netcode client created successfully");
            let client_entity = commands
                .spawn((
                    Client::default(),
                    LocalAddr(client_addr),
                    PeerAddr(SERVER_ADDR),
                    Link::new(None),
                    ReplicationReceiver::default(),
                    netcode_client,
                    UdpIo::default(),
                    PredictionManager::default(),
                ))
                .insert(Name::from(format!("Client {}", client_id.0)))
                .id();

            commands.trigger(Connect {
                entity: client_entity,
            });

            debug!(
                "🚀 Client connection initiated - entity: {:?}",
                client_entity
            );
        }
        Err(e) => {
            error!("❌ Failed to create Netcode client: {:?}", e);
            error!(
                "This might be because the client port {} is already in use.",
                client_port
            );
            error!("Server might not be ready yet or there's a network issue.");
        }
    }
}

fn handle_client_connected(trigger: On<Add, Connected>) {
    debug!(
        "🎉 Client {:?} successfully connected to server!",
        trigger.entity
    );
}

fn handle_client_disconnected(
    trigger: On<Remove, Connected>,
    mut commands: Commands,
    current_state: Res<State<ClientGameState>>,
) {
    let current_state_value = current_state.get();
    info!(
        "💔 Client {:?} disconnected from server while in state: {:?}",
        trigger.entity, current_state_value
    );

    commands.set_state(ClientGameState::LocalMenu);
}
