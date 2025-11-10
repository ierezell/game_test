use bevy::app::Startup;
use bevy::prelude::{
    App, Assets, Commands, CommandsStatesExt, Mesh, Name, Plugin, ResMut, Single, StandardMaterial,
    Update, error, info,
};

use lightyear::prelude::{
    MessageReceiver, MetadataChannel, NetworkTarget, Replicate, Server, ServerMultiMessageSender,
};

use crate::ServerGameState;
use shared::level::create_static::setup_static_level;

use shared::protocol::{GameSeed, HostStartGameEvent, LobbyState, StartLoadingGameEvent};

pub struct ServerLobbyPlugin;

impl Plugin for ServerLobbyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, host_start_game_event);
        app.add_systems(Startup, init_lobby);
    }
}

fn init_lobby(mut commands: Commands) {
    commands.spawn((
        LobbyState {
            players: Vec::new(),
            host_id: 0,
        },
        Replicate::to_clients(NetworkTarget::All),
        Name::from("LobbyState"),
    ));
    info!("🎯 SERVER: Initialized LobbyState resource");
}

fn host_start_game_event(
    mut message_receiver: Single<&mut MessageReceiver<HostStartGameEvent>>,
    mut sender: ServerMultiMessageSender,
    server: Single<&Server>,
    mut commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    materials: Option<ResMut<Assets<StandardMaterial>>>,
) {
    if message_receiver.has_messages() {
        info!("🚀 SERVER: Received HostStartGameEvent from client");
        commands.spawn(GameSeed { seed: 42 });
        commands.set_state(ServerGameState::Loading);
        sender
            .send::<StartLoadingGameEvent, MetadataChannel>(
                &StartLoadingGameEvent,
                server.into_inner(),
                &NetworkTarget::All,
            )
            .unwrap_or_else(|e| {
                error!("Failed to send message: {:?}", e);
            });

        setup_static_level(commands.reborrow(), meshes, materials, None);
        commands.set_state(ServerGameState::Playing);
    }
    message_receiver.receive().for_each(drop);
}
