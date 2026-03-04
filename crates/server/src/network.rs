use bevy::prelude::{
    Add, App, Commands, Entity, Name, On, Plugin, PreStartup, Query, Res, Single, State, Update,
    With, Without, info,
};
use std::collections::HashSet;
use std::time::Duration;

use lightyear::connection::client_of::ClientOf;
use lightyear::prelude::{
    Client, Connected, ControlledBy, DeltaManager, Disconnected, Link, LinkOf, Linked, LocalAddr,
    LocalId, NetworkTarget, PeerId, RemoteId, Replicate, ReplicationReceiver,
    ReplicationSender, SendUpdatesMode, Server,
    ServerMultiMessageSender,
    server::{NetcodeConfig, NetcodeServer, ServerUdpIo, Start, Started},
};
use shared::protocol::{LobbyControlChannel, LobbyState, PlayerId, StartLoadingGameEvent};
use shared::{SERVER_BIND_ADDR, SHARED_SETTINGS};

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
        println!("ServerNetworkPlugin: building with mode {:?}", network_mode);

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

    let mut existing_remote_ids: HashSet<PeerId> = existing_clientofs.iter().map(|id| id.0).collect();

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
            Link::new(None),
            Linked,
            RemoteId(client_peer_id),
            LocalId(PeerId::Server),
            Name::from(format!("LocalClientOf_{:?}", client_peer_id)),
        ));

        existing_remote_ids.insert(client_peer_id);
    }
}

fn startup_server_crossbeam(mut commands: Commands) {
    // In Crossbeam mode, connections are manually managed via LinkOf entities.
    // We just need a Server entity to exist to satisfy queries/Start event.
    let server_entity = commands
        .spawn((Name::new("Server"), Server::default(), Started))
        .id();
    println!(
        "ServerNetworkPlugin: spawned Server entity {:?}",
        server_entity
    );
    commands.trigger(Start {
        entity: server_entity,
    });
}

fn startup_server_local(mut commands: Commands) {
    // In Local mode (HostServer), server and client are in the same app.
    // Lightyear handles local communication via HostServer/HostClient automatically.
    let server_entity = commands
        .spawn((Name::new("Server"), Server::default(), Started))
        .id();
    println!(
        "ServerNetworkPlugin: spawned Server entity {:?} in Local mode (HostServer)",
        server_entity
    );
    commands.trigger(Start {
        entity: server_entity,
    });
}

fn startup_server(mut commands: Commands) {
    let netcode_config = NetcodeConfig {
        num_disconnect_packets: 10,
        keep_alive_send_rate: 1.0 / 10.0,
        client_timeout_secs: 10,
        protocol_id: SHARED_SETTINGS.protocol_id,
        private_key: SHARED_SETTINGS.private_key,
    };
    let server_entity = commands
        .spawn((
            NetcodeServer::new(netcode_config),
            LocalAddr(SERVER_BIND_ADDR),
            ServerUdpIo::default(),
            DeltaManager::default(),
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
        ReplicationSender::new(Duration::ZERO, SendUpdatesMode::SinceLastAck, true),
        ReplicationReceiver::default(),
    ));

    // Get or create the lobby state
    if let Some((lobby_entity, mut lobby_state)) = lobby_query.iter_mut().next() {
        // Lobby exists, add player if not already present
        if !lobby_state.players.contains(&client_id_bits) {
            println!(
                "DEBUG: Server accepted connection from Client_{}",
                client_id_bits
            );
            lobby_state.players.push(client_id_bits);
            commands
                .entity(lobby_entity)
                .insert(Replicate::to_clients(NetworkTarget::All));

            if lobby_state.players.len() == 1 {
                println!("DEBUG: Client_{} became host", client_id_bits);
                lobby_state.host_id = client_id_bits;
            }

            // If the game is already in progress, send the StartLoadingGameEvent to the newly connected client
            if *server_state.get() == ServerGameState::Playing {
                println!(
                    "DEBUG: Game already started, sending StartLoadingGameEvent to late-joining Client_{}",
                    client_id_bits
                );

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
            println!("DEBUG: Client_{} already in lobby", client_id_bits);
        }
    } else {
        // No lobby exists, create it with this first client as host
        println!(
            "DEBUG: Creating lobby with Client_{} as first player and host",
            client_id_bits
        );
        commands.spawn((
            LobbyState {
                players: vec![client_id_bits],
                host_id: client_id_bits,
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

        if lobby_state.host_id == client_id_bits {
            if let Some(&new_host_id) = lobby_state.players.first() {
                lobby_state.host_id = new_host_id;
            } else {
                lobby_state.host_id = 0;
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

    if lobby_state.host_id != 0 && !connected_ids.contains(&lobby_state.host_id) {
        lobby_state.host_id = lobby_state.players.first().copied().unwrap_or(0);
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
            host_id: 2,
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
        assert_eq!(lobby.host_id, 1);
        assert!(app.world().entities().contains(player_1));
        assert!(!app.world().entities().contains(player_2));
    }
}
