use bevy::prelude::{
    Add, App, Commands, Entity, Name, On, Plugin, PreStartup, Query, Single, With,
};

use lightyear::connection::client_of::ClientOf;
use lightyear::prelude::{
    Connected, ControlledBy, Disconnected, LinkOf, LocalAddr, RemoteId, ReplicationSender,
    SendUpdatesMode,
    server::{NetcodeConfig, NetcodeServer, ServerUdpIo, Start},
};
use shared::protocol::{LobbyState, PlayerId};
use shared::{SEND_INTERVAL, SERVER_BIND_ADDR, SHARED_SETTINGS};

pub struct ServerNetworkPlugin;

impl Plugin for ServerNetworkPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreStartup, startup_server);
        app.add_observer(handle_new_client);
        app.add_observer(handle_disconnected);
        app.add_observer(handle_connected);
    }
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
            // DeltaManager::default(), // Enable delta compression
        ))
        .id();

    commands.trigger(Start {
        entity: server_entity,
    });
}

fn handle_new_client(trigger: On<Add, LinkOf>, mut commands: Commands) {
    commands
        .entity(trigger.entity)
        .insert(ReplicationSender::new(
            SEND_INTERVAL,
            SendUpdatesMode::SinceLastAck,
            false,
        ));
}

fn handle_connected(
    trigger: On<Add, Connected>,
    query: Query<&RemoteId, With<ClientOf>>,
    lobby_query: Single<&mut LobbyState>,
    mut commands: Commands,
) {
    let Ok(client_id) = query.get(trigger.entity) else {
        return;
    };

    let client_id_bits = client_id.0.to_bits();

    commands
        .entity(trigger.entity)
        .insert(Name::from(format!("Client_{}", client_id_bits)));

    let mut lobby_state = lobby_query;
    if !lobby_state.players.contains(&client_id_bits) {
        lobby_state.players.push(client_id_bits);

        if lobby_state.players.len() == 1 {
            lobby_state.host_id = client_id_bits;
        }
    }
}

fn handle_disconnected(
    trigger: On<Add, Disconnected>,
    query: Query<&RemoteId, With<ClientOf>>,
    lobby_query: Single<&mut LobbyState>,
    player_query: Query<(Entity, &ControlledBy), With<PlayerId>>,
    mut commands: Commands,
) {
    let Ok(client_id) = query.get(trigger.entity) else {
        return;
    };

    let client_id_bits = client_id.0.to_bits();

    for (player_entity, controlled_by) in player_query.iter() {
        if controlled_by.owner == trigger.entity {
            commands.entity(player_entity).despawn();
        }
    }

    let mut lobby_state = lobby_query;
    if let Some(pos) = lobby_state
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
