use bevy::prelude::{
    App, Assets, Commands, CommandsStatesExt, Entity, IntoScheduleConfigs, Mesh, Plugin, Query,
    Res, ResMut, Single, StandardMaterial, Update, With, error,
};

use lightyear::prelude::{
    Connected, ControlledBy, MessageReceiver, NetworkTarget, RemoteId, Replicate, Server,
    ServerMultiMessageSender, server::ClientOf,
};
use avian3d::prelude::Position;

use crate::ServerGameState;

use shared::debug::debug_println;
use shared::protocol::{
    GameSeed, HostStartGameEvent, LevelSeed, LobbyControlChannel, LobbyState,
    StartLoadingGameEvent, TerminalInteractionRequest,
};
use shared::terminal::{TerminalConsole, TerminalNetworkResource, TerminalState, execute_terminal_command, validate_terminal_interaction};

pub struct ServerLobbyPlugin;

#[derive(bevy::prelude::Resource, Clone, Copy, Debug, Default)]
pub struct AutoStartOnLobbyReady(pub bool);

impl Plugin for ServerLobbyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            host_start_game_event.run_if(bevy::state::condition::in_state(ServerGameState::Lobby)),
        );
        app.add_systems(
            Update,
            auto_start_game_when_lobby_ready
                .run_if(bevy::state::condition::in_state(ServerGameState::Lobby)),
        );
        app.add_systems(
            Update,
            handle_terminal_interaction_requests
                .run_if(bevy::state::condition::in_state(ServerGameState::Playing)),
        );
    }
}

fn transition_to_loading(
    commands: &mut Commands,
    sender: &mut ServerMultiMessageSender,
    server: &Server,
) {
    debug_println(format_args!("DEBUG: Server transitioning to Loading state"));
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
    mut message_receiver_query: Query<
        (&RemoteId, &mut MessageReceiver<HostStartGameEvent>),
        bevy::prelude::With<Connected>,
    >,
    lobby_query: Query<&LobbyState>,
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

    let Ok(lobby) = lobby_query.single() else {
        return;
    };

    let mut trigger = false;
    for (remote_id, mut message_receiver) in message_receiver_query.iter_mut() {
        // There is one message receiver per connected client...
        if message_receiver.has_messages() {
            debug_println(format_args!(
                "DEBUG: Server received HostStartGameEvent from {:?}",
                remote_id.0
            ));
            for request in message_receiver.receive() {
                trigger |= is_authorized_host_start(&request, remote_id, lobby);
            }
        }
    }

    if trigger {
        transition_to_loading(&mut commands, &mut sender, server.into_inner());
    }
}

fn is_authorized_host_start(
    request: &HostStartGameEvent,
    sender: &RemoteId,
    lobby: &LobbyState,
) -> bool {
    request.requested && sender.0.to_bits() == lobby.host_id
}

fn handle_terminal_interaction_requests(
    mut receiver_query: Query<
        (Entity, &mut MessageReceiver<TerminalInteractionRequest>),
        (With<ClientOf>, With<Connected>),
    >,
    player_query: Query<(&Position, &ControlledBy)>,
    mut terminal_query: Query<(&TerminalConsole, &Position, &mut TerminalState)>,
    terminal_network: Option<Res<TerminalNetworkResource>>,
) {
    let Some(terminal_network) = terminal_network else {
        return;
    };

    for (client_entity, mut receiver) in receiver_query.iter_mut() {
        for request in receiver.receive() {
            let Some((player_position, _)) = player_query
                .iter()
                .find(|(_, controlled_by)| controlled_by.owner == client_entity)
            else {
                continue;
            };

            let Some((console, terminal_position, mut terminal_state)) = terminal_query
                .iter_mut()
                .find(|(console, _, _)| console.terminal_id == request.terminal_id)
            else {
                continue;
            };

            process_terminal_interaction_request(
                &request,
                player_position,
                console,
                terminal_position,
                &mut terminal_state,
                &terminal_network.network,
            );
        }
    }
}

fn process_terminal_interaction_request(
    request: &TerminalInteractionRequest,
    player_position: &Position,
    console: &TerminalConsole,
    terminal_position: &Position,
    terminal_state: &mut TerminalState,
    terminal_network: &shared::level::generation::TerminalNetwork,
) -> bool {
    if request.terminal_id != console.terminal_id
        || validate_terminal_interaction(player_position.0, terminal_position.0, true, true).is_err()
    {
        return false;
    }

    let response = execute_terminal_command(&request.command, terminal_state, terminal_network);
    bevy::log::debug!(
        "Executed terminal command {} for {}: success={}",
        request.command.full_command(),
        console.terminal_id,
        response.success
    );
    response.success
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

    if !lobby.players.is_empty() {
        transition_to_loading(&mut commands, &mut sender, server.into_inner());
    }
}

#[cfg(test)]
mod tests {
    use super::{is_authorized_host_start, process_terminal_interaction_request};
    use avian3d::prelude::Position;
    use bevy::prelude::Vec3;
    use lightyear::prelude::{PeerId, RemoteId};
    use shared::protocol::{HostStartGameEvent, LobbyState, TerminalInteractionRequest};
    use shared::terminal::{TerminalCommand, TerminalConsole, TerminalState};
    use shared::level::generation::{IndexedItem, IndexedItemKind, TerminalNetwork, ZoneId};

    fn lobby() -> LobbyState {
        LobbyState {
            players: vec![1, 2],
            host_id: 1,
        }
    }

    #[test]
    fn host_can_request_start() {
        let request = HostStartGameEvent { requested: true };
        let sender = RemoteId(PeerId::Netcode(1));

        assert!(is_authorized_host_start(&request, &sender, &lobby()));
    }

    #[test]
    fn non_host_cannot_request_start() {
        let request = HostStartGameEvent { requested: true };
        let sender = RemoteId(PeerId::Netcode(2));

        assert!(!is_authorized_host_start(&request, &sender, &lobby()));
    }

    #[test]
    fn unrequested_start_is_rejected() {
        let request = HostStartGameEvent { requested: false };
        let sender = RemoteId(PeerId::Netcode(1));

        assert!(!is_authorized_host_start(&request, &sender, &lobby()));
    }

    #[test]
    fn terminal_request_executes_for_matching_in_range_terminal() {
        let request = TerminalInteractionRequest {
            terminal_id: "TERM_1".to_string(),
            command: TerminalCommand::new("UNLOCK KEY_RED_842"),
        };
        let console = TerminalConsole::new("TERM_1", ZoneId(1), "Test terminal");
        let mut state = TerminalState {
            terminal_id: "TERM_1".to_string(),
            zone_id: ZoneId(1),
            ..Default::default()
        };

        let network = TerminalNetwork {
            items: vec![IndexedItem {
                id: "KEY_RED_842".to_string(),
                kind: IndexedItemKind::Keycard,
                zone: ZoneId(1),
                label: "Red keycard".to_string(),
            }],
        };

        assert!(process_terminal_interaction_request(
            &request,
            &Position::new(Vec3::ZERO),
            &console,
            &Position::new(Vec3::new(1.0, 0.0, 0.0)),
            &mut state,
            &network,
        ));
        assert!(state.unlocked_keycards.contains("KEY_RED_842"));
    }

    #[test]
    fn terminal_request_rejects_wrong_target_and_out_of_range_player() {
        let mut request = TerminalInteractionRequest {
            terminal_id: "TERM_WRONG".to_string(),
            command: TerminalCommand::new("UNLOCK KEY_RED_842"),
        };
        let console = TerminalConsole::new("TERM_1", ZoneId(1), "Test terminal");
        let mut state = TerminalState {
            terminal_id: "TERM_1".to_string(),
            zone_id: ZoneId(1),
            ..Default::default()
        };

        assert!(!process_terminal_interaction_request(
            &request,
            &Position::new(Vec3::ZERO),
            &console,
            &Position::new(Vec3::new(1.0, 0.0, 0.0)),
            &mut state,
            &TerminalNetwork::default(),
        ));

        request.terminal_id = "TERM_1".to_string();
        assert!(!process_terminal_interaction_request(
            &request,
            &Position::new(Vec3::ZERO),
            &console,
            &Position::new(Vec3::new(10.0, 0.0, 0.0)),
            &mut state,
            &TerminalNetwork::default(),
        ));
        assert!(state.unlocked_keycards.is_empty());
    }
}
