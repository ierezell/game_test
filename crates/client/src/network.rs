use crate::{ClientGameState, LocalPlayerId};

use bevy::prelude::{
    Add, App, Commands, CommandsStatesExt, Entity, IntoScheduleConfigs, Name, On, Plugin, Query,
    Remove, Res, Resource, State, Update, With, Without, error, in_state, info,
};

#[derive(Resource)]
pub struct ServerAddr(pub std::net::SocketAddr);
use lightyear::prelude::{
    Authentication, Client, Connect, Connected, Connecting, Link, LocalAddr, PeerAddr,
    ReplicationReceiver, ReplicationSender, UdpIo,
    client::{NetcodeClient, NetcodeConfig},
};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use shared::debug::debug_println;
use shared::protocol::{LevelSeed, LobbyState};
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
                app.add_systems(
                    Update,
                    start_connection.run_if(in_state(ClientGameState::Lobby)),
                );
            }
            NetworkMode::Crossbeam => {
                app.add_systems(
                    Update,
                    start_connection_crossbeam.run_if(in_state(ClientGameState::Lobby)),
                );
            }
            NetworkMode::Local => {
                app.add_systems(
                    Update,
                    start_connection_local.run_if(in_state(ClientGameState::Lobby)),
                );
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
    reconnect_candidates: Query<Entity, (With<Client>, Without<Connected>, Without<Connecting>)>,
    endpoint: Res<CrossbeamClientEndpoint>,
) {
    if !existing_clients.is_empty() {
        for client_entity in reconnect_candidates.iter() {
            commands.trigger(Connect {
                entity: client_entity,
            });
        }
        return;
    }

    debug_println(format_args!(
        "DEBUG: start_connection_crossbeam called for client {}",
        client_id.0
    ));

    use lightyear::prelude::{
        Connected, Linked, LocalId, MessageReceiver, MessageSender, PeerId, PingConfig,
        PingManager, RemoteId, ReplicationSender, Transport,
    };

    let io = endpoint.0.clone();

    let client_entity = commands
        .spawn((
            Client,
            Connected,
            Link::default(),
            Linked,
            io,
            Transport::default(),
            RemoteId(PeerId::Server),
            LocalId(PeerId::Netcode(client_id.0)),
            PingManager::new(PingConfig {
                ping_interval: std::time::Duration::default(),
            }),
            ReplicationSender::default(),
            ReplicationReceiver::default(),
            MessageSender::<shared::protocol::HostStartGameEvent>::default(),
            MessageSender::<shared::protocol::TerminalInteractionRequest>::default(),
            MessageReceiver::<shared::protocol::TerminalInteractionResponse>::default(),
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
    reconnect_candidates: Query<Entity, (With<Client>, Without<Connected>, Without<Connecting>)>,
    server_query: Query<Entity, With<lightyear::prelude::server::Server>>,
) {
    if !existing_clients.is_empty() {
        for client_entity in reconnect_candidates.iter() {
            commands.trigger(Connect {
                entity: client_entity,
            });
        }
        return;
    }

    debug_println(format_args!(
        "DEBUG: start_connection_local called for client {}",
        client_id.0
    ));

    use lightyear::prelude::{Link, LinkOf, LocalId, PeerId, RemoteId};

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
            Client,
            LinkOf {
                server: server_entity,
            },
            Link::default(),
            RemoteId(PeerId::Server),
            LocalId(PeerId::Netcode(client_id.0)),
            ReplicationSender::default(),
            ReplicationReceiver::default(),
        ))
        .insert(Name::from(format!("HostClient {}", client_id.0)))
        .id();

    debug_println(format_args!(
        "DEBUG: Created HostClient entity {:?} linked to Server entity {:?}",
        client_entity, server_entity
    ));

    commands.trigger(Connect {
        entity: client_entity,
    });
}

fn start_connection(
    mut commands: Commands,
    client_id: Res<LocalPlayerId>,
    existing_clients: Query<Entity, With<Client>>,
    reconnect_candidates: Query<Entity, (With<Client>, Without<Connected>, Without<Connecting>)>,
    test_server_addr: Option<Res<ServerAddr>>,
) {
    if !existing_clients.is_empty() {
        for client_entity in reconnect_candidates.iter() {
            commands.trigger(Connect {
                entity: client_entity,
            });
        }
        return;
    }

    debug_println(format_args!(
        "DEBUG: start_connection called for client {}",
        client_id.0
    ));

    let client_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0);

    let server_addr = if let Some(test_addr) = test_server_addr {
        test_addr.0
    } else {
        SERVER_ADDR
    };
    debug_println(format_args!(
        "DEBUG: Client {} connecting to server at {}",
        client_id.0, server_addr
    ));

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
            debug_println(format_args!(
                "DEBUG: NetcodeClient created successfully for client {}",
                client_id.0
            ));
            let client_entity = commands
                .spawn((
                    Client,
                    LocalAddr(client_addr),
                    PeerAddr(server_addr),
                    Link::default(),
                    ReplicationSender::default(),
                    ReplicationReceiver::default(),
                    netcode_client,
                    UdpIo::default(),
                ))
                .insert(Name::from(format!("Client {}", client_id.0)))
                .id();

            commands.trigger(Connect {
                entity: client_entity,
            });
        }
        Err(e) => {
            error!("❌ Failed to create Netcode client: {:?}", e);
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
    lobby_state_query: Query<Entity, With<LobbyState>>,
    level_seed_query: Query<Entity, With<LevelSeed>>,
) {
    let current_state_value = current_state.get();
    info!(
        "💔 Client {:?} disconnected from server while in state: {:?}",
        trigger.entity, current_state_value
    );

    // Clean up stale replicated lobby and level seed entities
    for entity in lobby_state_query.iter() {
        commands.entity(entity).despawn();
    }
    for entity in level_seed_query.iter() {
        commands.entity(entity).despawn();
    }

    commands.set_state(ClientGameState::LocalMenu);
}

#[cfg(test)]
mod tests {
    use super::handle_client_disconnected;
    use crate::ClientGameState;
    use bevy::prelude::{App, MinimalPlugins, State};
    use bevy::state::app::AppExtStates;
    use lightyear::prelude::{Connected, PeerId, RemoteId};
    use shared::protocol::{LevelSeed, LobbyState};

    #[test]
    fn disconnect_cleans_replicated_entities_and_transitions_to_local_menu() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(bevy::state::app::StatesPlugin);
        app.init_state::<ClientGameState>();
        app.insert_state(ClientGameState::Lobby);
        app.add_observer(handle_client_disconnected);

        let lobby_ent = app
            .world_mut()
            .spawn(LobbyState {
                players: vec![1, 2],
                host_id: Some(1),
            })
            .id();
        let seed_ent = app.world_mut().spawn(LevelSeed { seed: 42 }).id();

        let client_ent = app
            .world_mut()
            .spawn((Connected, RemoteId(PeerId::Server)))
            .id();
        app.update();

        // Simulate disconnection by removing Connected
        app.world_mut().entity_mut(client_ent).remove::<Connected>();
        app.update();

        let state = app.world().resource::<State<ClientGameState>>().get();
        assert_eq!(state, &ClientGameState::LocalMenu);

        let lobby_exists = app.world().get::<LobbyState>(lobby_ent).is_some();
        let seed_exists = app.world().get::<LevelSeed>(seed_ent).is_some();
        assert!(
            !lobby_exists,
            "LobbyState should be despawned on client disconnect"
        );
        assert!(
            !seed_exists,
            "LevelSeed should be despawned on client disconnect"
        );
    }
}
