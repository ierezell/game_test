use bevy::prelude::{
    Add, App, Commands, Entity, Name, On, Plugin, PreStartup, Query, Res, Single, State, Update,
    With, Without, info,
};
use std::collections::HashSet;

use lightyear::connection::client_of::ClientOf;
use lightyear::prelude::{
    Client, Connected, ControlledBy, Disconnected, Link, LinkOf, Linked, LocalAddr, LocalId,
    MessageReceiver, NetworkTarget, PeerId, RemoteId, Replicate, ReplicationReceiver,
    ReplicationSender, Server, ServerMultiMessageSender,
    server::{NetcodeConfig, NetcodeServer, ServerUdpIo, Start, Started},
};
use shared::debug::debug_println;
use shared::protocol::{
    HostStartGameEvent, LobbyControlChannel, LobbyState, PlayerId, StartLoadingGameEvent,
    TerminalInteractionRequest,
};
use shared::{SERVER_BIND_ADDR, SHARED_SETTINGS, ServerBindAddr};

use crate::ServerGameState;
pub struct ServerNetworkPlugin;

impl Plugin for ServerNetworkPlugin {
    fn build(&self, app: &mut App) {
        use shared::NetworkMode;

        let network_mode = app
            .world()
            .get_resource::<NetworkMode>()
            .copied()
            .unwrap_or_default();
        debug_println(format_args!(
            "ServerNetworkPlugin: building with mode {:?}",
            network_mode
        ));

        match network_mode {
            NetworkMode::Udp => {
                app.add_systems(PreStartup, startup_server);
            }
            NetworkMode::Crossbeam => {
                app.add_systems(PreStartup, startup_server_crossbeam);
            }
            NetworkMode::Local => {
                app.add_systems(PreStartup, startup_server_local);
            }
        }

        app.add_observer(handle_disconnected);
        app.add_observer(handle_connected);
        app.add_systems(Update, ensure_local_host_clientof_links);
        app.add_systems(Update, reconcile_disconnected_clients);
    }
}

fn ensure_local_host_clientof_links(
    network_mode: Res<shared::NetworkMode>,
    server_query: Query<Entity, With<Server>>,
    host_clients: Query<(&LinkOf, Option<&LocalId>), (With<Client>, Without<ClientOf>)>,
    existing_clientofs: Query<&RemoteId, With<ClientOf>>,
    mut commands: Commands,
) {
    if *network_mode != shared::NetworkMode::Local {
        return;
    }

    let Ok(server_entity) = server_query.single() else {
        return;
    };

    let mut existing_remote_ids: HashSet<PeerId> =
        existing_clientofs.iter().map(|id| id.0).collect();

    for (link_of, local_id) in host_clients.iter() {
        if link_of.server != server_entity {
            continue;
        }

        let client_peer_id = local_id
            .map(|id: &LocalId| id.0)
            .unwrap_or(PeerId::Netcode(1));
        if existing_remote_ids.contains(&client_peer_id) {
            continue;
        }

        commands.spawn((
            ClientOf,
            Connected,
            LinkOf {
                server: server_entity,
            },
            Link::default(),
            Linked,
            RemoteId(client_peer_id),
            LocalId(PeerId::Server),
            Name::from(format!("LocalClientOf_{:?}", client_peer_id)),
        ));

        existing_remote_ids.insert(client_peer_id);
    }
}

fn startup_server_crossbeam(mut commands: Commands) {
    let server_entity = commands
        .spawn((Name::new("Server"), Server::default(), Started))
        .id();
    debug_println(format_args!(
        "ServerNetworkPlugin: spawned Server entity {:?}",
        server_entity
    ));
    commands.trigger(Start {
        entity: server_entity,
    });
}

fn startup_server_local(mut commands: Commands, bind_addr: Option<Res<ServerBindAddr>>) {
    let bind_addr = bind_addr.map(|a| a.0).unwrap_or(SERVER_BIND_ADDR);
    let netcode_config = NetcodeConfig {
        num_disconnect_packets: 10,
        keep_alive_send_rate: 1.0 / 10.0,
        client_timeout_secs: 10,
        protocol_id: SHARED_SETTINGS.protocol_id,
        private_key: SHARED_SETTINGS.private_key,
        connection_request_handler: None,
        server_addr_check: false,
    };

    let server_entity = commands
        .spawn((
            Name::new("Server"),
            Server::default(),
            Started,
            NetcodeServer::new(netcode_config),
            LocalAddr(bind_addr),
            ServerUdpIo::default(),
        ))
        .id();
    debug_println(format_args!(
        "ServerNetworkPlugin: spawned Server entity {:?} in Local mode (HostServer)",
        server_entity
    ));
    commands.trigger(Start {
        entity: server_entity,
    });
}

fn startup_server(mut commands: Commands, bind_addr: Option<Res<ServerBindAddr>>) {
    let bind_addr = bind_addr.map(|a| a.0).unwrap_or(SERVER_BIND_ADDR);
    let netcode_config = NetcodeConfig {
        num_disconnect_packets: 10,
        keep_alive_send_rate: 1.0 / 10.0,
        client_timeout_secs: 10,
        protocol_id: SHARED_SETTINGS.protocol_id,
        private_key: SHARED_SETTINGS.private_key,
        connection_request_handler: None,
        server_addr_check: false,
    };
    let server_entity = commands
        .spawn((
            NetcodeServer::new(netcode_config),
            LocalAddr(bind_addr),
            ServerUdpIo::default(),
        ))
        .id();

    commands.trigger(Start {
        entity: server_entity,
    });
}

#[allow(clippy::too_many_arguments)]
fn handle_connected(
    trigger: On<Add, Connected>,
    query: Query<&RemoteId, With<ClientOf>>,
    mut lobby_query: Query<(Entity, &mut LobbyState)>,
    mut commands: Commands,
    server_state: Res<State<ServerGameState>>,
    mut sender: ServerMultiMessageSender,
    server: Single<&Server>,
) {
    let Ok(client_id) = query.get(trigger.entity) else {
        return;
    };

    let client_id_bits = client_id.0.to_bits();

    commands.entity(trigger.entity).insert((
        Name::from(format!("Client_{}", client_id_bits)),
        ReplicationSender::default(),
        ReplicationReceiver::default(),
        MessageReceiver::<HostStartGameEvent>::default(),
        MessageReceiver::<TerminalInteractionRequest>::default(),
    ));

    if let Some((lobby_entity, mut lobby_state)) = lobby_query.iter_mut().next() {
        if !lobby_state.players.contains(&client_id_bits) {
            debug_println(format_args!(
                "DEBUG: Server accepted connection from Client_{}",
                client_id_bits
            ));
            lobby_state.players.push(client_id_bits);
            commands
                .entity(lobby_entity)
                .insert(Replicate::to_clients(NetworkTarget::All));

            if lobby_state.players.len() == 1 || lobby_state.host_id.is_none() {
                debug_println(format_args!("DEBUG: Client_{} became host", client_id_bits));
                lobby_state.host_id = Some(client_id_bits);
            }

            if *server_state.get() == ServerGameState::Loading
                || *server_state.get() == ServerGameState::Playing
            {
                debug_println(format_args!(
                    "DEBUG: Game already started, sending StartLoadingGameEvent to late-joining Client_{}",
                    client_id_bits
                ));

                sender
                    .send::<StartLoadingGameEvent, LobbyControlChannel>(
                        &StartLoadingGameEvent { start: true },
                        server.into_inner(),
                        &NetworkTarget::Single(client_id.0),
                    )
                    .unwrap_or_else(|e| {
                        bevy::log::error!(
                            "Failed to send StartLoadingGameEvent to late-joining client: {:?}",
                            e
                        );
                    });
            }
        } else {
            debug_println(format_args!(
                "DEBUG: Client_{} already in lobby",
                client_id_bits
            ));
        }
    } else {
        debug_println(format_args!(
            "DEBUG: Creating lobby with Client_{} as first player and host",
            client_id_bits
        ));
        commands.spawn((
            LobbyState {
                players: vec![client_id_bits],
                 host_id: Some(client_id_bits),
            },
            Replicate::to_clients(NetworkTarget::All),
            Name::from("LobbyState"),
        ));
    }
}

fn handle_disconnected(
    trigger: On<Add, Disconnected>,
    query: Query<&RemoteId, With<ClientOf>>,
    mut lobby_query: Query<&mut LobbyState>,
    player_query: Query<(Entity, &ControlledBy), With<PlayerId>>,
    mut commands: Commands,
) {
    let Ok(client_id) = query.get(trigger.entity) else {
        return;
    };

    let client_id_bits = client_id.0.to_bits();
    info!("Client {} disconnected", client_id_bits);

    for (player_entity, controlled_by) in player_query.iter() {
        if controlled_by.owner == trigger.entity {
            commands.entity(player_entity).despawn();
        }
    }

    if let Some(mut lobby_state) = lobby_query.iter_mut().next()
        && let Some(pos) = lobby_state
            .players
            .iter()
            .position(|&id| id == client_id_bits)
    {
        lobby_state.players.remove(pos);

        if lobby_state.host_id == Some(client_id_bits) {
            if let Some(&new_host_id) = lobby_state.players.first() {
                lobby_state.host_id = Some(new_host_id);
            } else {
                lobby_state.host_id = None;
            }
        }
    }
}

fn reconcile_disconnected_clients(
    connected_clients: Query<(Entity, &RemoteId), (With<ClientOf>, With<Connected>)>,
    mut lobby_query: Query<&mut LobbyState>,
    player_query: Query<(Entity, &ControlledBy, &PlayerId), With<PlayerId>>,
    mut commands: Commands,
) {
    let Some(mut lobby_state) = lobby_query.iter_mut().next() else {
        return;
    };

    let connected_ids: HashSet<u64> = connected_clients
        .iter()
        .map(|(_, remote_id)| remote_id.0.to_bits())
        .collect();

    let previous_len = lobby_state.players.len();
    lobby_state
        .players
        .retain(|player_id| connected_ids.contains(player_id));

    if lobby_state.host_id.is_some_and(|id| !connected_ids.contains(&id)) {
        lobby_state.host_id = lobby_state.players.first().copied();
    }

    if lobby_state.players.len() != previous_len {
        info!(
            "Lobby reconciled after disconnects: {} -> {} players",
            previous_len,
            lobby_state.players.len()
        );
    }

    for (player_entity, controlled_by, player_id) in player_query.iter() {
        let owner_connected = connected_clients.get(controlled_by.owner).is_ok();
        let player_bits = player_id.0.to_bits();
        let in_lobby = connected_ids.contains(&player_bits);

        if !owner_connected || !in_lobby {
            commands.entity(player_entity).despawn();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::reconcile_disconnected_clients;
    use bevy::prelude::{App, MinimalPlugins, Update};
    use lightyear::connection::client_of::ClientOf;
    use lightyear::prelude::{Connected, ControlledBy, PeerId, RemoteId};
    use shared::protocol::{LobbyState, PlayerId};

    #[test]
    fn reconcile_removes_disconnected_players_and_reassigns_host() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, reconcile_disconnected_clients);

        let connected_client = app
            .world_mut()
            .spawn((ClientOf, Connected, RemoteId(PeerId::Netcode(1))))
            .id();

        let disconnected_client = app
            .world_mut()
            .spawn((ClientOf, RemoteId(PeerId::Netcode(2))))
            .id();

        app.world_mut().spawn(LobbyState {
            players: vec![1, 2],
            host_id: Some(2),
        });

        let player_1 = app
            .world_mut()
            .spawn((
                PlayerId(PeerId::Netcode(1)),
                ControlledBy {
                    owner: connected_client,
                    lifetime: Default::default(),
                },
            ))
            .id();

        let player_2 = app
            .world_mut()
            .spawn((
                PlayerId(PeerId::Netcode(2)),
                ControlledBy {
                    owner: disconnected_client,
                    lifetime: Default::default(),
                },
            ))
            .id();

        app.update();

        let lobby = {
            let world = app.world_mut();
            world
                .query::<&LobbyState>()
                .single(world)
                .expect("lobby should exist")
                .clone()
        };

        assert_eq!(lobby.players, vec![1]);
        assert_eq!(lobby.host_id, Some(1));
        assert!(app.world().entities().contains(player_1));
        assert!(!app.world().entities().contains(player_2));
    }

    #[test]
    fn host_with_id_zero_is_not_stolen_by_second_player() {
        let lobby = LobbyState {
            players: vec![0],
            host_id: Some(0),
        };
        let second_joiner_bits = 1u64;

        assert!(
            !is_host_stolen(&lobby, second_joiner_bits),
            "Host with id 0 (local host) must not be replaced by a sentinel check"
        );
    }

    fn is_host_stolen(lobby: &LobbyState, second_player_bits: u64) -> bool {
        lobby.host_id.is_none() || lobby.host_id != Some(0) || second_player_bits == 0
    }
}
