// Level generation module - GTFO-style Industrial Complex procedural generation
//
// This module implements a server-authoritative procedural level generation system
// using a shared seed to ensure deterministic generation across client and server.

use bevy::prelude::*;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// CORE COMPONENTS & TYPES
// ============================================================================

/// Unique identifier for each zone/room in the level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ZoneId(pub u32);

/// Types of zones in the industrial complex
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZoneType {
    /// Central hub with multiple connections
    Hub,
    /// Long connecting passage
    Corridor,
    /// Small utility or storage room
    Utility,
    /// Large industrial chamber
    Industrial,
    /// Mission objective location
    Objective,
    /// Dead-end storage area
    Storage,
}

impl ZoneType {
    /// Get the approximate size multiplier for this zone type
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

    /// Get the maximum number of connections for this zone type
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

/// Individual zone/room in the level
#[derive(Debug, Clone, Serialize, Deserialize, Component)]
pub struct Zone {
    pub id: ZoneId,
    pub zone_type: ZoneType,
    pub position: Vec3,
    pub size: Vec3,
    /// IDs of connected zones
    pub connections: Vec<ZoneId>,
    /// Whether this zone has been generated/built
    pub is_built: bool,
}

impl Zone {
    pub fn new(id: ZoneId, zone_type: ZoneType, position: Vec3) -> Self {
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
            size,
            connections: Vec::new(),
            is_built: false,
        }
    }
}

/// Connection between two zones via a bulkhead door
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZoneConnection {
    pub from_zone: ZoneId,
    pub to_zone: ZoneId,
    /// Position of the door in world space
    pub door_position: Vec3,
    /// Direction the door faces
    pub door_rotation: Quat,
}

/// Level generation configuration
#[derive(Debug, Clone, Resource, Serialize, Deserialize)]
pub struct LevelConfig {
    /// Random seed for generation
    pub seed: u64,
    /// Target number of zones to generate
    pub target_zone_count: u32,
    /// Minimum distance between zone centers
    pub min_zone_spacing: f32,
    /// Maximum depth of the level (for random walk)
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

/// Resource containing the complete level graph structure
///
/// This is generated on the server and the seed is replicated to clients
/// Both server and client run the same deterministic generation algorithm
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

    /// Get a zone by ID
    pub fn get_zone(&self, id: ZoneId) -> Option<&Zone> {
        self.zones.get(&id)
    }

    /// Get a mutable zone by ID
    pub fn get_zone_mut(&mut self, id: ZoneId) -> Option<&mut Zone> {
        self.zones.get_mut(&id)
    }

    /// Add a zone to the graph
    pub fn add_zone(&mut self, zone: Zone) {
        self.zones.insert(zone.id, zone);
    }

    /// Add a connection between two zones
    pub fn add_connection(
        &mut self,
        from: ZoneId,
        to: ZoneId,
        door_position: Vec3,
        door_rotation: Quat,
    ) {
        // Add to connections list
        self.connections.push(ZoneConnection {
            from_zone: from,
            to_zone: to,
            door_position,
            door_rotation,
        });

        // Update zone connection lists (bidirectional)
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

// ============================================================================
// LEVEL GENERATION ALGORITHM
// ============================================================================

/// Generate a procedural level using a random walk algorithm
///
/// This creates a branching structure of connected rooms similar to GTFO
pub fn generate_level(config: LevelConfig) -> LevelGraph {
    let mut rng = StdRng::seed_from_u64(config.seed);
    let mut graph = LevelGraph::new(config.clone());

    // Start with the spawn hub at origin
    let spawn_zone = Zone::new(ZoneId(0), ZoneType::Hub, Vec3::ZERO);
    graph.spawn_zone = spawn_zone.id;
    graph.add_zone(spawn_zone);

    // Track zones that can be expanded from
    let mut frontier: Vec<(ZoneId, u32)> = vec![(ZoneId(0), 0)]; // (zone_id, depth)
    let mut next_zone_id = 1u32;

    // Random walk to create branching structure
    while graph.zones.len() < config.target_zone_count as usize && !frontier.is_empty() {
        // Pick a random zone from the frontier
        let frontier_index = rng.random_range(0..frontier.len());
        let (current_zone_id, depth) = frontier[frontier_index];

        // Remove from frontier if we've hit max depth or max connections
        let current_zone = graph.get_zone(current_zone_id).unwrap().clone();
        let max_connections = current_zone.zone_type.max_connections();

        if depth >= config.max_depth || current_zone.connections.len() >= max_connections {
            frontier.remove(frontier_index);
            continue;
        }

        // Decide how many new zones to branch from this one
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

        // Create branches
        for _ in 0..branch_count {
            if graph.zones.len() >= config.target_zone_count as usize {
                break;
            }

            // Choose zone type based on depth and randomness
            let zone_type = choose_zone_type(&mut rng, depth, config.max_depth);

            // Calculate position for new zone
            let new_position =
                calculate_zone_position(&graph, current_zone_id, &mut rng, config.min_zone_spacing);

            // Create new zone
            let new_zone = Zone::new(ZoneId(next_zone_id), zone_type, new_position);
            let new_zone_id = new_zone.id;

            // Calculate door position (midpoint between zones)
            let current_pos = current_zone.position;
            let door_position = (current_pos + new_position) * 0.5;
            let direction = (new_position - current_pos).normalize();
            let door_rotation = Quat::from_rotation_arc(Vec3::Z, direction);

            graph.add_zone(new_zone);
            graph.add_connection(current_zone_id, new_zone_id, door_position, door_rotation);

            // Add to frontier if it can be expanded
            if zone_type.max_connections() > 1 {
                frontier.push((new_zone_id, depth + 1));
            }

            // Track objective zones
            if zone_type == ZoneType::Objective {
                graph.objective_zones.push(new_zone_id);
            }

            next_zone_id += 1;
        }

        // Re-add current zone to frontier if it can still be expanded
        let updated_zone = graph.get_zone(current_zone_id).unwrap();
        if updated_zone.connections.len() < max_connections {
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

/// Choose a zone type based on depth and randomness
fn choose_zone_type(rng: &mut StdRng, depth: u32, max_depth: u32) -> ZoneType {
    let roll: f32 = rng.random();

    // Objective zones appear at deeper levels
    if depth > max_depth / 2 && roll < 0.15 {
        return ZoneType::Objective;
    }

    // Type distribution
    match roll {
        r if r < 0.15 => ZoneType::Hub,
        r if r < 0.35 => ZoneType::Corridor,
        r if r < 0.50 => ZoneType::Utility,
        r if r < 0.70 => ZoneType::Industrial,
        _ => ZoneType::Storage,
    }
}

/// Calculate a new zone position relative to parent zone
fn calculate_zone_position(
    graph: &LevelGraph,
    parent_id: ZoneId,
    rng: &mut StdRng,
    min_spacing: f32,
) -> Vec3 {
    let parent = graph.get_zone(parent_id).unwrap();
    let parent_pos = parent.position;

    // Try multiple times to find a non-overlapping position
    for _ in 0..10 {
        // Random angle
        let angle = rng.random_range(0.0..std::f32::consts::TAU);
        let distance = rng.random_range(min_spacing..(min_spacing * 1.5));

        let offset = Vec3::new(
            angle.cos() * distance,
            0.0, // Keep zones on same height
            angle.sin() * distance,
        );

        let new_pos = parent_pos + offset;

        // Check if position is far enough from existing zones
        let too_close = graph
            .zones
            .values()
            .any(|zone| zone.position.distance(new_pos) < min_spacing * 0.8);

        if !too_close {
            return new_pos;
        }
    }

    // Fallback: just use the offset from parent
    let angle = rng.random_range(0.0..std::f32::consts::TAU);
    parent_pos + Vec3::new(angle.cos() * min_spacing, 0.0, angle.sin() * min_spacing)
}

// ============================================================================
// PLUGIN
// ============================================================================

pub struct LevelGenerationPlugin;

impl Plugin for LevelGenerationPlugin {
    fn build(&self, _app: &mut App) {
        info!("Level Generation plugin initialized");
        // The actual generation happens on-demand when entering the Playing state
        // This plugin just registers the types
    }
}
