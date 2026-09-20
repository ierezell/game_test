use avian3d::prelude::{SpatialQuery, SpatialQueryFilter};
use bevy::prelude::*;
use lightyear::prelude::AppComponentExt;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use crate::level::generation::{IndexedItemKind, LevelGraph, TerminalNetwork, ZoneId};
use crate::protocol::TerminalInteractionRequest;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash, Reflect, Default)]
pub enum ReactorState {
    #[default]
    Off,
    Starting,
    Online,
    Stabilizing,
    Critical,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash, Reflect, Default)]
pub enum UplinkState {
    #[default]
    Disconnected,
    Connecting,
    Uploading,
    Complete,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq, Reflect)]
pub enum TerminalMode {
    #[default]
    Navigation,
    Reactor,
    Uplink,
    Extraction,
}

#[derive(Component, Serialize, Deserialize, Clone, Debug, Reflect)]
#[reflect(Component)]
pub struct TerminalState {
    pub terminal_id: String,
    pub zone_id: ZoneId,
    pub mode: TerminalMode,
    pub active: bool,
    pub unlocked_keycards: HashSet<String>,
    pub objective_progress: u32,
    pub reactor_state: ReactorState,
    pub uplink_state: UplinkState,
    pub has_extracted: bool,
    pub extract_timer: f32,
    pub downloaded_files: HashSet<String>,
    pub completed_objectives: HashSet<String>,
}

impl Default for TerminalState {
    fn default() -> Self {
        Self {
            terminal_id: String::new(),
            zone_id: ZoneId::default(),
            mode: TerminalMode::default(),
            active: true,
            unlocked_keycards: HashSet::default(),
            objective_progress: 0,
            reactor_state: ReactorState::default(),
            uplink_state: UplinkState::default(),
            has_extracted: false,
            extract_timer: 0.0,
            downloaded_files: HashSet::default(),
            completed_objectives: HashSet::default(),
        }
    }
}

#[derive(Component, Serialize, Deserialize, Clone, Debug, Reflect)]
#[reflect(Component)]
pub struct TerminalConsole {
    pub terminal_id: String,
    pub zone_id: ZoneId,
    pub description: String,
}

pub const TERMINAL_INTERACTION_RANGE: f32 = 3.0;
pub const TERMINAL_INTERACTION_COOLDOWN: f32 = 0.5;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TerminalInteractionError {
    SessionNotPlayable,
    InvalidPosition,
    OutOfRange,
    LineOfSightBlocked,
    Cooldown,
    DuplicateRequest,
    TerminalInactive,
}

#[derive(Component, Clone, Debug, Default, PartialEq, Reflect, Serialize, Deserialize)]
pub struct TerminalInteractionState {
    pub last_interaction_time: Option<f32>,
    pub last_request_key: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
pub struct TerminalRequestKey {
    pub terminal_id: String,
    pub command: String,
}

impl TerminalRequestKey {
    pub fn from_request(request: &TerminalInteractionRequest) -> Self {
        Self {
            terminal_id: request.terminal_id.clone(),
            command: request.command.full_command(),
        }
    }
    pub fn as_string(&self) -> String {
        format!("{}::{}", self.terminal_id, self.command)
    }
}

pub fn validate_terminal_interaction(
    player_position: Vec3,
    terminal_position: Vec3,
    session_is_playable: bool,
    cooldown_ready: bool,
    line_of_sight_clear: bool,
) -> Result<(), TerminalInteractionError> {
    if !session_is_playable {
        return Err(TerminalInteractionError::SessionNotPlayable);
    }

    if !player_position.is_finite() || !terminal_position.is_finite() {
        return Err(TerminalInteractionError::InvalidPosition);
    }

    if player_position.distance_squared(terminal_position)
        > TERMINAL_INTERACTION_RANGE * TERMINAL_INTERACTION_RANGE
    {
        return Err(TerminalInteractionError::OutOfRange);
    }

    if !line_of_sight_clear {
        return Err(TerminalInteractionError::LineOfSightBlocked);
    }

    if !cooldown_ready {
        return Err(TerminalInteractionError::Cooldown);
    }

    Ok(())
}

pub fn check_line_of_sight(
    spatial_query: &SpatialQuery,
    player_entity: Entity,
    player_position: Vec3,
    terminal_entity: Entity,
    terminal_position: Vec3,
) -> bool {
    let direction = terminal_position - player_position;
    let distance = direction.length();
    if distance < 1e-6 {
        return true;
    }
    let dir = match Dir3::new(direction) {
        Ok(d) => d,
        Err(_) => return false,
    };
    let filter =
        SpatialQueryFilter::default().with_excluded_entities([player_entity, terminal_entity]);
    spatial_query
        .cast_ray(player_position, dir, distance, false, &filter)
        .is_none()
}

pub fn is_cooldown_ready(
    state: &TerminalInteractionState,
    current_time: f32,
    cooldown_duration: f32,
) -> bool {
    match state.last_interaction_time {
        Some(last_time) => current_time - last_time >= cooldown_duration,
        None => true,
    }
}

pub fn is_duplicate_request(
    state: &TerminalInteractionState,
    request: &TerminalInteractionRequest,
    current_time: f32,
    cooldown_duration: f32,
) -> bool {
    let Some(last_time) = state.last_interaction_time else {
        return false;
    };
    if current_time - last_time >= cooldown_duration {
        return false;
    }
    let key = TerminalRequestKey::from_request(request).as_string();
    state.last_request_key.as_deref() == Some(key.as_str())
}

pub fn record_interaction(
    state: &mut TerminalInteractionState,
    request: &TerminalInteractionRequest,
    current_time: f32,
) {
    state.last_interaction_time = Some(current_time);
    state.last_request_key = Some(TerminalRequestKey::from_request(request).as_string());
}

impl TerminalConsole {
    pub fn new(terminal_id: &str, zone_id: ZoneId, description: &str) -> Self {
        Self {
            terminal_id: terminal_id.to_string(),
            zone_id,
            description: description.to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct TerminalCommand {
    pub command: String,
    pub args: Vec<String>,
}

impl TerminalCommand {
    pub fn new(cmd: &str) -> Self {
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        if parts.is_empty() {
            return Self {
                command: String::new(),
                args: Vec::new(),
            };
        }
        Self {
            command: parts[0].to_uppercase(),
            args: parts[1..].iter().map(|s| s.to_string()).collect(),
        }
    }

    pub fn full_command(&self) -> String {
        if self.args.is_empty() {
            self.command.clone()
        } else {
            format!("{} {}", self.command, self.args.join(" "))
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TerminalResponse {
    pub success: bool,
    pub output: String,
    pub triggers_reactor: bool,
    pub triggers_uplink: bool,
    pub triggers_extraction: bool,
    pub error: Option<String>,
}

impl TerminalResponse {
    pub fn ok(output: &str) -> Self {
        Self {
            success: true,
            output: output.to_string(),
            triggers_reactor: false,
            triggers_uplink: false,
            triggers_extraction: false,
            error: None,
        }
    }

    pub fn fail(msg: &str) -> Self {
        Self {
            success: false,
            output: String::new(),
            triggers_reactor: false,
            triggers_uplink: false,
            triggers_extraction: false,
            error: Some(msg.to_string()),
        }
    }
}

#[derive(Resource, Clone, Debug, Default)]
pub struct TerminalNetworkResource {
    pub network: TerminalNetwork,
}

impl TerminalNetworkResource {
    pub fn from_graph(graph: &LevelGraph) -> Self {
        Self {
            network: graph.terminal_network.clone(),
        }
    }
}

pub struct TerminalPlugin;

impl Plugin for TerminalPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<TerminalConsole>()
            .register_type::<TerminalState>()
            .register_type::<TerminalInteractionState>();

        app.component::<TerminalConsole>().replicate();
        app.component::<TerminalState>().replicate();

        app.add_systems(FixedUpdate, extraction_sequence_system);
    }
}

pub fn parse_terminal_input(input: &str) -> TerminalCommand {
    TerminalCommand::new(input)
}

pub fn execute_terminal_command(
    command: &TerminalCommand,
    state: &mut TerminalState,
    network: &TerminalNetwork,
) -> TerminalResponse {
    if command.command.is_empty() {
        return TerminalResponse::ok("READY.");
    }

    match command.command.as_str() {
        "LIST" => {
            let items: Vec<String> = network
                .items
                .iter()
                .map(|item| format!("{} [{}] - ZONE {}", item.id, item.kind, item.zone))
                .collect();
            let output = format!("{} items indexed.\n{}", items.len(), items.join("\n"));
            TerminalResponse::ok(&output)
        }
        "QUERY" => {
            if command.args.is_empty() {
                return TerminalResponse::ok("Usage: QUERY <SEARCH_TERM>");
            }
            let term = command.args[0].to_lowercase();
            let results: Vec<String> = network
                .items
                .iter()
                .filter(|item| {
                    item.id.to_lowercase().contains(&term)
                        || item.kind.to_string().to_lowercase().contains(&term)
                        || item.label.to_lowercase().contains(&term)
                })
                .map(|item| format!("{} [{}] - ZONE {}", item.id, item.kind, item.zone))
                .collect();
            if results.is_empty() {
                TerminalResponse::ok(&format!("No items match '{}'", term))
            } else {
                TerminalResponse::ok(&format!(
                    "Found {} item(s):\n{}",
                    results.len(),
                    results.join("\n")
                ))
            }
        }
        "PING" => {
            if command.args.is_empty() {
                return TerminalResponse::ok("Usage: PING <ZONE_ID>");
            }
            if let Ok(zone_num) = command.args[0].parse::<u32>() {
                let zone = ZoneId(zone_num);
                let found = network.items.iter().any(|item| item.zone == zone);
                if found {
                    TerminalResponse::ok(&format!("Pinging Zone {} - items detected", zone_num))
                } else {
                    TerminalResponse::ok(&format!("Pinging Zone {} - clear", zone_num))
                }
            } else {
                TerminalResponse::fail(&format!("Invalid zone ID: {}", command.args[0]))
            }
        }
        "UNLOCK" => {
            if command.args.is_empty() {
                return TerminalResponse::ok("Usage: UNLOCK <KEYCARD_ID>");
            }
            let keycard_id = command.args[0].to_uppercase();
            if !network
                .items
                .iter()
                .any(|item| item.id == keycard_id && item.kind == IndexedItemKind::Keycard)
            {
                return TerminalResponse::fail(&format!("Unknown keycard: {}", command.args[0]));
            }
            state.unlocked_keycards.insert(keycard_id.clone());
            TerminalResponse::ok(&format!("Security doors unlocked: {}", keycard_id))
        }
        "DOWNLOAD" => {
            if command.args.is_empty() {
                return TerminalResponse::ok("Usage: DOWNLOAD <FILE_ID>");
            }
            let file_id = command.args[0].to_uppercase();
            if state.downloaded_files.contains(&file_id) {
                return TerminalResponse::fail("File already downloaded");
            }
            if !network.items.iter().any(|item| item.id == file_id) {
                return TerminalResponse::fail(&format!("Unknown file: {}", file_id));
            }
            state.downloaded_files.insert(file_id.clone());
            TerminalResponse::ok(&format!("Downloaded: {}", file_id))
        }
        "REACTOR_START" => {
            if state.reactor_state != ReactorState::Off {
                return TerminalResponse::fail("Reactor already started");
            }
            state.reactor_state = ReactorState::Starting;
            state.mode = TerminalMode::Reactor;
            TerminalResponse {
                success: true,
                output: "REACTOR: Initiating startup sequence...".to_string(),
                triggers_reactor: true,
                ..TerminalResponse::ok("")
            }
        }
        "REACTOR_STABILIZE" => {
            if state.reactor_state != ReactorState::Starting {
                return TerminalResponse::fail("Reactor not in startup sequence");
            }
            state.reactor_state = ReactorState::Online;
            TerminalResponse::ok("REACTOR: Online. Output nominal.")
        }
        "REACTOR_CONTAIN" => {
            if state.reactor_state != ReactorState::Critical {
                return TerminalResponse::fail("Reactor not in critical state");
            }
            state.reactor_state = ReactorState::Stabilizing;
            TerminalResponse::ok("REACTOR: Containment initiated. Standing by...")
        }
        "UPLINK_CONNECT" => {
            if state.uplink_state != UplinkState::Disconnected {
                return TerminalResponse::fail("Uplink already initiated");
            }
            state.uplink_state = UplinkState::Connecting;
            state.mode = TerminalMode::Uplink;
            TerminalResponse {
                success: true,
                output: "UPLINK: Establishing connection...".to_string(),
                triggers_uplink: true,
                ..TerminalResponse::ok("")
            }
        }
        "UPLINK_UPLOAD" => {
            if state.uplink_state != UplinkState::Connecting {
                return TerminalResponse::fail("Uplink not connected");
            }
            state.uplink_state = UplinkState::Uploading;
            TerminalResponse::ok("UPLINK: Uploading data...")
        }
        "UPLINK_COMPLETE" => {
            if state.uplink_state != UplinkState::Uploading {
                return TerminalResponse::fail("Uplink not uploading");
            }
            state.uplink_state = UplinkState::Complete;
            state.completed_objectives.insert("uplink".to_string());
            TerminalResponse::ok("UPLINK: Transfer complete. Objective secured.")
        }
        "EXTRACT" => {
            if state.has_extracted {
                return TerminalResponse::fail("Extraction already called");
            }
            state.has_extracted = true;
            state.extract_timer = 90.0;
            state.mode = TerminalMode::Extraction;
            TerminalResponse {
                success: true,
                output: "EXTRACTION: Sequence initiated. Defend for 90 seconds.".to_string(),
                triggers_extraction: true,
                ..TerminalResponse::ok("")
            }
        }
        "STATUS" => TerminalResponse::ok(&format!(
            "Terminal {} | Zone: {:?} | Mode: {:?} | Reactor: {:?} | Uplink: {:?}\nUnlocked: {:?} | Downloaded: {} file(s)\nExtract: {}",
            state.terminal_id,
            state.zone_id,
            state.mode,
            state.reactor_state,
            state.uplink_state,
            state.unlocked_keycards,
            state.downloaded_files.len(),
            if state.has_extracted {
                "ACTIVE"
            } else {
                "READY"
            }
        )),
        _ => TerminalResponse::fail(&format!(
            "Unknown command: {}. Type LIST for available commands.",
            command.command
        )),
    }
}

fn extraction_sequence_system(time: Res<Time>, mut query: Query<&mut TerminalState>) {
    let dt = time.delta_secs();
    for mut state in query.iter_mut() {
        if state.extract_timer > 0.0 {
            state.extract_timer -= dt;
            if state.extract_timer <= 0.0 {
                state.extract_timer = 0.0;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::level::generation::{
        IndexedItem, IndexedItemKind, LevelGraph, TerminalNetwork, ZoneId,
    };

    fn test_network() -> TerminalNetwork {
        TerminalNetwork {
            items: vec![
                IndexedItem {
                    id: "KEY_RED_842".to_string(),
                    kind: IndexedItemKind::Keycard,
                    zone: ZoneId(42),
                    label: "Red Keycard".to_string(),
                },
                IndexedItem {
                    id: "OBJ_DATA_CORE".to_string(),
                    kind: IndexedItemKind::Objective,
                    zone: ZoneId(99),
                    label: "Data Core".to_string(),
                },
                IndexedItem {
                    id: "TERM_ZONE_42".to_string(),
                    kind: IndexedItemKind::Tool,
                    zone: ZoneId(42),
                    label: "Zone 42 Terminal".to_string(),
                },
            ],
        }
    }

    fn test_state() -> TerminalState {
        TerminalState {
            terminal_id: "T001".to_string(),
            zone_id: ZoneId(1),
            ..Default::default()
        }
    }

    #[test]
    fn terminal_interaction_accepts_playable_in_range_request() {
        assert_eq!(
            validate_terminal_interaction(Vec3::ZERO, Vec3::new(2.0, 0.0, 0.0), true, true, true),
            Ok(())
        );
    }

    #[test]
    fn terminal_interaction_rejects_non_playable_session() {
        assert_eq!(
            validate_terminal_interaction(Vec3::ZERO, Vec3::ZERO, false, true, true),
            Err(TerminalInteractionError::SessionNotPlayable)
        );
    }

    #[test]
    fn terminal_interaction_rejects_invalid_positions() {
        assert_eq!(
            validate_terminal_interaction(Vec3::NAN, Vec3::ZERO, true, true, true),
            Err(TerminalInteractionError::InvalidPosition)
        );
    }

    #[test]
    fn terminal_interaction_rejects_out_of_range_request() {
        assert_eq!(
            validate_terminal_interaction(
                Vec3::ZERO,
                Vec3::new(TERMINAL_INTERACTION_RANGE + 0.1, 0.0, 0.0),
                true,
                true,
                true
            ),
            Err(TerminalInteractionError::OutOfRange)
        );
    }

    #[test]
    fn terminal_interaction_rejects_blocked_line_of_sight() {
        assert_eq!(
            validate_terminal_interaction(Vec3::ZERO, Vec3::ZERO, true, true, false),
            Err(TerminalInteractionError::LineOfSightBlocked)
        );
    }

    #[test]
    fn terminal_interaction_rejects_cooldown_request() {
        assert_eq!(
            validate_terminal_interaction(Vec3::ZERO, Vec3::ZERO, true, false, true),
            Err(TerminalInteractionError::Cooldown)
        );
    }

    #[test]
    fn is_cooldown_ready_returns_true_when_never_used() {
        let state = TerminalInteractionState::default();
        assert!(is_cooldown_ready(
            &state,
            0.0,
            TERMINAL_INTERACTION_COOLDOWN
        ));
    }

    #[test]
    fn is_cooldown_ready_returns_false_within_cooldown_window() {
        let state = TerminalInteractionState {
            last_interaction_time: Some(0.0),
            ..Default::default()
        };
        assert!(!is_cooldown_ready(
            &state,
            TERMINAL_INTERACTION_COOLDOWN - 0.01,
            TERMINAL_INTERACTION_COOLDOWN
        ));
    }

    #[test]
    fn is_cooldown_ready_returns_true_after_expiry() {
        let state = TerminalInteractionState {
            last_interaction_time: Some(0.0),
            ..Default::default()
        };
        assert!(is_cooldown_ready(
            &state,
            TERMINAL_INTERACTION_COOLDOWN + 0.01,
            TERMINAL_INTERACTION_COOLDOWN
        ));
    }

    #[test]
    fn is_duplicate_request_detects_same_command() {
        let state = TerminalInteractionState {
            last_interaction_time: Some(1.0),
            last_request_key: Some("TERM_A::UNLOCK KEY_RED".to_string()),
        };
        let request = TerminalInteractionRequest {
            terminal_id: "TERM_A".to_string(),
            command: TerminalCommand::new("UNLOCK KEY_RED"),
        };
        assert!(is_duplicate_request(
            &state,
            &request,
            1.0 + TERMINAL_INTERACTION_COOLDOWN - 0.01,
            TERMINAL_INTERACTION_COOLDOWN
        ));
        let request2 = TerminalInteractionRequest {
            terminal_id: "TERM_A".to_string(),
            command: TerminalCommand::new("LIST"),
        };
        assert!(!is_duplicate_request(
            &state,
            &request2,
            1.0 + TERMINAL_INTERACTION_COOLDOWN - 0.01,
            TERMINAL_INTERACTION_COOLDOWN
        ));
    }

    #[test]
    fn record_interaction_updates_cooldown_and_key() {
        let mut state = TerminalInteractionState::default();
        let request = TerminalInteractionRequest {
            terminal_id: "TERM_X".to_string(),
            command: TerminalCommand::new("STATUS"),
        };
        record_interaction(&mut state, &request, 10.0);
        assert_eq!(state.last_interaction_time, Some(10.0));
        assert_eq!(state.last_request_key, Some("TERM_X::STATUS".to_string()));
    }

    #[test]
    fn terminal_state_defaults_to_active() {
        let state = TerminalState::default();
        assert!(state.active);
    }

    #[test]
    fn parse_terminal_input_extracts_command_and_args() {
        let cmd = TerminalCommand::new("QUERY KEY_RED");
        assert_eq!(cmd.command, "QUERY");
        assert_eq!(cmd.args, vec!["KEY_RED".to_string()]);

        let cmd = TerminalCommand::new("  list  ");
        assert_eq!(cmd.command, "LIST");
        assert!(cmd.args.is_empty());

        let cmd = TerminalCommand::new("");
        assert!(cmd.command.is_empty());
    }

    #[test]
    fn execute_command_list_shows_all_items() {
        let network = test_network();
        let state = test_state();
        let cmd = TerminalCommand::new("LIST");
        let response = execute_terminal_command(&cmd, &mut state.clone(), &network);

        assert!(response.success);
        assert!(response.output.contains("KEY_RED_842"));
        assert!(response.output.contains("OBJ_DATA_CORE"));
        assert!(response.output.contains("3 items indexed"));
    }

    #[test]
    fn execute_command_query_finds_matching_items() {
        let network = test_network();
        let state = test_state();
        let cmd = TerminalCommand::new("QUERY KEY");
        let response = execute_terminal_command(&cmd, &mut state.clone(), &network);

        assert!(response.success);
        assert!(response.output.contains("KEY_RED_842"));
        assert!(!response.output.contains("OBJ_DATA_CORE"));
    }

    #[test]
    fn execute_command_query_case_insensitive() {
        let network = test_network();
        let state = test_state();
        let cmd = TerminalCommand::new("QUERY key_red");
        let response = execute_terminal_command(&cmd, &mut state.clone(), &network);

        assert!(response.success);
        assert!(response.output.contains("KEY_RED_842"));
    }

    #[test]
    fn execute_command_query_no_results() {
        let network = test_network();
        let state = test_state();
        let cmd = TerminalCommand::new("QUERY NONEXISTENT");
        let response = execute_terminal_command(&cmd, &mut state.clone(), &network);

        assert!(response.success);
        assert!(response.output.contains("No items match"));
    }

    #[test]
    fn execute_command_ping_returns_zone_info() {
        let network = test_network();
        let state = test_state();
        let cmd = TerminalCommand::new("PING 42");
        let response = execute_terminal_command(&cmd, &mut state.clone(), &network);

        assert!(response.success);
        assert!(response.output.contains("Zone 42"));
        assert!(response.output.contains("items detected"));
    }

    #[test]
    fn execute_command_ping_clear_zone() {
        let network = test_network();
        let state = test_state();
        let cmd = TerminalCommand::new("PING 50");
        let response = execute_terminal_command(&cmd, &mut state.clone(), &network);

        assert!(response.success);
        assert!(response.output.contains("clear"));
    }

    #[test]
    fn execute_command_ping_invalid_zone() {
        let network = test_network();
        let state = test_state();
        let cmd = TerminalCommand::new("PING ABC");
        let response = execute_terminal_command(&cmd, &mut state.clone(), &network);

        assert!(!response.success);
        assert!(response.error.unwrap().contains("Invalid zone ID"));
    }

    #[test]
    fn execute_command_unlock_adds_keycard() {
        let network = test_network();
        let mut state = test_state();
        let cmd = TerminalCommand::new("UNLOCK KEY_RED_842");
        let response = execute_terminal_command(&cmd, &mut state, &network);

        assert!(response.success);
        assert!(state.unlocked_keycards.contains("KEY_RED_842"));
    }

    #[test]
    fn execute_command_unlock_unknown_keycard_fails() {
        let network = test_network();
        let mut state = test_state();
        let cmd = TerminalCommand::new("UNLOCK FORGED_KEYCARD");
        let response = execute_terminal_command(&cmd, &mut state, &network);

        assert!(!response.success);
        assert!(response.error.unwrap().contains("Unknown keycard"));
        assert!(state.unlocked_keycards.is_empty());
    }

    #[test]
    fn execute_command_download_records_file() {
        let network = test_network();
        let mut state = test_state();
        let cmd = TerminalCommand::new("DOWNLOAD OBJ_DATA_CORE");
        let response = execute_terminal_command(&cmd, &mut state, &network);

        assert!(response.success);
        assert!(state.downloaded_files.contains("OBJ_DATA_CORE"));
    }

    #[test]
    fn execute_command_download_unknown_file_fails() {
        let network = test_network();
        let state = test_state();
        let cmd = TerminalCommand::new("DOWNLOAD UNKNOWN_FILE");
        let response = execute_terminal_command(&cmd, &mut state.clone(), &network);

        assert!(!response.success);
    }

    #[test]
    fn execute_command_download_duplicate_fails() {
        let network = test_network();
        let mut state = test_state();
        let cmd = TerminalCommand::new("DOWNLOAD OBJ_DATA_CORE");
        let _ = execute_terminal_command(&cmd, &mut state, &network);
        let response = execute_terminal_command(&cmd, &mut state, &network);

        assert!(!response.success);
        assert!(response.error.unwrap().contains("already downloaded"));
    }

    #[test]
    fn execute_command_extract_triggers_extraction() {
        let network = test_network();
        let mut state = test_state();
        let cmd = TerminalCommand::new("EXTRACT");
        let response = execute_terminal_command(&cmd, &mut state, &network);

        assert!(response.success);
        assert!(response.triggers_extraction);
        assert!((state.extract_timer - 90.0).abs() < 0.001);
        assert!(state.has_extracted);
    }

    #[test]
    fn execute_command_extract_duplicate_fails() {
        let network = test_network();
        let mut state = test_state();
        let cmd = TerminalCommand::new("EXTRACT");
        let _ = execute_terminal_command(&cmd, &mut state, &network);
        let response = execute_terminal_command(&cmd, &mut state, &network);

        assert!(!response.success);
        assert!(response.error.unwrap().contains("already called"));
    }

    #[test]
    fn execute_reactor_sequence_commands() {
        let network = test_network();
        let mut state = test_state();

        let cmd = TerminalCommand::new("REACTOR_START");
        let resp = execute_terminal_command(&cmd, &mut state, &network);
        assert!(resp.success);
        assert!(resp.triggers_reactor);
        assert_eq!(state.reactor_state, ReactorState::Starting);

        let cmd = TerminalCommand::new("REACTOR_STABILIZE");
        let resp = execute_terminal_command(&cmd, &mut state, &network);
        assert!(resp.success);
        assert_eq!(state.reactor_state, ReactorState::Online);
    }

    #[test]
    fn execute_reactor_stabilize_without_start_fails() {
        let network = test_network();
        let mut state = test_state();
        let cmd = TerminalCommand::new("REACTOR_STABILIZE");
        let resp = execute_terminal_command(&cmd, &mut state, &network);

        assert!(!resp.success);
    }

    #[test]
    fn execute_uplink_sequence_commands() {
        let network = test_network();
        let mut state = test_state();

        let cmd = TerminalCommand::new("UPLINK_CONNECT");
        let resp = execute_terminal_command(&cmd, &mut state, &network);
        assert!(resp.success);
        assert!(resp.triggers_uplink);
        assert_eq!(state.uplink_state, UplinkState::Connecting);

        let cmd = TerminalCommand::new("UPLINK_UPLOAD");
        let resp = execute_terminal_command(&cmd, &mut state, &network);
        assert!(resp.success);
        assert_eq!(state.uplink_state, UplinkState::Uploading);

        let cmd = TerminalCommand::new("UPLINK_COMPLETE");
        let resp = execute_terminal_command(&cmd, &mut state, &network);
        assert!(resp.success);
        assert_eq!(state.uplink_state, UplinkState::Complete);
        assert!(state.completed_objectives.contains("uplink"));
    }

    #[test]
    fn execute_uplink_upload_without_connect_fails() {
        let network = test_network();
        let mut state = test_state();
        let cmd = TerminalCommand::new("UPLINK_UPLOAD");
        let resp = execute_terminal_command(&cmd, &mut state, &network);

        assert!(!resp.success);
    }

    #[test]
    fn execute_unknown_command_fails() {
        let network = test_network();
        let state = test_state();
        let cmd = TerminalCommand::new("INVALID_COMMAND");
        let response = execute_terminal_command(&cmd, &mut state.clone(), &network);

        assert!(!response.success);
        assert!(response.error.unwrap().contains("Unknown command"));
    }

    #[test]
    fn execute_status_command_returns_info() {
        let network = test_network();
        let state = test_state();
        let cmd = TerminalCommand::new("STATUS");
        let response = execute_terminal_command(&cmd, &mut state.clone(), &network);

        assert!(response.success);
        assert!(response.output.contains("T001"));
        assert!(response.output.contains("READY"));
    }

    #[test]
    fn terminal_response_ok_and_fail_constructors() {
        let ok = TerminalResponse::ok("Success!");
        assert!(ok.success);
        assert_eq!(ok.output, "Success!");
        assert!(ok.error.is_none());

        let fail = TerminalResponse::fail("Error!");
        assert!(!fail.success);
        assert!(fail.error.unwrap().contains("Error!"));
    }

    #[test]
    fn reactor_and_uplink_state_defaults() {
        assert_eq!(ReactorState::default(), ReactorState::Off);
        assert_eq!(UplinkState::default(), UplinkState::Disconnected);
        assert_eq!(TerminalMode::default(), TerminalMode::Navigation);
    }

    #[test]
    fn terminal_network_resource_from_graph() {
        let network = test_network();
        let graph = LevelGraph {
            terminal_network: network,
            ..Default::default()
        };
        let resource = TerminalNetworkResource::from_graph(&graph);
        assert_eq!(resource.network.items.len(), 3);
    }
}
