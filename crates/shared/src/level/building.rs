use avian3d::prelude::{LinearVelocity, Position, Rotation};
use bevy::prelude::{
	Color, Commands, Component, Name, PointLight, Quat, Vec2, Vec3, default, info,
};
use lightyear::prelude::{InterpolationTarget, NetworkTarget, Replicate};
use vleue_navigator::prelude::{ManagedNavMesh, NavMeshSettings, NavMeshUpdateMode, Triangulation};

use crate::components::health::{Health, Respawnable};
use crate::entities::NpcPhysicsBundle;
use crate::level::generation::{LevelGraph, Zone, ZoneType};
use crate::navigation::setup_patrol;
use crate::protocol::CharacterMarker;

#[derive(Component, Debug)]
pub struct ProceduralNavMeshMarker;

#[derive(Component, Debug)]
pub struct ProceduralEnemyMarker;

#[derive(Component, Debug)]
pub struct ProceduralConnectionLightMarker;

pub fn setup_procedural_navmesh(commands: &mut Commands, level_graph: &LevelGraph) {
	let mut min_x = f32::INFINITY;
	let mut max_x = f32::NEG_INFINITY;
	let mut min_z = f32::INFINITY;
	let mut max_z = f32::NEG_INFINITY;

	for zone in level_graph.zones.values() {
		let half_x = zone.size.x * 0.5;
		let half_z = zone.size.z * 0.5;
		min_x = min_x.min(zone.position.x - half_x);
		max_x = max_x.max(zone.position.x + half_x);
		min_z = min_z.min(zone.position.z - half_z);
		max_z = max_z.max(zone.position.z + half_z);
	}

	if !min_x.is_finite() || !max_x.is_finite() || !min_z.is_finite() || !max_z.is_finite() {
		return;
	}

	let margin = 2.0;
	let edges = [
		Vec2::new(min_x + margin, min_z + margin),
		Vec2::new(max_x - margin, min_z + margin),
		Vec2::new(max_x - margin, max_z - margin),
		Vec2::new(min_x + margin, max_z - margin),
	];

	commands.spawn((
		ManagedNavMesh::single(),
		NavMeshSettings {
			fixed: Triangulation::from_outer_edges(&edges),
			simplify: 0.1,
			merge_steps: 2,
			build_timeout: Some(20.0),
			agent_radius: 1.0,
			..default()
		},
		NavMeshUpdateMode::Direct,
		ProceduralNavMeshMarker,
		Name::new("ProceduralNavMesh"),
	));

	info!(
		"🗺️ Procedural navmesh built with bounds x:[{:.1}, {:.1}] z:[{:.1}, {:.1}]",
		min_x,
		max_x,
		min_z,
		max_z
	);
}

pub fn spawn_procedural_connection_lights(commands: &mut Commands, level_graph: &LevelGraph) {
	for (index, connection) in level_graph.connections.iter().enumerate() {
		commands.spawn((
			PointLight {
				color: Color::srgb(0.85, 0.9, 1.0),
				intensity: 20000.0,
				range: 16.0,
				radius: 0.6,
				shadows_enabled: false,
				..default()
			},
			bevy::prelude::Transform::from_translation(
				connection.door_position + Vec3::new(0.0, 2.5, 0.0),
			),
			ProceduralConnectionLightMarker,
			Name::new(format!("ProceduralDoorLight_{}", index)),
		));
	}

	info!(
		"💡 Spawned {} procedural connection lights",
		level_graph.connections.len()
	);
}

fn patrol_points_for_zone(zone: &Zone) -> Vec<Vec3> {
	let half_x = (zone.size.x * 0.30).min(12.0);
	let half_z = (zone.size.z * 0.30).min(12.0);
	let offsets = [
		Vec3::new(-half_x, 1.0, -half_z),
		Vec3::new(half_x, 1.0, -half_z),
		Vec3::new(half_x, 1.0, half_z),
		Vec3::new(-half_x, 1.0, half_z),
	];

	offsets
		.iter()
		.map(|offset| zone.position + zone.rotation * *offset)
		.collect()
}

fn enemy_speed_for_zone(zone_type: ZoneType) -> f32 {
	match zone_type {
		ZoneType::Corridor => 3.6,
		ZoneType::Objective => 3.3,
		ZoneType::Industrial => 3.0,
		ZoneType::Hub => 2.8,
		ZoneType::Utility => 2.9,
		ZoneType::Storage => 2.7,
	}
}

pub fn spawn_procedural_enemies(commands: &mut Commands, level_graph: &LevelGraph) {
	let mut candidate_zones: Vec<&Zone> = level_graph
		.zones
		.values()
		.filter(|zone| zone.zone_type != ZoneType::Corridor)
		.collect();

	candidate_zones.sort_by_key(|zone| zone.id.0);

	let max_npcs = candidate_zones.len().clamp(2, 6);
	let selected = candidate_zones.into_iter().take(max_npcs);

	let mut spawned = 0usize;
	for zone in selected {
		let spawn_position = zone.position + Vec3::new(0.0, 1.0, 0.0);
		let patrol_points = patrol_points_for_zone(zone);

		let enemy_entity = commands
			.spawn((
				Name::new(format!("ProceduralEnemy_{}", zone.id.0)),
				Position::new(spawn_position),
				Rotation::from(Quat::IDENTITY),
				LinearVelocity::default(),
				Health::basic(),
				Respawnable::with_position(4.0, spawn_position),
				Replicate::to_clients(NetworkTarget::All),
				InterpolationTarget::to_clients(NetworkTarget::All),
				CharacterMarker,
				ProceduralEnemyMarker,
				NpcPhysicsBundle::default(),
			))
			.id();

		setup_patrol(
			commands,
			enemy_entity,
			patrol_points,
			enemy_speed_for_zone(zone.zone_type),
		);
		spawned += 1;
	}

	info!("🤖 Spawned {} procedural patrolling enemies", spawned);
}

pub fn build_procedural_runtime_content(commands: &mut Commands, level_graph: &LevelGraph) {
	setup_procedural_navmesh(commands, level_graph);
	spawn_procedural_connection_lights(commands, level_graph);
	spawn_procedural_enemies(commands, level_graph);
}

#[cfg(test)]
mod tests {
	use super::{
		ProceduralConnectionLightMarker, ProceduralEnemyMarker, ProceduralNavMeshMarker,
		build_procedural_runtime_content,
	};
	use crate::level::generation::{LevelConfig, LevelGraph, generate_level};
	use crate::navigation::{PatrolRoute, SimpleNavigationAgent};
	use bevy::prelude::{App, Commands, MinimalPlugins, Res, Resource, Update};
	use lightyear::prelude::server::ServerPlugins;
	use std::time::Duration;

	#[derive(Resource, Clone)]
	struct TestLevelGraph(LevelGraph);

	fn build_runtime_content_system(mut commands: Commands, level_graph: Res<TestLevelGraph>) {
		build_procedural_runtime_content(&mut commands, &level_graph.0);
	}

	#[test]
	fn procedural_runtime_content_spawns_navmesh_lights_and_enemies() {
		let mut app = App::new();
		app.add_plugins(MinimalPlugins);
		app.add_plugins(ServerPlugins {
			tick_duration: Duration::from_millis(16),
		});
		app.insert_resource(TestLevelGraph(generate_level(LevelConfig {
			seed: 77,
			target_zone_count: 12,
			min_zone_spacing: 32.0,
			max_depth: 7,
		})));
		app.add_systems(Update, build_runtime_content_system);

		app.update();

		let world = app.world_mut();

		let navmesh_count = world
			.query_filtered::<bevy::prelude::Entity, bevy::prelude::With<ProceduralNavMeshMarker>>()
			.iter(world)
			.count();
		assert_eq!(
			navmesh_count, 1,
			"Expected exactly one procedural navmesh, found {}",
			navmesh_count
		);

		let enemy_count = world
			.query_filtered::<bevy::prelude::Entity, bevy::prelude::With<ProceduralEnemyMarker>>()
			.iter(world)
			.count();
		assert!(
			enemy_count >= 2,
			"Expected at least two procedural enemies, found {}",
			enemy_count
		);

		let connection_light_count = world
			.query_filtered::<
				bevy::prelude::Entity,
				bevy::prelude::With<ProceduralConnectionLightMarker>,
			>()
			.iter(world)
			.count();
		assert!(
			connection_light_count >= 1,
			"Expected at least one procedural connection light, found {}",
			connection_light_count
		);
	}

	#[test]
	fn procedural_enemies_get_patrol_navigation_components() {
		let mut app = App::new();
		app.add_plugins(MinimalPlugins);
		app.add_plugins(ServerPlugins {
			tick_duration: Duration::from_millis(16),
		});
		app.insert_resource(TestLevelGraph(generate_level(LevelConfig {
			seed: 1337,
			target_zone_count: 10,
			min_zone_spacing: 30.0,
			max_depth: 6,
		})));
		app.add_systems(Update, build_runtime_content_system);

		app.update();

		let world = app.world_mut();
		let mut enemy_query = world.query_filtered::<
			(&SimpleNavigationAgent, &PatrolRoute),
			bevy::prelude::With<ProceduralEnemyMarker>,
		>();

		let mut checked = 0usize;
		for (agent, route) in enemy_query.iter(world) {
			checked += 1;
			assert!(
				agent.current_target.is_some(),
				"Procedural enemy should have an initial patrol target"
			);
			assert!(
				route.points.len() >= 4,
				"Procedural patrol route should have at least 4 points"
			);
		}

		assert!(
			checked >= 2,
			"Expected to validate at least two procedural enemies, validated {}",
			checked
		);
	}
}

