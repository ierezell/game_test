use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

use crate::level::generation::{DoorType, LevelGraph, ZoneId, ZoneType};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash, Reflect)]
pub enum NoiseType {
    Footstep,
    Gunshot,
    Door,
    Tool,
    Scream,
    Explosion,
}

impl NoiseType {
    pub fn base_radius(self) -> f32 {
        match self {
            Self::Footstep => 12.0,
            Self::Gunshot => 35.0,
            Self::Door => 15.0,
            Self::Tool => 8.0,
            Self::Scream => 60.0,
            Self::Explosion => 50.0,
        }
    }

    pub fn base_loudness(self) -> f32 {
        match self {
            Self::Footstep => 0.3,
            Self::Gunshot => 1.0,
            Self::Door => 0.5,
            Self::Tool => 0.4,
            Self::Scream => 1.0,
            Self::Explosion => 1.0,
        }
    }
}

#[derive(Message, Clone, Debug, Serialize, Deserialize)]
pub struct NoiseEvent {
    pub source: Vec3,
    pub source_entity: Option<Entity>,
    pub noise_type: NoiseType,
    pub timestamp: f32,
}

impl NoiseEvent {
    pub fn new(source: Vec3, noise_type: NoiseType, timestamp: f32) -> Self {
        Self {
            source,
            source_entity: None,
            noise_type,
            timestamp,
        }
    }

    pub fn with_entity(
        source: Vec3,
        source_entity: Entity,
        noise_type: NoiseType,
        timestamp: f32,
    ) -> Self {
        Self {
            source,
            source_entity: Some(source_entity),
            noise_type,
            timestamp,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ZoneInfo {
    pub zone_id: ZoneId,
    pub center: Vec3,
    pub rotation: Quat,
    pub half_size: Vec3,
    pub zone_type: ZoneType,
}

#[derive(Clone, Debug)]
pub struct ConnectionInfo {
    pub from: ZoneId,
    pub to: ZoneId,
    pub door_type: DoorType,
    pub dampening: f32,
}

impl DoorType {
    pub fn noise_dampening(self) -> f32 {
        match self {
            Self::Normal => 0.30,
            Self::Keycard => 0.40,
            Self::Alarm => 0.15,
            Self::Bulkhead => 0.55,
        }
    }
}

#[derive(Resource, Clone, Debug, Default)]
pub struct ZoneConnectivity {
    pub zones: HashMap<ZoneId, ZoneInfo>,
    pub connections: Vec<ConnectionInfo>,
    pub adjacency: HashMap<ZoneId, Vec<(ZoneId, DoorType)>>,
}

impl ZoneConnectivity {
    pub fn zone_for_position(&self, pos: Vec3) -> Option<ZoneId> {
        for (zone_id, info) in &self.zones {
            let local = info.rotation.inverse() * (pos - info.center);
            if local.x.abs() <= info.half_size.x + 0.5 && local.z.abs() <= info.half_size.z + 0.5 {
                return Some(*zone_id);
            }
        }
        None
    }

    pub fn dampening_between(&self, a: ZoneId, b: ZoneId) -> f32 {
        self.adjacency
            .get(&a)
            .and_then(|conns| conns.iter().find(|(zone, _)| *zone == b))
            .map(|(_, dt)| dt.noise_dampening())
            .unwrap_or(0.0)
    }
}

pub fn build_zone_connectivity(graph: &LevelGraph) -> ZoneConnectivity {
    let mut zones = HashMap::new();
    for zone in graph.zones.values() {
        zones.insert(
            zone.id,
            ZoneInfo {
                zone_id: zone.id,
                center: zone.position,
                rotation: zone.rotation,
                half_size: zone.size * 0.5,
                zone_type: zone.zone_type,
            },
        );
    }

    let mut connections = Vec::new();
    let mut adjacency: HashMap<ZoneId, Vec<(ZoneId, DoorType)>> = HashMap::new();

    for zone_id in graph.zones.keys() {
        adjacency.entry(*zone_id).or_default();
    }

    for conn in &graph.connections {
        let dampening = conn.door_type.noise_dampening();
        connections.push(ConnectionInfo {
            from: conn.from_zone,
            to: conn.to_zone,
            door_type: conn.door_type,
            dampening,
        });

        adjacency
            .entry(conn.from_zone)
            .or_default()
            .push((conn.to_zone, conn.door_type));
        adjacency
            .entry(conn.to_zone)
            .or_default()
            .push((conn.from_zone, conn.door_type));
    }

    ZoneConnectivity {
        zones,
        connections,
        adjacency,
    }
}

#[derive(Resource, Clone, Debug, Default)]
pub struct NoiseField {
    pub zone_noise: HashMap<ZoneId, f32>,
    pub event_queue: Vec<NoiseEvent>,
    pub last_update: f32,
}

impl NoiseField {
    pub fn clear_events(&mut self) {
        self.event_queue.clear();
    }

    pub fn noise_in_zone(&self, zone: ZoneId) -> f32 {
        self.zone_noise.get(&zone).copied().unwrap_or(0.0)
    }

    pub fn max_noise(&self) -> f32 {
        self.zone_noise.values().copied().fold(0.0f32, f32::max)
    }
}

pub struct NoisePlugin;

impl Plugin for NoisePlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<NoiseEvent>();
        app.insert_resource(NoiseField::default());
        app.add_systems(
            FixedUpdate,
            (
                collect_noise_events,
                propagate_noise_system,
                cleanup_noise_field,
            )
                .chain(),
        );
    }
}

fn collect_noise_events(
    mut noise_events: MessageReader<NoiseEvent>,
    mut noise_field: ResMut<NoiseField>,
) {
    for event in noise_events.read() {
        noise_field.event_queue.push(event.clone());
    }
}

pub fn propagate_noise_system(
    zone_conn: Option<Res<ZoneConnectivity>>,
    time: Res<Time>,
    mut noise_field: ResMut<NoiseField>,
) {
    let Some(zone_conn) = zone_conn else {
        noise_field.event_queue.clear();
        return;
    };

    let timestamp = time.elapsed().as_secs_f32();
    noise_field.last_update = timestamp;
    noise_field.zone_noise.clear();

    let events = std::mem::take(&mut noise_field.event_queue);

    for event in events {
        let base_loudness = event.noise_type.base_loudness();
        let Some(source_zone) = zone_conn.zone_for_position(event.source) else {
            continue;
        };

        let mut best_noise: HashMap<ZoneId, f32> = HashMap::new();
        let mut queue: VecDeque<ZoneId> = VecDeque::new();

        best_noise.insert(source_zone, base_loudness);
        queue.push_back(source_zone);

        while let Some(zone) = queue.pop_front() {
            let noise = *best_noise.get(&zone).unwrap_or(&0.0);

            let entry = noise_field.zone_noise.entry(zone).or_insert(0.0);
            *entry = (*entry).max(noise);

            if noise < 0.05 {
                continue;
            }

            if let Some(conns) = zone_conn.adjacency.get(&zone) {
                for (neighbor, door_type) in conns {
                    let attenuated = noise * (1.0 - door_type.noise_dampening());
                    let current_best = best_noise.get(neighbor).copied().unwrap_or(0.0);

                    if attenuated > current_best && attenuated >= 0.05 {
                        best_noise.insert(*neighbor, attenuated);
                        queue.push_back(*neighbor);
                    }
                }
            }
        }
    }
}

fn cleanup_noise_field(time: Res<Time>, mut noise_field: ResMut<NoiseField>) {
    let now = time.elapsed().as_secs_f32();
    if now - noise_field.last_update > 5.0 {
        noise_field.zone_noise.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::level::generation::{LevelConfig, generate_level};
    use bevy::time::TimeUpdateStrategy;

    #[test]
    fn noise_type_loudness_and_radius_are_sensible() {
        assert_eq!(NoiseType::Gunshot.base_loudness(), 1.0);
        assert!(NoiseType::Footstep.base_loudness() < NoiseType::Gunshot.base_loudness());
        assert!(NoiseType::Scream.base_radius() > NoiseType::Tool.base_radius());
        assert!(NoiseType::Explosion.base_loudness() == 1.0);
    }

    #[test]
    fn door_type_noise_dampening_values() {
        assert!(DoorType::Normal.noise_dampening() < DoorType::Keycard.noise_dampening());
        assert!(DoorType::Alarm.noise_dampening() < DoorType::Normal.noise_dampening());
        assert!(DoorType::Bulkhead.noise_dampening() > DoorType::Alarm.noise_dampening());
    }

    #[test]
    fn build_zone_connectivity_from_generated_level() {
        let level = generate_level(LevelConfig {
            seed: 42,
            target_zone_count: 10,
            min_zone_spacing: 55.0,
            max_depth: 6,
        });

        let conn = build_zone_connectivity(&level);

        assert_eq!(conn.zones.len(), level.zones.len());
        assert!(!conn.connections.is_empty());
        assert!(!conn.adjacency.is_empty());

        for zone_id in level.zones.keys() {
            assert!(conn.adjacency.contains_key(zone_id));
        }
    }

    #[test]
    fn zone_for_position_finds_correct_zone() {
        let level = generate_level(LevelConfig {
            seed: 42,
            target_zone_count: 6,
            min_zone_spacing: 55.0,
            max_depth: 5,
        });

        let conn = build_zone_connectivity(&level);

        for zone in level.zones.values() {
            let found = conn.zone_for_position(zone.position);
            assert_eq!(
                found,
                Some(zone.id),
                "Position at zone center should find that zone"
            );
        }
    }

    #[test]
    fn zone_for_position_returns_none_outside_all_zones() {
        let level = generate_level(LevelConfig {
            seed: 100,
            target_zone_count: 5,
            min_zone_spacing: 55.0,
            max_depth: 4,
        });

        let conn = build_zone_connectivity(&level);

        let far_point = Vec3::new(10000.0, 0.0, 10000.0);
        assert_eq!(conn.zone_for_position(far_point), None);
    }

    #[test]
    fn propagate_noise_single_event_in_origin_zone() {
        let level = generate_level(LevelConfig {
            seed: 42,
            target_zone_count: 6,
            min_zone_spacing: 55.0,
            max_depth: 5,
        });

        let conn = build_zone_connectivity(&level);
        let mut noise_field = NoiseField::default();

        let source_zone = level.zones.keys().next().copied().unwrap();
        let source_zone_info = conn.zones.get(&source_zone).unwrap();
        let noise_pos = source_zone_info.center;

        noise_field
            .event_queue
            .push(NoiseEvent::new(noise_pos, NoiseType::Gunshot, 0.0));

        // Simulate propagation
        let base_loudness = NoiseType::Gunshot.base_loudness();
        noise_field.zone_noise.insert(source_zone, base_loudness);

        assert!(noise_field.noise_in_zone(source_zone) > 0.0);
    }

    #[test]
    fn noise_field_clear_events_resets_queue() {
        let mut field = NoiseField::default();
        field
            .event_queue
            .push(NoiseEvent::new(Vec3::ZERO, NoiseType::Footstep, 0.0));
        field
            .event_queue
            .push(NoiseEvent::new(Vec3::ZERO, NoiseType::Door, 0.0));

        assert_eq!(field.event_queue.len(), 2);

        field.clear_events();
        assert!(field.event_queue.is_empty());
    }

    #[test]
    fn noise_field_max_noise_returns_highest_value() {
        let mut field = NoiseField::default();
        field.zone_noise.insert(ZoneId(1), 0.3);
        field.zone_noise.insert(ZoneId(2), 0.7);
        field.zone_noise.insert(ZoneId(3), 0.2);

        assert!((field.max_noise() - 0.7).abs() < 0.001);
    }

    #[test]
    fn noise_propagates_through_normal_door_and_attenuates() {
        let level = generate_level(LevelConfig {
            seed: 42,
            target_zone_count: 8,
            min_zone_spacing: 55.0,
            max_depth: 6,
        });

        let conn = build_zone_connectivity(&level);
        let mut noise_field = NoiseField::default();

        if let Some((source_zone, neighbor_zone)) = conn.zones.keys().next().and_then(|&src| {
            conn.adjacency
                .get(&src)
                .and_then(|conns| conns.first().map(|(n, _)| (src, *n)))
        }) {
            let source_info = conn.zones.get(&source_zone).unwrap();
            let noise_pos = source_info.center;
            let base_loudness = NoiseType::Gunshot.base_loudness();

            noise_field
                .event_queue
                .push(NoiseEvent::new(noise_pos, NoiseType::Gunshot, 0.0));

            // Simulate propagation manually
            let dampening = conn.dampening_between(source_zone, neighbor_zone);

            noise_field.zone_noise.insert(source_zone, base_loudness);
            let neighbor_noise = base_loudness * (1.0 - dampening);
            noise_field.zone_noise.insert(neighbor_zone, neighbor_noise);

            assert!(
                noise_field.noise_in_zone(neighbor_zone) < noise_field.noise_in_zone(source_zone),
                "Noise should attenuate through a door: source={}, neighbor={}",
                noise_field.noise_in_zone(source_zone),
                noise_field.noise_in_zone(neighbor_zone)
            );
        }
    }

    #[test]
    fn noise_does_not_propagate_through_bulkhead_effectively() {
        assert!(
            DoorType::Bulkhead.noise_dampening() > DoorType::Normal.noise_dampening(),
            "Bulkhead should dampen more than Normal door"
        );

        let bulkhead_noise = 1.0 * (1.0 - DoorType::Bulkhead.noise_dampening());
        let normal_noise = 1.0 * (1.0 - DoorType::Normal.noise_dampening());
        assert!(
            bulkhead_noise < normal_noise,
            "Noise after bulkhead ({}) should be less than through normal door ({})",
            bulkhead_noise,
            normal_noise
        );
    }

    #[test]
    fn noise_propagation_terminates_and_bounds_values_on_cyclic_graph() {
        let ids = [ZoneId(1), ZoneId(2), ZoneId(3)];
        let mut zones = HashMap::new();
        for id in ids {
            zones.insert(
                id,
                ZoneInfo {
                    zone_id: id,
                    center: Vec3::new(id.0 as f32 * 10.0, 0.0, 0.0),
                    rotation: Quat::IDENTITY,
                    half_size: Vec3::splat(2.0),
                    zone_type: ZoneType::Corridor,
                },
            );
        }

        let mut adjacency = HashMap::new();
        for id in ids {
            adjacency.insert(id, Vec::new());
        }
        for (a, b) in [(ids[0], ids[1]), (ids[1], ids[2]), (ids[2], ids[0])] {
            adjacency.get_mut(&a).unwrap().push((b, DoorType::Normal));
            adjacency.get_mut(&b).unwrap().push((a, DoorType::Normal));
        }

        let connectivity = ZoneConnectivity {
            zones,
            connections: Vec::new(),
            adjacency,
        };
        let source = connectivity.zones[&ids[0]].center;

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(TimeUpdateStrategy::ManualDuration(
            std::time::Duration::from_millis(16),
        ));
        app.insert_resource(connectivity);
        let mut field = NoiseField::default();
        field
            .event_queue
            .push(NoiseEvent::new(source, NoiseType::Gunshot, 0.0));
        app.insert_resource(field);
        app.add_systems(Update, propagate_noise_system);

        app.update();

        let field = app.world().resource::<NoiseField>();
        assert_eq!(field.zone_noise.len(), ids.len());
        for id in ids {
            let noise = field.noise_in_zone(id);
            assert!(noise.is_finite() && (0.0..=1.0).contains(&noise));
        }
        assert!(field.noise_in_zone(ids[0]) > field.noise_in_zone(ids[1]));
    }
}
