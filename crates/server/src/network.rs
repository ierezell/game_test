use bevy::prelude::{App, Commands, Name, OnAdd, Plugin, Res, Startup, Trigger, info};
use lightyear::prelude::server::NetcodeConfig;
use lightyear::prelude::server::Start;
use lightyear::prelude::*;
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
                client_timeout_secs: 3,
                protocol_id: SHARED_SETTINGS.protocol_id,
                private_key: SHARED_SETTINGS.private_key,
            };

            let server_entity = commands
                .spawn((
                    NetcodeServer::new(netcode_config),
                    LocalAddr(SERVER_BIND_ADDR),
                    ServerUdpIo::default(),
                    DeltaManager::default(), // Enable delta compression
                ))
                .id();

            commands.trigger_targets(Start, server_entity);

            info!(
                "Server started on {} with protocol_id: {:x}",
                SERVER_BIND_ADDR, SHARED_SETTINGS.protocol_id
            );
        } // TODO: Add other transports when implemented
    }
}

fn handle_new_client(trigger: Trigger<OnAdd, LinkOf>, mut commands: Commands) {
    info!("🎉 New client connected: {:?}", trigger.target());

    commands.entity(trigger.target()).insert((
        ReplicationSender::new(SEND_INTERVAL, SendUpdatesMode::SinceLastAck, false),
        Name::from(format!("Client-{}", trigger.target())),
    ));
}
