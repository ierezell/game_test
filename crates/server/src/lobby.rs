use bevy::prelude::{
    App, Assets, Commands, CommandsStatesExt, Mesh, Plugin, Query, ResMut, Single,
    StandardMaterial, Update, error,
};

use lightyear::prelude::{
    MessageReceiver, MetadataChannel, NetworkTarget, Server, ServerMultiMessageSender,
};

use crate::ServerGameState;

use shared::protocol::{GameSeed, HostStartGameEvent, StartLoadingGameEvent};

pub struct ServerLobbyPlugin;

impl Plugin for ServerLobbyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, host_start_game_event);
    }
}

fn host_start_game_event(
    mut message_receiver_query: Query<&mut MessageReceiver<HostStartGameEvent>>,
    mut sender: ServerMultiMessageSender,
    server: Single<&Server>,
    mut commands: Commands,
    _meshes: ResMut<Assets<Mesh>>,
    _materials: Option<ResMut<Assets<StandardMaterial>>>,
) {
    let mut trigger = false;
    for mut message_receiver in message_receiver_query.iter_mut() {
        // There is one message receiver per connected client...
        if message_receiver.has_messages() {
            println!("DEBUG: Server received HostStartGameEvent");
            trigger = true;
            message_receiver.receive().for_each(drop);
        }
    }

    if trigger {
        println!("DEBUG: Server transitioning to Loading state");
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

        // Stay in Loading state - transition to Playing happens after gym environment loads
        // This is handled by the entities plugin OnEnter(Loading) systems
    }
}
