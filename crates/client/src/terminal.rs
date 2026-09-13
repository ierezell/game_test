use avian3d::prelude::Position;
use bevy::prelude::{
    App, ButtonInput, IntoScheduleConfigs, KeyCode, Plugin, Query, Res, Resource, Update, Vec3,
    With, in_state,
};
use lightyear::prelude::{Client, Controlled, MessageSender};
use shared::inputs::PLAYER_CAPSULE_HEIGHT;
use shared::protocol::{LobbyControlChannel, TerminalInteractionRequest};
use shared::terminal::{TERMINAL_INTERACTION_RANGE, TerminalCommand, TerminalConsole};

use crate::ClientGameState;

#[derive(Resource, Clone, Debug)]
pub struct TerminalCommandInput(pub String);

impl Default for TerminalCommandInput {
    fn default() -> Self {
        Self("STATUS".to_string())
    }
}

pub struct ClientTerminalPlugin;

impl Plugin for ClientTerminalPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TerminalCommandInput>();
        app.add_systems(
            Update,
            send_terminal_interaction.run_if(in_state(ClientGameState::Playing)),
        );
    }
}

fn send_terminal_interaction(
    keys: Res<ButtonInput<KeyCode>>,
    command_input: Res<TerminalCommandInput>,
    player_query: Query<&Position, (With<Controlled>, With<shared::protocol::CharacterMarker>)>,
    terminal_query: Query<(&TerminalConsole, &Position)>,
    mut sender_query: Query<&mut MessageSender<TerminalInteractionRequest>, With<Client>>,
) {
    if !keys.just_pressed(KeyCode::KeyE) {
        return;
    }

    let Ok(player_position) = player_query.single() else {
        return;
    };

    let terminals: Vec<(String, Vec3)> = terminal_query
        .iter()
        .map(|(console, position)| {
            (
                console.terminal_id.clone(),
                position.0 + Vec3::Y * PLAYER_CAPSULE_HEIGHT * 0.5,
            )
        })
        .collect();

    let Some(terminal_id) =
        nearest_terminal_in_range(player_position.0, &terminals, TERMINAL_INTERACTION_RANGE)
    else {
        return;
    };

    let Some(mut sender) = sender_query.iter_mut().next() else {
        return;
    };

    sender.send::<LobbyControlChannel>(TerminalInteractionRequest {
        terminal_id,
        command: TerminalCommand::new(&command_input.0),
    });
}

fn nearest_terminal_in_range(
    player_position: Vec3,
    terminals: &[(String, Vec3)],
    interaction_range: f32,
) -> Option<String> {
    terminals
        .iter()
        .filter_map(|(terminal_id, terminal_position)| {
            let distance_squared = player_position.distance_squared(*terminal_position);
            (distance_squared <= interaction_range * interaction_range)
                .then_some((distance_squared, terminal_id))
        })
        .min_by(|(distance_a, _), (distance_b, _)| distance_a.total_cmp(distance_b))
        .map(|(_, terminal_id)| terminal_id.clone())
}

#[cfg(test)]
mod tests {
    use super::nearest_terminal_in_range;
    use bevy::prelude::Vec3;

    #[test]
    fn nearest_terminal_prefers_closest_in_range_target() {
        let terminals = vec![
            ("far".to_string(), Vec3::new(2.5, 0.0, 0.0)),
            ("near".to_string(), Vec3::new(1.0, 0.0, 0.0)),
        ];

        assert_eq!(
            nearest_terminal_in_range(Vec3::ZERO, &terminals, 3.0),
            Some("near".to_string())
        );
    }

    #[test]
    fn nearest_terminal_rejects_targets_outside_range() {
        let terminals = vec![("far".to_string(), Vec3::new(3.1, 0.0, 0.0))];

        assert_eq!(nearest_terminal_in_range(Vec3::ZERO, &terminals, 3.0), None);
    }
}
