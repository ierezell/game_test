use avian3d::prelude::{Collider, Position, RigidBody, Rotation};
use bevy::prelude::*;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

#[derive(Debug, Clone, Serialize, Deserialize, Component)]
pub struct Zone {
    pub id: ZoneId,
    pub zone_type: ZoneType,
    pub position: Vec3,
    pub rotation: Quat,
    pub size: Vec3,
    pub connections: Vec<ZoneId>,
    pub is_built: bool,
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
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZoneConnection {
    pub from_zone: ZoneId,
    pub to_zone: ZoneId,
    pub door_position: Vec3,
    pub door_rotation: Quat,
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
}

impl LevelGraph {
    pub fn new(config: LevelConfig) -> Self {
        Self {
            config,
            zones: HashMap::new(),
            connections: Vec::new(),
            spawn_zone: ZoneId(0),
            objective_zones: Vec::new(),
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
    ) {
        self.connections.push(ZoneConnection {
            from_zone: from,
            to_zone: to,
            door_position,
            door_rotation,
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
            graph.add_connection(current_zone_id, new_zone_id, door_position, door_rotation);

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

    info!(
        "Generated level with {} zones and {} connections",
        graph.zones.len(),
        graph.connections.len()
    );

    graph
}

fn choose_zone_type(rng: &mut StdRng, depth: u32, max_depth: u32) -> ZoneType {
    let roll: f32 = rng.random();

    if depth > max_depth / 2 && roll < 0.15 {
        return ZoneType::Objective;
    }

    match roll {
        r if r < 0.15 => ZoneType::Hub,
        r if r < 0.35 => ZoneType::Corridor,
        r if r < 0.50 => ZoneType::Utility,
        r if r < 0.70 => ZoneType::Industrial,
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
        let floor_thickness = 1.0;
        let floor_position = zone.position + Vec3::new(0.0, -floor_thickness / 2.0, 0.0);
        commands.spawn((
            RigidBody::Static,
            Collider::cuboid(zone.size.x, floor_thickness, zone.size.z),
            Position::new(floor_position),
            Rotation::from(zone.rotation),
            Transform::from_translation(floor_position).with_rotation(zone.rotation),
            Name::new(format!("Physics_Floor_Zone_{}", zone.id.0)),
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
        LevelConfig, WallSide, ZoneId, build_wall_segments, collect_zone_wall_segments,
        generate_level, wall_half_span,
    };
    use bevy::prelude::Vec3;

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
}
