use crate::{ClientGameState, LocalPlayerId};

use bevy::prelude::{
    Add, App, Commands, CommandsStatesExt, Entity, Name, On, OnEnter, Plugin, Query, Remove, Res,
    Resource, State, With, error, info,
};

#[derive(Resource)]
pub struct ServerAddr(pub std::net::SocketAddr);
use lightyear::prelude::{
    Authentication, Client, Connect, Connected, Link, LocalAddr, PeerAddr, PredictionManager,
    ReplicationReceiver, ReplicationSender, UdpIo,
    client::{NetcodeClient, NetcodeConfig},
};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use shared::{SERVER_ADDR, SHARED_SETTINGS};

#[derive(Resource)]
pub struct CrossbeamClientEndpoint(pub lightyear::crossbeam::CrossbeamIo);

pub struct ClientNetworkPlugin;
impl Plugin for ClientNetworkPlugin {
    fn build(&self, app: &mut App) {
        use shared::NetworkMode;

        let network_mode = app
            .world()
            .get_resource::<NetworkMode>()
            .copied()
            .unwrap_or_default();
        match network_mode {
            NetworkMode::Udp => {
                app.add_systems(OnEnter(ClientGameState::Lobby), start_connection);
            }
            NetworkMode::Crossbeam => {
                app.add_systems(OnEnter(ClientGameState::Lobby), start_connection_crossbeam);
            }
            NetworkMode::Local => {
                app.add_systems(OnEnter(ClientGameState::Lobby), start_connection_local);
            }
        }

        app.add_observer(handle_client_connected);
        app.add_observer(handle_client_disconnected);
    }
}

fn start_connection_crossbeam(
    mut commands: Commands,
    client_id: Res<LocalPlayerId>,
    existing_clients: Query<Entity, With<Client>>,
    endpoint: Res<CrossbeamClientEndpoint>,
) {
    if !existing_clients.is_empty() {
        for client_entity in existing_clients.iter() {
            commands.trigger(Connect {
                entity: client_entity,
            });
        }
        return;
    }

    println!(
        "DEBUG: start_connection_crossbeam called for client {}",
        client_id.0
    );

    use lightyear::prelude::{
        Linked, LocalId, PeerId, PingConfig, PingManager, RemoteId, ReplicationSender,
        Transport,
    };

    // Clone the endpoint because we might need it again if we reconnect (though Res is borrowed)
    // CrossbeamIo should be cloneable (channels are).
    let io = endpoint.0.clone();

    let client_entity = commands
        .spawn((
            Client::default(),
            Link::new(None),
            Linked, // Crossbeam is always immediately linked
            io,
            Transport::default(),
            RemoteId(PeerId::Server),
            LocalId(PeerId::Netcode(client_id.0)),
            PingManager::new(PingConfig {
                ping_interval: std::time::Duration::default(),
            }),
            ReplicationSender::default(),
            ReplicationReceiver::default(),
            PredictionManager::default(),
        ))
        .insert(Name::from(format!("Client {}", client_id.0)))
        .id();

    commands.trigger(Connect {
        entity: client_entity,
    });
}

fn start_connection_local(
    mut commands: Commands,
    client_id: Res<LocalPlayerId>,
    existing_clients: Query<Entity, With<Client>>,
    server_query: Query<Entity, With<lightyear::prelude::server::Server>>,
) {
    if !existing_clients.is_empty() {
        for client_entity in existing_clients.iter() {
            commands.trigger(Connect {
                entity: client_entity,
            });
        }
        return;
    }

    println!(
        "DEBUG: start_connection_local called for client {}",
        client_id.0
    );

    // Local mode (HostClient): Create a Client entity linked to the Server entity
    // This is the HostServer pattern from Lightyear
    use lightyear::prelude::{Link, LinkOf, Linked, PingConfig, PingManager};

    let server_entity = match server_query.iter().next() {
        Some(entity) => entity,
        None => {
            error!(
                "Failed to find Server entity for HostClient - server may not be initialized yet"
            );
            return;
        }
    };

    let client_entity = commands
        .spawn((
            Client::default(),
            LinkOf {
                server: server_entity,
            },
            Link::new(None),
            Linked, // HostClient is always immediately linked
            PingManager::new(PingConfig {
                ping_interval: std::time::Duration::default(),
            }),
            ReplicationSender::default(),
            PredictionManager::default(),
        ))
        .insert(Name::from(format!("HostClient {}", client_id.0)))
        .id();

    println!(
        "DEBUG: Created HostClient entity {:?} linked to Server entity {:?}",
        client_entity, server_entity
    );

    commands.trigger(Connect {
        entity: client_entity,
    });
}

fn start_connection(
    mut commands: Commands,
    client_id: Res<LocalPlayerId>,
    existing_clients: Query<Entity, With<Client>>,
    test_server_addr: Option<Res<ServerAddr>>,
) {
    if !existing_clients.is_empty() {
        for client_entity in existing_clients.iter() {
            commands.trigger(Connect {
                entity: client_entity,
            });
        }
        return;
    }

    println!("DEBUG: start_connection called for client {}", client_id.0);

    // Use a different port range to avoid conflicts with server
    let client_port = 5000 + client_id.0 as u16;
    let client_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), client_port);

    // Use dynamic server address for testing if available, otherwise use default
    let server_addr = if let Some(test_addr) = test_server_addr {
        test_addr.0
    } else {
        SERVER_ADDR
    };
    println!(
        "DEBUG: Client {} connecting to server at {}",
        client_id.0, server_addr
    );

    let auth = Authentication::Manual {
        server_addr,
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
            println!(
                "DEBUG: NetcodeClient created successfully for client {}",
                client_id.0
            );
            let client_entity = commands
                .spawn((
                    Client::default(),
                    LocalAddr(client_addr),
                    PeerAddr(server_addr),
                    Link::new(None),
                    ReplicationSender::default(),
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
    info!(
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
