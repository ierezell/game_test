use bevy::prelude::{Add, App, Commands, Entity, Name, On, Plugin, PreStartup, Query, Single, With, info};
use lightyear::connection::client_of::ClientOf;
use lightyear::prelude::{
    Connected, ControlledBy, Disconnected, LinkOf, LocalAddr, RemoteId, ReplicationSender, SendUpdatesMode,
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
    info!("Starting server");

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

    info!(
        "Server started on {} with protocol_id: {:x}",
        SERVER_BIND_ADDR, SHARED_SETTINGS.protocol_id
    );
}

fn handle_new_client(trigger: On<Add, LinkOf>, mut commands: Commands) {
    info!("🎉 New client connected: {:?}", trigger.entity);

    commands
        .entity(trigger.entity)
        .insert((ReplicationSender::new(
            SEND_INTERVAL,
            SendUpdatesMode::SinceLastAck,
            false,
        ),));
}

fn handle_connected(
    trigger: On<Add, Connected>,
    query: Query<&RemoteId, With<ClientOf>>,
    lobby_query: Single<&mut LobbyState>,
    mut commands: Commands,
) {
    let Ok(client_id) = query.get(trigger.entity) else {
        info!(
            "❌ Failed to get RemoteId for connected entity {:?}",
            trigger.entity
        );
        return;
    };

    let client_id_bits = client_id.0.to_bits();

    info!(
        "✅ Client connected with remote-id {:?}. Adding to lobby.",
        client_id
    );

    commands
        .entity(trigger.entity)
        .insert(Name::from(format!("Client_{}", client_id_bits)));

    let mut lobby_state = lobby_query;
    if !lobby_state.players.contains(&client_id_bits) {
        lobby_state.players.push(client_id_bits);

        // Set first player as host
        if lobby_state.players.len() == 1 {
            lobby_state.host_id = client_id_bits;
            info!("👑 Player {} is now the host", client_id_bits);
        }
    }
    info!("🎪 Lobby now has {} players", lobby_state.players.len());
    info!("👥 Player added to lobby, waiting for game start to spawn entities");
}

fn handle_disconnected(
    trigger: On<Add, Disconnected>,
    query: Query<&RemoteId, With<ClientOf>>,
    lobby_query: Single<&mut LobbyState>,
    player_query: Query<(Entity, &ControlledBy), With<PlayerId>>,
    mut commands: Commands,
) {
    let Ok(client_id) = query.get(trigger.entity) else {
        info!(
            "❌ Failed to get RemoteId for disconnected entity {:?}",
            trigger.entity
        );
        return;
    };

    let client_id_bits = client_id.0.to_bits();

    info!(
        "❌ Client disconnected with remote-id {:?}. Removing from lobby.",
        client_id
    );

    // Despawn all player entities controlled by this client
    for (player_entity, controlled_by) in player_query.iter() {
        if controlled_by.owner == trigger.entity {
            info!(
                "🗑️ Despawning player entity {:?} controlled by disconnected client",
                player_entity
            );
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
        info!("🎪 Lobby now has {} players", lobby_state.players.len());

        // If the host disconnected, assign a new host
        if lobby_state.host_id == client_id_bits {
            if let Some(&new_host_id) = lobby_state.players.first() {
                lobby_state.host_id = new_host_id;
                info!("👑 Player {} is now the host", new_host_id);
            } else {
                lobby_state.host_id = 0; // No host
                info!("👑 No players left in the lobby");
            }
        }
    }
}
