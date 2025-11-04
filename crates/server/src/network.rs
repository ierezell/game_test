use bevy::prelude::{Add, App, Commands, Name, On, Plugin, Res, Startup, info};
use lightyear::prelude::{
    LinkOf, LocalAddr, ReplicationSender, SendUpdatesMode,
    server::{NetcodeConfig, Start},
};
use lightyear::{netcode::NetcodeServer, prelude::server::ServerUdpIo};
use shared::NetTransport;
use shared::{SEND_INTERVAL, SERVER_BIND_ADDR, SHARED_SETTINGS};

pub struct NetworkPlugin;

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(NetTransport::Udp);
        app.add_systems(Startup, startup_server);
        app.add_observer(handle_new_client);
    }
}

fn startup_server(mut commands: Commands, transport: Res<NetTransport>) {
    info!("Starting server with transport: {:?}", transport.as_ref());
    match transport.as_ref() {
        NetTransport::Udp => {
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
        NetTransport::Tcp => {
            info!("TCP transport is not yet implemented.");
            // TCP support can be added for different network configurations
        }
    }
}

fn handle_new_client(trigger: On<Add, LinkOf>, mut commands: Commands) {
    info!("🎉 New client connected: {:?}", trigger.entity);

    commands.entity(trigger.entity).insert((
        ReplicationSender::new(SEND_INTERVAL, SendUpdatesMode::SinceLastAck, false),
        Name::from(format!("Client-{}", trigger.entity)),
    ));
}
