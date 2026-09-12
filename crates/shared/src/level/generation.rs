use avian3d::prelude::{Collider, Position, RigidBody, Rotation};
use bevy::prelude::*;
use rand::rngs::StdRng;
use rand::{RngExt, SeedableRng};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};

use crate::navigation::NavigationObstacle;

pub(crate) const WALL_THICKNESS: f32 = 0.5;
const DOOR_OPENING_WIDTH: f32 = 6.0;
const DOOR_EDGE_MARGIN: f32 = 1.0;
const MIN_WALL_SEGMENT_LENGTH: f32 = 0.5;
pub(crate) const WALL_SIDE_EAST: usize = 0;
pub(crate) const WALL_SIDE_WEST: usize = 1;
pub(crate) const WALL_SIDE_NORTH: usize = 2;
pub(crate) const WALL_SIDE_SOUTH: usize = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WallSide {
    East,
    West,
    North,
    South,
}

impl WallSide {
    fn as_index(self) -> usize {
        match self {
            Self::East => WALL_SIDE_EAST,
            Self::West => WALL_SIDE_WEST,
            Self::North => WALL_SIDE_NORTH,
            Self::South => WALL_SIDE_SOUTH,
        }
    }

    fn from_local_direction(local_direction: Vec3) -> Self {
        if local_direction.x.abs() >= local_direction.z.abs() {
            if local_direction.x >= 0.0 {
                Self::East
            } else {
                Self::West
            }
        } else if local_direction.z >= 0.0 {
            Self::North
        } else {
            Self::South
        }
    }
}

fn wall_half_span(zone: &Zone, side: WallSide) -> f32 {
    match side {
        WallSide::East | WallSide::West => zone.size.z * 0.5,
        WallSide::North | WallSide::South => zone.size.x * 0.5,
    }
}

fn opening_coord_for_side(local_door: Vec3, side: WallSide) -> f32 {
    match side {
        WallSide::East | WallSide::West => local_door.z,
        WallSide::North | WallSide::South => local_door.x,
    }
}

fn collect_zone_wall_openings(zone: &Zone, level_graph: &LevelGraph) -> [Vec<f32>; 4] {
    let mut openings: [Vec<f32>; 4] = std::array::from_fn(|_| Vec::new());

    for connection in &level_graph.connections {
        let maybe_other = if connection.from_zone == zone.id {
            Some(connection.to_zone)
        } else if connection.to_zone == zone.id {
            Some(connection.from_zone)
        } else {
            None
        };

        let Some(other_zone_id) = maybe_other else {
            continue;
        };
        let Some(other_zone) = level_graph.get_zone(other_zone_id) else {
            continue;
        };

        let local_direction = zone.rotation.inverse() * (other_zone.position - zone.position);
        let side = WallSide::from_local_direction(local_direction);

        let local_door = zone.rotation.inverse() * (connection.door_position - zone.position);
        let half_span = wall_half_span(zone, side);
        let max_coord = (half_span - DOOR_EDGE_MARGIN).max(0.0);
        let opening_coord = opening_coord_for_side(local_door, side).clamp(-max_coord, max_coord);

        openings[side.as_index()].push(opening_coord);
    }

    openings
}

fn build_wall_segments(
    half_span: f32,
    opening_centers: &[f32],
    opening_width: f32,
) -> Vec<(f32, f32)> {
    if half_span <= 0.0 {
        return Vec::new();
    }

    if opening_centers.is_empty() {
        return vec![(0.0, half_span * 2.0)];
    }

    let opening_half_width = opening_width * 0.5;
    let mut ranges: Vec<(f32, f32)> = opening_centers
        .iter()
        .map(|center| {
            (
                (center - opening_half_width).clamp(-half_span, half_span),
                (center + opening_half_width).clamp(-half_span, half_span),
            )
        })
        .filter(|(start, end)| end > start)
        .collect();

    if ranges.is_empty() {
        return vec![(0.0, half_span * 2.0)];
    }

    ranges.sort_by(|(a_start, _), (b_start, _)| {
        a_start
            .partial_cmp(b_start)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let mut merged_ranges: Vec<(f32, f32)> = Vec::with_capacity(ranges.len());
    for (start, end) in ranges {
        if let Some((_, prev_end)) = merged_ranges.last_mut()
            && start <= *prev_end
        {
            *prev_end = prev_end.max(end);
        } else {
            merged_ranges.push((start, end));
        }
    }

    let mut segments = Vec::new();
    let mut cursor = -half_span;

    for (open_start, open_end) in merged_ranges {
        let segment_length = open_start - cursor;
        if segment_length >= MIN_WALL_SEGMENT_LENGTH {
            let segment_center = cursor + segment_length * 0.5;
            segments.push((segment_center, segment_length));
        }
        cursor = cursor.max(open_end);
    }

    let tail_length = half_span - cursor;
    if tail_length >= MIN_WALL_SEGMENT_LENGTH {
        let tail_center = cursor + tail_length * 0.5;
        segments.push((tail_center, tail_length));
    }

    segments
}

pub(crate) fn collect_zone_wall_segments(
    zone: &Zone,
    level_graph: &LevelGraph,
) -> [Vec<(f32, f32)>; 4] {
    let openings = collect_zone_wall_openings(zone, level_graph);
    let mut segments: [Vec<(f32, f32)>; 4] = std::array::from_fn(|_| Vec::new());

    for side in [WallSide::East, WallSide::West, WallSide::North, WallSide::South] {
        segments[side.as_index()] = build_wall_segments(
            wall_half_span(zone, side),
            &openings[side.as_index()],
            DOOR_OPENING_WIDTH,
        );
    }

    segments
}

fn spawn_wall_segments_for_side(
    commands: &mut Commands,
    zone: &Zone,
    side: WallSide,
    segment_definitions: &[(f32, f32)],
) {
    let half_x = zone.size.x * 0.5;
    let half_z = zone.size.z * 0.5;

    let (wall_anchor, span_on_z) = match side {
        WallSide::East => (Vec3::new(half_x, zone.size.y * 0.5, 0.0), true),
        WallSide::West => (Vec3::new(-half_x, zone.size.y * 0.5, 0.0), true),
        WallSide::North => (Vec3::new(0.0, zone.size.y * 0.5, half_z), false),
        WallSide::South => (Vec3::new(0.0, zone.size.y * 0.5, -half_z), false),
    };

    for (segment_index, (segment_center, segment_length)) in segment_definitions.iter().enumerate() {
        let local_offset = if span_on_z {
            wall_anchor + Vec3::new(0.0, 0.0, *segment_center)
        } else {
            wall_anchor + Vec3::new(*segment_center, 0.0, 0.0)
        };

        let wall_size = if span_on_z {
            Vec3::new(WALL_THICKNESS, zone.size.y, *segment_length)
        } else {
            Vec3::new(*segment_length, zone.size.y, WALL_THICKNESS)
        };

        let world_position = zone.position + zone.rotation * local_offset;
        commands.spawn((
            RigidBody::Static,
            Collider::cuboid(wall_size.x, wall_size.y, wall_size.z),
            NavigationObstacle,
            Position::new(world_position),
            Rotation::from(zone.rotation),
            Transform::from_translation(world_position).with_rotation(zone.rotation),
            Name::new(format!(
                "Physics_Wall_{:?}_{}_Zone_{}",
                side, segment_index, zone.id.0
            )),
        ));
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ZoneId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZoneType {
    Hub,
    Corridor,
    Utility,
    Industrial,
    Objective,
    Storage,
}

impl ZoneType {
    pub fn size_multiplier(&self) -> f32 {
        match self {
            ZoneType::Hub => 2.0,
            ZoneType::Corridor => 0.5,
            ZoneType::Utility => 0.8,
            ZoneType::Industrial => 2.5,
            ZoneType::Objective => 1.5,
            ZoneType::Storage => 1.0,
        }
    }

    pub fn max_connections(&self) -> usize {
        match self {
            ZoneType::Hub => 5,
            ZoneType::Corridor => 2,
            ZoneType::Utility => 2,
            ZoneType::Industrial => 4,
            ZoneType::Objective => 2,
            ZoneType::Storage => 1,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Reflect)]
pub enum DoorType {
    Normal,
    Keycard,
    Alarm,
    Bulkhead,
}

impl DoorType {
    pub fn requires_keycard(self) -> bool {
        matches!(self, DoorType::Keycard)
    }

    pub fn is_defensive_hold(self) -> bool {
        matches!(self, DoorType::Alarm | DoorType::Bulkhead)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Reflect)]
pub enum KeycardColor {
    Red,
    Blue,
}

impl Default for KeycardColor {
    fn default() -> Self {
        Self::Red
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IndexedItemKind {
    Keycard,
    Objective,
    Ammo,
    Medical,
    Tool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexedItem {
    pub id: String,
    pub kind: IndexedItemKind,
    pub zone: ZoneId,
    pub label: String,
}

/// Pure-data representation of the virtual terminal network that every terminal
/// console in the level can answer `QUERY`/`PING`/`LIST` against. It is computed
/// from the expedition design so the layout stays deterministic for a given seed.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TerminalNetwork {
    pub items: Vec<IndexedItem>,
}

/// Per-zone resource budget. Items are concentrated in dead-ends to reward the
/// risk of straying off the beaten path, mirroring GTFO's resource starvation.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZoneResources {
    pub ammo_packs: u8,
    pub tool_packs: u8,
    pub med_packs: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize, Component)]
pub struct Zone {
    pub id: ZoneId,
    pub zone_type: ZoneType,
    pub position: Vec3,
    pub rotation: Quat,
    pub size: Vec3,
    pub connections: Vec<ZoneId>,
    pub is_built: bool,
    pub is_objective: bool,
    pub is_keycard: bool,
    pub hazard_level: u8,
    pub resources: ZoneResources,
}

impl Zone {
    pub fn new(id: ZoneId, zone_type: ZoneType, position: Vec3, rotation: Quat) -> Self {
        let base_size = 20.0;
        let multiplier = zone_type.size_multiplier();

        let size = match zone_type {
            ZoneType::Corridor => Vec3::new(base_size * 0.3, 10.0, base_size * 2.0),
            _ => Vec3::new(base_size * multiplier, 10.0, base_size * multiplier),
        };

        Self {
            id,
            zone_type,
            position,
            rotation,
            size,
            connections: Vec::new(),
            is_built: false,
            is_objective: false,
            is_keycard: false,
            hazard_level: 0,
            resources: ZoneResources::default(),
        }
    }

    pub fn is_leaf(&self, spawn: ZoneId) -> bool {
        self.id != spawn && self.connections.len() <= 1
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZoneConnection {
    pub from_zone: ZoneId,
    pub to_zone: ZoneId,
    pub door_position: Vec3,
    pub door_rotation: Quat,
    pub door_type: DoorType,
    pub required_keycard: Option<ZoneId>,
    pub is_critical_path: bool,
}

#[derive(Debug, Clone, Resource, Serialize, Deserialize)]
pub struct LevelConfig {
    pub seed: u64,
    pub target_zone_count: u32,
    pub min_zone_spacing: f32,
    pub max_depth: u32,
}

impl Default for LevelConfig {
    fn default() -> Self {
        Self {
            seed: 12345,
            target_zone_count: 15,
            min_zone_spacing: 30.0,
            max_depth: 10,
        }
    }
}

#[derive(Debug, Clone, Resource)]
pub struct LevelGraph {
    pub config: LevelConfig,
    pub zones: HashMap<ZoneId, Zone>,
    pub connections: Vec<ZoneConnection>,
    pub spawn_zone: ZoneId,
    pub objective_zones: Vec<ZoneId>,
    pub objective_zone: Option<ZoneId>,
    pub keycard_zone: Option<ZoneId>,
    pub keycard_color: KeycardColor,
    pub terminal_network: TerminalNetwork,
    pub horde_spawn_zones: Vec<ZoneId>,
    pub scan_node_zones: Vec<ZoneId>,
}

impl LevelGraph {
    pub fn new(config: LevelConfig) -> Self {
        Self {
            config,
            zones: HashMap::new(),
            connections: Vec::new(),
            spawn_zone: ZoneId(0),
            objective_zones: Vec::new(),
            objective_zone: None,
            keycard_zone: None,
            keycard_color: KeycardColor::default(),
            terminal_network: TerminalNetwork::default(),
            horde_spawn_zones: Vec::new(),
            scan_node_zones: Vec::new(),
        }
    }

    pub fn get_zone(&self, id: ZoneId) -> Option<&Zone> {
        self.zones.get(&id)
    }

    pub fn get_zone_mut(&mut self, id: ZoneId) -> Option<&mut Zone> {
        self.zones.get_mut(&id)
    }

    pub fn add_zone(&mut self, zone: Zone) {
        self.zones.insert(zone.id, zone);
    }

    pub fn add_connection(
        &mut self,
        from: ZoneId,
        to: ZoneId,
        door_position: Vec3,
        door_rotation: Quat,
        door_type: DoorType,
    ) {
        self.connections.push(ZoneConnection {
            from_zone: from,
            to_zone: to,
            door_position,
            door_rotation,
            door_type,
            required_keycard: None,
            is_critical_path: false,
        });

        if let Some(from_zone) = self.zones.get_mut(&from)
            && !from_zone.connections.contains(&to)
        {
            from_zone.connections.push(to);
        }
        if let Some(to_zone) = self.zones.get_mut(&to)
            && !to_zone.connections.contains(&from)
        {
            to_zone.connections.push(from);
        }
    }

    pub fn critical_path(&self, objective: ZoneId) -> Vec<ZoneId> {
        bfs_path(self, self.spawn_zone, objective).unwrap_or_default()
    }

    pub fn depth_map(&self) -> HashMap<ZoneId, u32> {
        zone_depths(self)
    }

    pub fn connections_of(&self, zone: ZoneId) -> impl Iterator<Item = &ZoneConnection> {
        self.connections
            .iter()
            .filter(move |c| c.from_zone == zone || c.to_zone == zone)
    }
}

pub fn generate_level(config: LevelConfig) -> LevelGraph {
    let mut rng = StdRng::seed_from_u64(config.seed);
    let mut graph = LevelGraph::new(config.clone());

    let spawn_zone = Zone::new(ZoneId(0), ZoneType::Hub, Vec3::ZERO, Quat::IDENTITY);
    graph.spawn_zone = spawn_zone.id;
    graph.add_zone(spawn_zone);

    let mut frontier: Vec<(ZoneId, u32)> = vec![(ZoneId(0), 0)]; // (zone_id, depth)
    let mut next_zone_id = 1u32;

    while graph.zones.len() < config.target_zone_count as usize && !frontier.is_empty() {
        let frontier_index = rng.random_range(0..frontier.len());
        let (current_zone_id, depth) = frontier[frontier_index];

        let Some(current_zone) = graph.get_zone(current_zone_id).cloned() else {
            warn!(
                "Skipping missing zone {:?} while generating level",
                current_zone_id
            );
            frontier.remove(frontier_index);
            continue;
        };
        let max_connections = current_zone.zone_type.max_connections();

        if depth >= config.max_depth || current_zone.connections.len() >= max_connections {
            frontier.remove(frontier_index);
            continue;
        }

        let remaining_connections = max_connections - current_zone.connections.len();
        let branch_count = if remaining_connections > 0 {
            rng.random_range(1..=remaining_connections.min(3))
        } else {
            0
        };

        if branch_count == 0 {
            frontier.remove(frontier_index);
            continue;
        }

        for _ in 0..branch_count {
            if graph.zones.len() >= config.target_zone_count as usize {
                break;
            }

            let zone_type = choose_zone_type(&mut rng, depth, config.max_depth);

            let new_position =
                calculate_zone_position(&graph, current_zone_id, &mut rng, config.min_zone_spacing);
            let current_pos = current_zone.position;

            let direction = (new_position - current_pos).normalize_or_zero();
            let zone_rotation = if zone_type == ZoneType::Corridor && direction != Vec3::ZERO {
                Quat::from_rotation_arc(Vec3::Z, direction)
            } else {
                Quat::IDENTITY
            };

            let new_zone = Zone::new(ZoneId(next_zone_id), zone_type, new_position, zone_rotation);
            let new_zone_id = new_zone.id;

            let door_position = (current_pos + new_position) * 0.5;
            let door_rotation = if direction != Vec3::ZERO {
                Quat::from_rotation_arc(Vec3::Z, direction)
            } else {
                Quat::IDENTITY
            };

            graph.add_zone(new_zone);
            graph.add_connection(
                current_zone_id,
                new_zone_id,
                door_position,
                door_rotation,
                DoorType::Normal,
            );

            if zone_type.max_connections() > 1 {
                frontier.push((new_zone_id, depth + 1));
            }

            if zone_type == ZoneType::Objective {
                graph.objective_zones.push(new_zone_id);
            }

            next_zone_id += 1;
        }

        if let Some(updated_zone) = graph.get_zone(current_zone_id)
            && updated_zone.connections.len() < max_connections
        {
            frontier.push((current_zone_id, depth));
        }
    }

    let mut design_rng = StdRng::seed_from_u64(config.seed);
    apply_expedition_design(&mut graph, &mut design_rng);

    info!(
        "Generated level with {} zones and {} connections",
        graph.zones.len(),
        graph.connections.len()
    );

    graph
}

fn adjacency_list(graph: &LevelGraph) -> HashMap<ZoneId, Vec<ZoneId>> {
    let mut adj: HashMap<ZoneId, Vec<ZoneId>> = HashMap::new();
    for zone_id in graph.zones.keys() {
        adj.entry(*zone_id).or_default();
    }
    for conn in &graph.connections {
        let entry = adj.entry(conn.from_zone).or_default();
        if !entry.contains(&conn.to_zone) {
            entry.push(conn.to_zone);
        }
        let entry = adj.entry(conn.to_zone).or_default();
        if !entry.contains(&conn.from_zone) {
            entry.push(conn.from_zone);
        }
    }
    adj
}

fn zone_depths(graph: &LevelGraph) -> HashMap<ZoneId, u32> {
    let adj = adjacency_list(graph);
    let mut depths: HashMap<ZoneId, u32> = HashMap::new();
    let mut queue = VecDeque::new();
    depths.insert(graph.spawn_zone, 0);
    queue.push_back(graph.spawn_zone);
    while let Some(current) = queue.pop_front() {
        let depth = *depths.get(&current).unwrap_or(&0);
        let Some(neighbors) = adj.get(&current) else {
            continue;
        };
        for neighbor in neighbors {
            if !depths.contains_key(neighbor) {
                depths.insert(*neighbor, depth + 1);
                queue.push_back(*neighbor);
            }
        }
    }
    depths
}

fn leaf_zones(graph: &LevelGraph) -> Vec<ZoneId> {
    graph
        .zones
        .values()
        .filter(|zone| zone.is_leaf(graph.spawn_zone))
        .map(|zone| zone.id)
        .collect()
}

fn bfs_path(graph: &LevelGraph, from: ZoneId, to: ZoneId) -> Option<Vec<ZoneId>> {
    let adj = adjacency_list(graph);
    let mut prev: HashMap<ZoneId, ZoneId> = HashMap::new();
    let mut queue = VecDeque::new();
    queue.push_back(from);
    prev.insert(from, from);

    while let Some(current) = queue.pop_front() {
        if current == to {
            break;
        }
        let Some(neighbors) = adj.get(&current) else {
            continue;
        };
        for neighbor in neighbors {
            if !prev.contains_key(neighbor) {
                prev.insert(*neighbor, current);
                queue.push_back(*neighbor);
            }
        }
    }

    if !prev.contains_key(&to) {
        return None;
    }

    let mut path = vec![to];
    let mut current = to;
    while current != from {
        let parent = *prev.get(&current)?;
        path.push(parent);
        current = parent;
    }
    path.reverse();
    Some(path)
}

fn connection_endpoints(conn: &ZoneConnection) -> (ZoneId, ZoneId) {
    (conn.from_zone, conn.to_zone)
}

fn set_path_edge_door(
    graph: &mut LevelGraph,
    critical_path: &[ZoneId],
    target: ZoneId,
    door_type: DoorType,
) {
    let Some(idx) = critical_path.iter().position(|z| *z == target) else {
        return;
    };
    let Some(prev) = idx.checked_sub(1).and_then(|i| critical_path.get(i)) else {
        return;
    };
    if let Some(conn) = find_connection_mut(&mut graph.connections, *prev, target) {
        if conn.door_type != DoorType::Keycard {
            conn.door_type = door_type;
        }
    }
}

fn find_connection_mut<'a>(
    connections: &'a mut [ZoneConnection],
    a: ZoneId,
    b: ZoneId,
) -> Option<&'a mut ZoneConnection> {
    for conn in connections.iter_mut() {
        let (f, t) = connection_endpoints(conn);
        if (f == a && t == b) || (f == b && t == a) {
            return Some(conn);
        }
    }
    None
}

fn color_name(color: KeycardColor) -> &'static str {
    match color {
        KeycardColor::Red => "RED",
        KeycardColor::Blue => "BLUE",
    }
}

/// Post-generation pass that layers GTFO-style expedition design onto the raw
/// zone graph: a deep-leaf objective, a keycard locked in a separate branch to
/// force backtracking, escalating door types along the critical path, hazard
/// density that grows downstream, a resource budget concentrated in dead-ends,
/// a terminal network index, scan-node hold positions and routed horde spawns.
pub fn apply_expedition_design(graph: &mut LevelGraph, rng: &mut StdRng) {
    if graph.zones.len() < 4 {
        return;
    }

    let depths = zone_depths(graph);
    let max_depth = depths.values().copied().max().unwrap_or(0);
    let leaves = leaf_zones(graph);

    if leaves.is_empty() {
        return;
    }

    let objective = leaves
        .iter()
        .max_by_key(|zone| depths.get(*zone).copied().unwrap_or(0))
        .copied()
        .unwrap();
    graph.objective_zone = Some(objective);
    if let Some(zone) = graph.get_zone_mut(objective) {
        zone.is_objective = true;
        if !graph.objective_zones.contains(&objective) {
            graph.objective_zones.push(objective);
        }
    }

    let critical_path = bfs_path(graph, graph.spawn_zone, objective).unwrap_or_default();
    let critical_set: HashSet<ZoneId> = critical_path.iter().copied().collect();

    for conn in &mut graph.connections {
        let (a, b) = connection_endpoints(conn);
        let on_critical = critical_set.contains(&a)
            && critical_set.contains(&b)
            && critical_path
                .windows(2)
                .any(|w| (w[0] == a && w[1] == b) || (w[0] == b && w[1] == a));
        conn.is_critical_path = on_critical;
    }

    let keycard = leaves
        .iter()
        .filter(|zone| !critical_set.contains(*zone))
        .max_by_key(|zone| depths.get(*zone).copied().unwrap_or(0))
        .copied();

    let keycard_zone: Option<ZoneId> = if let Some(kc) = keycard {
        graph.keycard_zone = Some(kc);
        graph.keycard_color = if kc.0.is_multiple_of(2) {
            KeycardColor::Blue
        } else {
            KeycardColor::Red
        };
        if let Some(zone) = graph.get_zone_mut(kc) {
            zone.is_keycard = true;
        }
        Some(kc)
    } else {
        None
    };

    let depth_threshold = max_depth / 2;
    for conn in &mut graph.connections {
        if !conn.is_critical_path {
            if conn.door_type == DoorType::Normal && rng.random_bool(0.18) {
                conn.door_type = DoorType::Alarm;
            }
            continue;
        }

        let deeper = conn.to_zone.0.max(conn.from_zone.0);
        let depth = depths.get(&ZoneId(deeper)).copied().unwrap_or(0);
        if depth >= depth_threshold && conn.door_type != DoorType::Keycard {
            conn.door_type = DoorType::Alarm;
        }
    }

    if let Some(kc) = keycard_zone {
        let attach = bfs_path(graph, kc, objective)
            .and_then(|path| {
                path.iter()
                    .find(|zone| critical_set.contains(zone))
                    .copied()
            })
            .unwrap_or(graph.spawn_zone);

        let attach_idx = critical_path.iter().position(|z| *z == attach);
        if let Some(idx) = attach_idx {
            if let Some(gate_target) = critical_path.get(idx + 1).copied() {
                if let Some(conn) =
                    find_connection_mut(&mut graph.connections, attach, gate_target)
                {
                    conn.door_type = DoorType::Keycard;
                    conn.required_keycard = Some(kc);
                }
            }
        }
    }

    set_path_edge_door(graph, &critical_path, objective, DoorType::Bulkhead);

    for conn in &mut graph.connections {
        if conn.door_type == DoorType::Bulkhead && conn.required_keycard.is_some() {
            conn.door_type = DoorType::Keycard;
        }
    }

    for zone in graph.zones.values_mut() {
        let depth = depths.get(&zone.id).copied().unwrap_or(0);
        zone.hazard_level = (depth as u8).min(5);
        zone.resources = assign_zone_resources(zone, &depths, objective, keycard_zone, rng);
    }

    let terminal_network = build_terminal_network(graph, &depths);
    graph.terminal_network = terminal_network;

    graph.horde_spawn_zones = horde_spawn_zones(&critical_path, objective);
    graph.scan_node_zones = scan_node_zones(&critical_path, objective, graph.spawn_zone);
}

fn assign_zone_resources(
    zone: &Zone,
    _depths: &HashMap<ZoneId, u32>,
    objective: ZoneId,
    keycard: Option<ZoneId>,
    rng: &mut StdRng,
) -> ZoneResources {
    let mut resources = ZoneResources::default();

    if zone.id == objective {
        resources.med_packs = 1;
        resources.tool_packs = 1;
        resources.ammo_packs = 1;
        return resources;
    }

    if Some(zone.id) == keycard {
        resources.med_packs = 1;
        resources.tool_packs = 0;
        resources.ammo_packs = rng.random_range(0..=1);
        return resources;
    }

    if zone.is_leaf(objective) && zone.id != objective {
        resources.ammo_packs = rng.random_range(1..=2).min(3);
    }

    if zone.hazard_level >= 3 {
        resources.ammo_packs = resources.ammo_packs.saturating_add(1);
    }

    if zone.zone_type == ZoneType::Storage {
        resources.ammo_packs = resources.ammo_packs.saturating_add(2);
        resources.tool_packs = 1;
    }

    resources
}

fn build_terminal_network(graph: &LevelGraph, _depths: &HashMap<ZoneId, u32>) -> TerminalNetwork {
    let mut items = Vec::new();

    if let Some(kc) = graph.keycard_zone {
        items.push(IndexedItem {
            id: format!("KC_{}_{}", color_name(graph.keycard_color), kc.0),
            kind: IndexedItemKind::Keycard,
            zone: kc,
            label: format!("KEYCARD {} IS IN ZONE {}", color_name(graph.keycard_color), kc.0),
        });
    }

    if let Some(obj) = graph.objective_zone {
        items.push(IndexedItem {
            id: format!("OBJ_{}", obj.0),
            kind: IndexedItemKind::Objective,
            zone: obj,
            label: format!("OBJECTIVE IS IN ZONE {}", obj.0),
        });
    }

    for zone in graph.zones.values() {
        let base = zone.id.0;
        if zone.resources.ammo_packs > 0 {
            items.push(IndexedItem {
                id: format!("AMMO_{}", base),
                kind: IndexedItemKind::Ammo,
                zone: zone.id,
                label: format!("AMMO CACHE {} IS IN ZONE {}", zone.resources.ammo_packs, base),
            });
        }
        if zone.resources.med_packs > 0 {
            items.push(IndexedItem {
                id: format!("MED_{}", base),
                kind: IndexedItemKind::Medical,
                zone: zone.id,
                label: format!("MEDICAL SUPPLIES ARE IN ZONE {}", base),
            });
        }
        if zone.resources.tool_packs > 0 {
            items.push(IndexedItem {
                id: format!("TOOL_{}", base),
                kind: IndexedItemKind::Tool,
                zone: zone.id,
                label: format!("TOOL CACHE IS IN ZONE {}", base),
            });
        }
    }

    TerminalNetwork { items }
}

/// Alarm hordes are routed to spawn 2-3 rooms away from the objective so that
/// players have time to set up mines/sentries at the defensive choke points.
fn horde_spawn_zones(critical_path: &[ZoneId], objective: ZoneId) -> Vec<ZoneId> {
    let mut spawns = Vec::new();
    if critical_path.is_empty() {
        return spawns;
    }
    let obj_index = critical_path
        .iter()
        .rposition(|z| *z == objective)
        .unwrap_or(critical_path.len().saturating_sub(1));

    for back in [3usize, 2] {
        let idx = obj_index.checked_sub(back);
        if let Some(idx) = idx {
            if let Some(zone) = critical_path.get(idx) {
                spawns.push(*zone);
            }
        }
    }

    spawns.dedup();
    spawns
}

/// Scan nodes anchor the defensive hold inside the objective room and just
/// upstream so players cannot cheese the alarm behind a single wall.
fn scan_node_zones(
    critical_path: &[ZoneId],
    objective: ZoneId,
    spawn: ZoneId,
) -> Vec<ZoneId> {
    let mut zones = vec![];
    let obj_index = critical_path
        .iter()
        .rposition(|z| *z == objective);
    if let Some(idx) = obj_index {
        zones.push(objective);
        if let Some(upstream) = idx.checked_sub(1).and_then(|i| critical_path.get(i)) {
            if *upstream != spawn {
                zones.push(*upstream);
            }
        }
    }
    zones.dedup();
    zones
}

fn choose_zone_type(rng: &mut StdRng, depth: u32, max_depth: u32) -> ZoneType {
    let roll: f32 = rng.random();

    if depth > max_depth / 2 && roll < 0.15 {
        return ZoneType::Objective;
    }

    match roll {
        r if r < 0.15 => ZoneType::Hub,
        r if r < 0.35 => ZoneType::Utility,
        r if r < 0.50 => ZoneType::Industrial,
        _ => ZoneType::Storage,
    }
}

fn calculate_zone_position(
    graph: &LevelGraph,
    parent_id: ZoneId,
    rng: &mut StdRng,
    min_spacing: f32,
) -> Vec3 {
    let Some(parent) = graph.get_zone(parent_id) else {
        warn!(
            "Missing parent zone {:?} while calculating zone position",
            parent_id
        );
        return Vec3::ZERO;
    };
    let parent_pos = parent.position;

    for _ in 0..10 {
        let angle = rng.random_range(0.0..std::f32::consts::TAU);
        let distance = rng.random_range(min_spacing..(min_spacing * 1.5));

        let offset = Vec3::new(angle.cos() * distance, 0.0, angle.sin() * distance);

        let new_pos = parent_pos + offset;

        let too_close = graph
            .zones
            .values()
            .any(|zone| zone.position.distance(new_pos) < min_spacing * 0.8);

        if !too_close {
            return new_pos;
        }
    }

    let angle = rng.random_range(0.0..std::f32::consts::TAU);
    parent_pos + Vec3::new(angle.cos() * min_spacing, 0.0, angle.sin() * min_spacing)
}

pub fn build_level_physics(mut commands: Commands, level_graph: &LevelGraph) {
    info!(
        "Building physics representation for level with {} zones",
        level_graph.zones.len()
    );

    let wall_height = 10.0;
    let floor_thickness = 1.0;

    let mut min_x = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;
    let mut min_z = f32::INFINITY;
    let mut max_z = f32::NEG_INFINITY;

    // Build physics for all zones
    for zone in level_graph.zones.values() {
        min_x = min_x.min(zone.position.x - zone.size.x * 0.5);
        max_x = max_x.max(zone.position.x + zone.size.x * 0.5);
        min_z = min_z.min(zone.position.z - zone.size.z * 0.5);
        max_z = max_z.max(zone.position.z + zone.size.z * 0.5);

        // Floor collider
        let floor_position = zone.position + Vec3::new(0.0, -floor_thickness / 2.0, 0.0);
        commands.spawn((
            RigidBody::Static,
            Collider::cuboid(zone.size.x, floor_thickness, zone.size.z),
            Position::new(floor_position),
            Rotation::from(zone.rotation),
            Transform::from_translation(floor_position).with_rotation(zone.rotation),
            Name::new(format!("Physics_Floor_Zone_{}", zone.id.0)),
        ));

        // Ceiling collider (prevents players from walking above walls)
        let ceiling_position =
            zone.position + Vec3::new(0.0, wall_height + floor_thickness / 2.0, 0.0);
        commands.spawn((
            RigidBody::Static,
            Collider::cuboid(zone.size.x, floor_thickness, zone.size.z),
            Position::new(ceiling_position),
            Rotation::from(zone.rotation),
            Transform::from_translation(ceiling_position).with_rotation(zone.rotation),
            Name::new(format!("Physics_Ceiling_Zone_{}", zone.id.0)),
        ));

        // Walls colliders
        let wall_segments = collect_zone_wall_segments(zone, level_graph);

        spawn_wall_segments_for_side(
            &mut commands,
            zone,
            WallSide::East,
            &wall_segments[WALL_SIDE_EAST],
        );
        spawn_wall_segments_for_side(
            &mut commands,
            zone,
            WallSide::West,
            &wall_segments[WALL_SIDE_WEST],
        );
        spawn_wall_segments_for_side(
            &mut commands,
            zone,
            WallSide::North,
            &wall_segments[WALL_SIDE_NORTH],
        );
        spawn_wall_segments_for_side(
            &mut commands,
            zone,
            WallSide::South,
            &wall_segments[WALL_SIDE_SOUTH],
        );
    }

    if min_x.is_finite() && max_x.is_finite() && min_z.is_finite() && max_z.is_finite() {
        let safety_margin = 20.0;
        let safety_width = (max_x - min_x) + safety_margin;
        let safety_depth = (max_z - min_z) + safety_margin;
        let safety_center = Vec3::new((min_x + max_x) * 0.5, -4.0, (min_z + max_z) * 0.5);

        commands.spawn((
            RigidBody::Static,
            Collider::cuboid(safety_width, 6.0, safety_depth),
            Position::new(safety_center),
            Rotation::default(),
            Transform::from_translation(safety_center),
            Name::new("Physics_SafetyFloor"),
        ));
    }

    info!("Level physics built successfully");
}

#[cfg(test)]
mod tests {
    use super::{
        DoorType, IndexedItemKind, LevelConfig, LevelGraph, WallSide, ZoneId, build_level_physics,
        build_wall_segments, collect_zone_wall_segments, generate_level, wall_half_span,
    };
    use bevy::prelude::Vec3;
    use std::collections::HashSet;

    #[test]
    fn generate_level_is_deterministic_for_same_seed() {
        let config = LevelConfig {
            seed: 1337,
            target_zone_count: 10,
            min_zone_spacing: 30.0,
            max_depth: 6,
        };

        let level_a = generate_level(config.clone());
        let level_b = generate_level(config);

        assert_eq!(level_a.zones.len(), level_b.zones.len());
        assert_eq!(level_a.connections.len(), level_b.connections.len());

        for zone_id in level_a.zones.keys() {
            let pos_a = level_a
                .zones
                .get(zone_id)
                .expect("zone should exist in first graph")
                .position;
            let pos_b = level_b
                .zones
                .get(zone_id)
                .expect("zone should exist in second graph")
                .position;
            assert_eq!(pos_a, pos_b);
        }
    }

    #[test]
    fn generated_level_contains_spawn_zone_and_connections() {
        let level = generate_level(LevelConfig {
            seed: 7,
            target_zone_count: 12,
            min_zone_spacing: 35.0,
            max_depth: 8,
        });

        assert!(
            level.zones.len() >= 2,
            "Level should contain multiple zones"
        );
        assert!(
            !level.connections.is_empty(),
            "Level should contain at least one connection"
        );
        assert!(
            level.zones.contains_key(&ZoneId(0)),
            "Spawn zone should exist"
        );

        let spawn_pos = level
            .zones
            .get(&ZoneId(0))
            .expect("spawn zone should exist")
            .position;
        assert_eq!(
            spawn_pos,
            Vec3::ZERO,
            "Spawn zone position should be origin"
        );
    }

    #[test]
    fn wall_segments_split_around_single_opening() {
        let segments = build_wall_segments(10.0, &[0.0], 6.0);

        assert_eq!(segments.len(), 2, "Expected two wall segments around one opening");
        assert!(
            (segments[0].1 - 7.0).abs() < 0.001,
            "First segment length should be 7.0, got {:?}",
            segments[0]
        );
        assert!(
            (segments[1].1 - 7.0).abs() < 0.001,
            "Second segment length should be 7.0, got {:?}",
            segments[1]
        );
    }

    #[test]
    fn procedural_connections_create_openings_on_both_sides() {
        let level = generate_level(LevelConfig {
            seed: 99,
            target_zone_count: 12,
            min_zone_spacing: 30.0,
            max_depth: 8,
        });

        assert!(
            !level.connections.is_empty(),
            "Generated level should have at least one connection"
        );

        for connection in &level.connections {
            let from_zone = level
                .get_zone(connection.from_zone)
                .expect("Connection source zone should exist");
            let to_zone = level
                .get_zone(connection.to_zone)
                .expect("Connection target zone should exist");

            let from_direction =
                from_zone.rotation.inverse() * (to_zone.position - from_zone.position);
            let to_direction = to_zone.rotation.inverse() * (from_zone.position - to_zone.position);

            let from_side = WallSide::from_local_direction(from_direction);
            let to_side = WallSide::from_local_direction(to_direction);

            let from_segments = collect_zone_wall_segments(from_zone, &level);
            let to_segments = collect_zone_wall_segments(to_zone, &level);

            let from_full_length = wall_half_span(from_zone, from_side) * 2.0;
            let to_full_length = wall_half_span(to_zone, to_side) * 2.0;

            let from_segment_total: f32 = from_segments[from_side.as_index()]
                .iter()
                .map(|(_, length)| *length)
                .sum();
            let to_segment_total: f32 = to_segments[to_side.as_index()]
                .iter()
                .map(|(_, length)| *length)
                .sum();

            assert!(
                from_segment_total < from_full_length - 0.01,
                "Source zone {:?} side {:?} should have doorway opening (full {:.2}, segmented {:.2})",
                from_zone.id,
                from_side,
                from_full_length,
                from_segment_total
            );

            assert!(
                to_segment_total < to_full_length - 0.01,
                "Target zone {:?} side {:?} should have doorway opening (full {:.2}, segmented {:.2})",
                to_zone.id,
                to_side,
                to_full_length,
                to_segment_total
            );
        }
    }

    fn branched_level() -> LevelGraph {
        for seed in 1..=40 {
            let graph = generate_level(LevelConfig {
                seed,
                target_zone_count: 16,
                min_zone_spacing: 28.0,
                max_depth: 6,
            });
            if graph.keycard_zone.is_some() {
                return graph;
            }
        }
        panic!("No seed in 1..=40 produced a keycard-bearing level; widen the search");
    }

    #[test]
    fn expedition_design_places_objective_in_a_deep_leaf() {
        let level = generate_level(LevelConfig {
            seed: 42,
            target_zone_count: 18,
            min_zone_spacing: 28.0,
            max_depth: 8,
        });

        let objective = level
            .objective_zone
            .expect("expedition design should designate an objective");
        let zone = level
            .get_zone(objective)
            .unwrap_or_else(|| panic!("objective zone {:?} should exist", objective));
        assert!(zone.is_objective);
        assert!(
            zone.is_leaf(level.spawn_zone),
            "objective {:?} should be a leaf node",
            objective
        );
    }

    #[test]
    fn expedition_design_keycard_forces_backtrack() {
        let level = branched_level();

        let keycard_zone = level
            .keycard_zone
            .expect("a branched level should place a keycard");

        let critical_set: HashSet<ZoneId> = level
            .critical_path(objective_or_fail(&level))
            .iter()
            .copied()
            .collect();
        let keycard_zone_handle = level
            .get_zone(keycard_zone)
            .expect("keycard zone should exist");
        assert!(
            keycard_zone_handle.is_keycard,
            "keycard zone {:?} should be flagged",
            keycard_zone
        );
        assert!(
            keycard_zone_handle.is_leaf(level.spawn_zone),
            "keycard zone {:?} should sit in its own branch leaf",
            keycard_zone
        );
        assert!(
            !critical_set.contains(&keycard_zone),
            "keycard zone {:?} must live off the critical path to force backtracking",
            keycard_zone
        );

        let keyed = level
            .connections
            .iter()
            .filter(|c| c.door_type == DoorType::Keycard)
            .collect::<Vec<_>>();
        assert!(
            !keyed.is_empty(),
            "at least one door should require the keycard"
        );
        assert!(
            keyed
                .iter()
                .all(|c| c.required_keycard == Some(keycard_zone)),
            "every keyed door should reference the placed keycard zone"
        );
    }

    #[test]
    fn expedition_design_gates_objective_approach_with_defensive_doors() {
        let level = generate_level(LevelConfig {
            seed: 7,
            target_zone_count: 18,
            min_zone_spacing: 28.0,
            max_depth: 8,
        });
        let objective = level.objective_zone.expect("objective should exist");

        let defensive: Vec<_> = level
            .connections
            .iter()
            .filter(|c| c.door_type.is_defensive_hold())
            .collect();
        assert!(
            !defensive.is_empty(),
            "the critical path should escalate into defensive hold doors"
        );

        let adjacent_to_objective = level.connections.iter().any(|c| {
            c.door_type.is_defensive_hold()
                && (c.from_zone == objective || c.to_zone == objective)
        });
        assert!(
            adjacent_to_objective,
            "objective room should be gated by a defensive-hold door"
        );
    }

    #[test]
    fn expedition_design_terminal_network_indexes_keycard_and_objective() {
        let level = branched_level();

        let has_keycard_entry = level
            .terminal_network
            .items
            .iter()
            .any(|item| item.kind == IndexedItemKind::Keycard);
        let has_objective_entry = level
            .terminal_network
            .items
            .iter()
            .any(|item| item.kind == IndexedItemKind::Objective);
        assert!(has_keycard_entry, "terminal network should index the keycard");
        assert!(has_objective_entry, "terminal network should index the objective");

        let keycard = level.keycard_zone.expect("keycard zone should exist");
        let kc_entry = level
            .terminal_network
            .items
            .iter()
            .find(|item| item.kind == IndexedItemKind::Keycard)
            .expect("keycard terminal entry should exist");
        assert_eq!(kc_entry.zone, keycard);

        let objective = level.objective_zone.expect("objective should exist");
        let obj_entry = level
            .terminal_network
            .items
            .iter()
            .find(|item| item.kind == IndexedItemKind::Objective)
            .expect("objective terminal entry should exist");
        assert_eq!(obj_entry.zone, objective);
    }

    #[test]
    fn expedition_design_resources_concentrated_in_dead_ends() {
        let level = generate_level(LevelConfig {
            seed: 13,
            target_zone_count: 14,
            min_zone_spacing: 30.0,
            max_depth: 7,
        });

        for zone in level.zones.values() {
            if zone.is_leaf(level.spawn_zone)
                && !zone.is_objective
                && !zone.is_keycard
            {
                assert!(
                    zone.resources.ammo_packs >= 1,
                    "dead-end zone {:?} should reward exploration with ammo",
                    zone.id
                );
            }
        }

        let objective = level
            .objective_zone
            .expect("objective should exist");
        let objective_zone = level.get_zone(objective).unwrap();
        assert!(
            objective_zone.resources.ammo_packs + objective_zone.resources.med_packs
                + objective_zone.resources.tool_packs
                > 0,
            "objective zone should carry a resource cache"
        );
    }

    #[test]
    fn expedition_design_horde_spawns_upstream_of_objective() {
        let level = generate_level(LevelConfig {
            seed: 21,
            target_zone_count: 16,
            min_zone_spacing: 28.0,
            max_depth: 7,
        });
        let objective = level.objective_zone.expect("objective should exist");
        let depths = level.depth_map();
        let obj_depth = depths
            .get(&objective)
            .copied()
            .expect("objective should have a depth");

        assert!(
            !level.horde_spawn_zones.is_empty(),
            "alarm hordes should spawn upstream of the objective"
        );
        for spawn_zone in &level.horde_spawn_zones {
            let depth = depths.get(spawn_zone).copied().unwrap_or(0);
            assert!(
                depth < obj_depth,
                "horde spawn zone {:?} should be upstream of objective (depth {} < {})",
                spawn_zone,
                depth,
                obj_depth
            );
        }
    }

    #[test]
    fn expedition_design_does_not_mutate_layout_positions() {
        let config = LevelConfig {
            seed: 999,
            target_zone_count: 14,
            min_zone_spacing: 30.0,
            max_depth: 7,
        };
        let level_a = generate_level(config.clone());
        let level_b = generate_level(config.clone());

        assert_eq!(level_a.zones.len(), level_b.zones.len());
        for (id, zone) in &level_a.zones {
            let other = level_b
                .get_zone(*id)
                .unwrap_or_else(|| panic!("zone {:?} should exist in second run", id));
            assert_eq!(zone.position, other.position);
            assert_eq!(zone.is_objective, other.is_objective);
            assert_eq!(zone.is_keycard, other.is_keycard);
        }
        let _ = &level_a;
    }

    #[test]
    fn expedition_design_handles_chain_without_branching() {
        let level = generate_level(LevelConfig {
            seed: 1,
            target_zone_count: 5,
            min_zone_spacing: 30.0,
            max_depth: 8,
        });
        assert!(
            level.objective_zone.is_some(),
            "even a chain should get an objective"
        );
        // No hard crash and the graph stays well-formed.
        assert!(!level.zones.is_empty());
    }

    fn objective_or_fail(level: &LevelGraph) -> ZoneId {
        level
            .objective_zone
            .unwrap_or_else(|| panic!("expected an objective zone to exist"))
    }

    // ---------------------------------------------------------------------------
    // Room construction tests
    // ---------------------------------------------------------------------------

    use avian3d::prelude::Position;
    use bevy::prelude::{
        App, Commands, Entity, MinimalPlugins, Name, Resource, Res, Update, With,
    };

    #[derive(Resource, Clone)]
    struct TestLevelGraph(LevelGraph);

    fn build_physics_system(commands: Commands, level_graph: Res<TestLevelGraph>) {
        build_level_physics(commands, &level_graph.0);
    }

    fn make_physics_app(level: LevelGraph) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(TestLevelGraph(level));
        app.add_systems(Update, build_physics_system);
        app.finish();
        app.update();
        app
    }

    fn entities_named_with(app: &mut App, prefix: &str) -> Vec<Entity> {
        let world = app.world_mut();
        let mut query = world.query_filtered::<(Entity, &Name), With<Name>>();
        query
            .iter(world)
            .filter(|(_, name)| name.as_str().starts_with(prefix))
            .map(|(entity, _)| entity)
            .collect()
    }

    fn position_of_named(app: &mut App, name: &str) -> Option<Position> {
        let world = app.world_mut();
        let mut query = world.query_filtered::<(&Position, &Name), With<Name>>();
        query
            .iter(world)
            .find(|(_, n)| n.as_str() == name)
            .map(|(pos, _)| *pos)
    }

    #[test]
    fn build_level_physics_creates_floor_for_every_zone() {
        let level = generate_level(LevelConfig {
            seed: 42,
            target_zone_count: 8,
            min_zone_spacing: 30.0,
            max_depth: 6,
        });

        let mut app = make_physics_app(level.clone());

        let floors = entities_named_with(&mut app, "Physics_Floor_Zone");
        assert_eq!(
            floors.len(),
            level.zones.len(),
            "every zone should have a floor collider"
        );
    }

    #[test]
    fn build_level_physics_creates_ceiling_for_every_zone() {
        let level = generate_level(LevelConfig {
            seed: 42,
            target_zone_count: 8,
            min_zone_spacing: 30.0,
            max_depth: 6,
        });

        let mut app = make_physics_app(level.clone());

        let ceilings = entities_named_with(&mut app, "Physics_Ceiling_Zone");
        assert_eq!(
            ceilings.len(),
            level.zones.len(),
            "every zone should have a ceiling collider"
        );
    }

    #[test]
    fn build_level_physics_creates_walls_and_safety_floor() {
        let level = generate_level(LevelConfig {
            seed: 42,
            target_zone_count: 8,
            min_zone_spacing: 30.0,
            max_depth: 6,
        });

        let mut app = make_physics_app(level);

        let walls = entities_named_with(&mut app, "Physics_Wall_");
        assert!(walls.len() > 0, "should have wall segments");

        let safety_exists = {
            let world = app.world_mut();
            let mut query = world.query_filtered::<(Entity, &Name), With<Name>>();
            query.iter(world).any(|(_, name)| name.as_str() == "Physics_SafetyFloor")
        };
        assert!(safety_exists, "should have safety floor");
    }

    #[test]
    fn floor_and_ceiling_align_vertically_per_zone() {
        let level = generate_level(LevelConfig {
            seed: 42,
            target_zone_count: 8,
            min_zone_spacing: 30.0,
            max_depth: 6,
        });

        let mut app = make_physics_app(level.clone());

        for zone in level.zones.values() {
            let floor =
                position_of_named(&mut app, &format!("Physics_Floor_Zone_{}", zone.id.0));
            let ceiling =
                position_of_named(&mut app, &format!("Physics_Ceiling_Zone_{}", zone.id.0));

            assert!(floor.is_some(), "Zone {} should have floor", zone.id.0);
            assert!(
                ceiling.is_some(),
                "Zone {} should have ceiling",
                zone.id.0
            );

            let expected_floor = zone.position.y - 0.5;
            let expected_ceiling = zone.position.y + 10.5;

            assert!(
                (floor.unwrap().0.y - expected_floor).abs() < 0.001,
                "Zone {} floor Y mismatch: expected {}, got {}",
                zone.id.0,
                expected_floor,
                floor.unwrap().0.y
            );
            assert!(
                (ceiling.unwrap().0.y - expected_ceiling).abs() < 0.001,
                "Zone {} ceiling Y mismatch: expected {}, got {}",
                zone.id.0,
                expected_ceiling,
                ceiling.unwrap().0.y
            );
        }
    }

    #[test]
    fn wall_segments_produce_openings_for_connected_zones() {
        let level = generate_level(LevelConfig {
            seed: 42,
            target_zone_count: 12,
            min_zone_spacing: 28.0,
            max_depth: 8,
        });

        assert!(
            !level.connections.is_empty(),
            "level should have connections"
        );

        for zone in level.zones.values() {
            if zone.connections.is_empty() {
                continue;
            }

            let segments = collect_zone_wall_segments(zone, &level);
            let total: f32 = segments.iter().flatten().map(|(_, len)| *len).sum();
            assert!(
                total > 0.0,
                "Zone {} with connections should have wall segments",
                zone.id.0
            );
        }
    }
}
