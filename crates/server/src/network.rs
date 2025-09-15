use avian3d::prelude::Position;
use bevy::math::Vec3;
use bevy::prelude::{
    App, Bundle, Color, Commands, CommandsStatesExt, Entity, Name, OnAdd, OnEnter, OnRemove,
    Plugin, Query, Res, ResMut, Startup, State, Trigger, Update, With, default, error, info, warn,
};
use common::NetTransport;
use common::protocol::{PlayerColor, PlayerId};
use common::{PRIVATE_KEY, PROTOCOL_ID, SERVER_BIND_ADDR};
use lightyear::prelude::server::ClientOf;
use lightyear::prelude::server::NetcodeConfig;
use lightyear::prelude::server::Start;
use lightyear::prelude::*;
use lightyear::{netcode::Key, netcode::NetcodeServer, prelude::server::ServerUdpIo};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::Duration;
pub struct NetworkPlugin;

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        // app.add_systems(Update, log_connection_events);
        app.insert_resource(NetTransport::Udp);
        app.add_systems(Startup, startup_server);

        app.add_observer(handle_new_client);
        app.add_observer(handle_connected_client);
    }
}

fn startup_server(mut commands: Commands, transport: Res<NetTransport>) {
    info!("Starting server with transport: {:?}", transport.as_ref());
    match transport.as_ref() {
        NetTransport::Udp => {
            // TODO: Could some of it be common with client ?? (as it needs same transport)
            // Maybe a util like pub fn get_client_server_config(transport)
            let netcode_config = NetcodeConfig {
                num_disconnect_packets: 10,
                keep_alive_send_rate: 1.0 / 10.0,
                client_timeout_secs: 15,
                protocol_id: PROTOCOL_ID,
                private_key: PRIVATE_KEY,
            };

            let server_entity = commands
                .spawn((
                    NetcodeServer::new(netcode_config),
                    LocalAddr(SERVER_BIND_ADDR),
                    ServerUdpIo::default(),
                ))
                .id();

            commands.trigger_targets(Start, server_entity);

            info!(
                "Server started on {} with protocol_id: {:x}",
                SERVER_BIND_ADDR, PROTOCOL_ID
            );
        } // TODO: Add other transports when implemented
    }
}

/// System to handle new connections
fn handle_new_client(trigger: Trigger<OnAdd, LinkOf>, mut commands: Commands) {
    info!("🎉 New client connected: {:?}", trigger.target());

    commands.entity(trigger.target()).insert((
        ReplicationSender::new(
            Duration::from_millis(100),
            SendUpdatesMode::SinceLastAck,
            false,
        ),
        Name::from(format!("Client-{}", trigger.target())),
    ));
}

fn handle_connected_client(
    trigger: Trigger<OnAdd, Connected>,
    query: Query<&RemoteId, With<ClientOf>>,
    mut commands: Commands,
) {
    if let Ok(client_id) = query.get(trigger.target()) {
        info!("🤝 Client connected: {:?}", client_id.0);
        info!("👤 Spawning player for client {:?}", client_id.0);

        let entity = commands
            .spawn((
                Position(Vec3::ZERO),
                PlayerColor(color_from_remote_id(client_id.0)),
                Name::from(format!("Player {}", client_id.0)),
                Replicate::to_clients(NetworkTarget::All),
            ))
            .id();

        info!(
            "Create player entity {:?} for client {:?}",
            entity, client_id
        );
    }
}

/// Generate a unique color from a remote_id (u64)
fn color_from_remote_id(remote_id: PeerId) -> Color {
    // Simple hash to float [0,1] for RGB channels
    let id = remote_id.to_bits();
    let r = ((id >> 16) & 0xFF) as f32 / 255.0;
    let g = ((id >> 8) & 0xFF) as f32 / 255.0;
    let b = (id & 0xFF) as f32 / 255.0;
    Color::srgb(r, g, b)
}
