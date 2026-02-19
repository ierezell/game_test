use bevy::prelude::{
    App, Assets, Commands, CommandsStatesExt, Entity, IntoScheduleConfigs, Mesh, Plugin, Query,
    Res, ResMut, Single, StandardMaterial, Update, error,
};

use lightyear::prelude::{
    Connected, MessageReceiver, NetworkTarget, RemoteId, Replicate, Server,
    ServerMultiMessageSender,
};

use crate::ServerGameState;

use shared::protocol::{
    GameSeed, HostStartGameEvent, LevelSeed, LobbyControlChannel, LobbyState, StartLoadingGameEvent,
};

pub struct ServerLobbyPlugin;

#[derive(bevy::prelude::Resource, Clone, Copy, Debug, Default)]
pub struct AutoStartOnLobbyReady(pub bool);

impl Plugin for ServerLobbyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            keep_lobby_state_replicating
                .run_if(bevy::state::condition::in_state(ServerGameState::Lobby)),
        );
        app.add_systems(
            Update,
            host_start_game_event.run_if(bevy::state::condition::in_state(ServerGameState::Lobby)),
        );
        app.add_systems(
            Update,
            auto_start_game_when_lobby_ready
                .run_if(bevy::state::condition::in_state(ServerGameState::Lobby)),
        );
    }
}

fn keep_lobby_state_replicating(
    lobby_query: Query<(Entity, &LobbyState)>,
    mut commands: Commands,
) {
    for (lobby_entity, lobby_state) in &lobby_query {
        commands
            .entity(lobby_entity)
            .insert((
                lobby_state.clone(),
                Replicate::to_clients(NetworkTarget::All),
            ));
    }
}

fn transition_to_loading(
    commands: &mut Commands,
    sender: &mut ServerMultiMessageSender,
    server: &Server,
) {
    println!("DEBUG: Server transitioning to Loading state");
    commands.spawn(GameSeed { seed: 42 });
    commands.spawn((
        LevelSeed { seed: 42 },
        Replicate::to_clients(NetworkTarget::All),
    ));
    commands.set_state(ServerGameState::Loading);
    sender
        .send::<StartLoadingGameEvent, LobbyControlChannel>(
            &StartLoadingGameEvent { start: true },
            server,
            &NetworkTarget::All,
        )
        .unwrap_or_else(|e| {
            error!("Failed to send message: {:?}", e);
        });
}

fn host_start_game_event(
    mut message_receiver_query: Query<(&RemoteId, &mut MessageReceiver<HostStartGameEvent>), bevy::prelude::With<Connected>>,
    mut sender: ServerMultiMessageSender,
    server: Single<&Server>,
    mut commands: Commands,
    server_state: Res<bevy::prelude::State<ServerGameState>>,
    _meshes: ResMut<Assets<Mesh>>,
    _materials: Option<ResMut<Assets<StandardMaterial>>>,
) {
    if server_state.get() != &ServerGameState::Lobby {
        return;
    }

    let mut trigger = false;
    for (remote_id, mut message_receiver) in message_receiver_query.iter_mut() {
        // There is one message receiver per connected client...
        if message_receiver.has_messages() {
            println!("DEBUG: Server received HostStartGameEvent from {:?}", remote_id.0);
            trigger = true;
            message_receiver.receive().for_each(drop);
        }
    }

    if trigger {
        transition_to_loading(&mut commands, &mut sender, server.into_inner());
    }
}

fn auto_start_game_when_lobby_ready(
    auto_start: Option<Res<AutoStartOnLobbyReady>>,
    lobby_state: Query<&LobbyState>,
    mut sender: ServerMultiMessageSender,
    server: Single<&Server>,
    mut commands: Commands,
) {
    let enabled = auto_start.map(|resource| resource.0).unwrap_or(false);
    if !enabled {
        return;
    }

    let Ok(lobby) = lobby_state.single() else {
        return;
    };

    if lobby.players.len() >= 2 {
        transition_to_loading(&mut commands, &mut sender, server.into_inner());
    }
}
