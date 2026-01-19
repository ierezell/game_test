pub fn init(&mut self) {
    self.server_app.finish();
    self.server_app.cleanup();
    self.server_app.update();

    // Setup channels first
    for (i, client_app) in self.client_apps.iter_mut().enumerate() {
        client_app.finish();
        client_app.cleanup();

        let client_id = self.client_ids[i];
        let (client_channel, server_channel) = lightyear::crossbeam::CrossbeamIo::new_pair();

        client_app.insert_resource(::client::network::CrossbeamClientEndpoint(
            client_channel.clone(),
        ));

        // Server setup for this client
        let server_world = self.server_app.world_mut();
        let server_entity = server_world
            .query_filtered::<Entity, With<Name>>()
            .iter(server_world)
            .find(|e| {
                server_world
                    .get::<Name>(*e)
                    .map(|n| n.as_str() == "Server")
                    .unwrap_or(false)
            })
            .expect("Server entity not found");

        server_world.spawn((
            lightyear::prelude::LinkOf {
                server: server_entity,
            },
            lightyear::prelude::ReplicationSender::new(
                std::time::Duration::from_millis(10),
                lightyear::prelude::SendUpdatesMode::SinceLastAck,
                true,
            ),
            lightyear::prelude::ReplicationReceiver::default(),
            lightyear::prelude::Link::new(None),
            lightyear::prelude::PeerAddr(SocketAddr::new(
                std::net::IpAddr::V4(Ipv4Addr::LOCALHOST),
                client_id as u16,
            )),
            lightyear::prelude::Connected,
            server_channel,
            Name::new(format!("Client_{}_Link", client_id)),
            lightyear::prelude::RemoteId(lightyear::prelude::PeerId::Netcode(client_id)),
        ));

        for mut lobby_state in server_world
            .query::<&mut shared::protocol::LobbyState>()
            .iter_mut(server_world)
        {
            if !lobby_state.players.contains(&client_id) {
                lobby_state.players.push(client_id);
            }
        }

        // Manually spawn client networking entity (mimics start_connection_crossbeam)
        use lightyear::prelude::{Client, Connect, PredictionManager, ReplicationReceiver};

        let client_entity = client_app
            .world_mut()
            .spawn((
                Client::default(),
                client_channel,
                ReplicationReceiver::default(),
                PredictionManager::default(),
                Name::from(format!("Client {}", client_id)),
            ))
            .id();

        // Trigger Connect
        client_app.world_mut().commands().trigger(Connect {
            entity: client_entity,
        });
    }

    // NOW run updates after resources are in place
    for client_app in &mut self.client_apps {
        client_app.insert_state(ClientGameState::Lobby);
        client_app.update();
    }
}
