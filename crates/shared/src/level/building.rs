use avian3d::prelude::{LinearVelocity, Position, Rotation};
use bevy::prelude::{
	Color, Commands, Component, Name, PointLight, Vec2, Vec3, default, info,
};
use lightyear::prelude::{NetworkTarget, Replicate};
use vleue_navigator::prelude::{ManagedNavMesh, NavMeshSettings, NavMeshUpdateMode, Triangulation};

use crate::entities::NpcPhysicsBundle;
use crate::level::generation::{LevelGraph, Zone, ZoneType};
use crate::protocol::CharacterMarker;
pub use crate::sleeper::{ProceduralSleeperMarker, SleeperArchetype, spawn_sleeper};
use crate::terminal::{TerminalConsole, TerminalState};
use rand::rngs::StdRng;
use rand::SeedableRng;
use rand::RngExt;

#[derive(Component, Debug)]
pub struct ProceduralNavMeshMarker;

#[derive(Component, Debug)]
pub struct ProceduralConnectionLightMarker;

#[derive(Component, Debug)]
pub struct ProceduralTerminalMarker;

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
				shadow_maps_enabled: false,
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

fn weighted_archetype(rng: &mut StdRng) -> SleeperArchetype {
	let roll: f32 = rng.random();
	if roll < 0.55 {
		SleeperArchetype::Drone
	} else if roll < 0.70 {
		SleeperArchetype::Spitter
	} else if roll < 0.90 {
		SleeperArchetype::Scout
	} else {
		SleeperArchetype::Tank
	}
}

fn pseudo_rand(seed: u32, axis: u32) -> f32 {
	let n = seed.wrapping_mul(73856093) ^ (axis * 31);
	let n = n.wrapping_mul(73856093) ^ (n >> 16);
	((n as f32 / u32::MAX as f32) - 0.5) * 2.0
}

pub fn spawn_procedural_sleepers(commands: &mut Commands, level_graph: &LevelGraph) {
	use crate::sleeper::SleeperConfig;
	let config = SleeperConfig::default();

	let mut rng = StdRng::seed_from_u64(level_graph.config.seed);

	let mut candidate_zones: Vec<&Zone> = level_graph
		.zones
		.values()
		.filter(|zone| zone.zone_type != ZoneType::Corridor && !zone.is_objective)
		.collect();

	candidate_zones.sort_by_key(|zone| zone.id.0);

	let mut spawned = 0usize;
	for zone in &candidate_zones {
		let count = rng.random_range(config.min_per_zone..=config.max_per_zone);

		for i in 0..count {
			let offset = Vec3::new(
				pseudo_rand(zone.id.0 as u32 + i * 7, 0) * zone.size.x * 0.3,
				1.0,
				pseudo_rand(zone.id.0 as u32 + i * 7, 1) * zone.size.z * 0.3,
			);
			let position = zone.position + offset;
			let archetype = weighted_archetype(&mut rng);

			let entity = spawn_sleeper(commands, archetype, position, zone.id);
			commands.entity(entity).insert((
				ProceduralSleeperMarker,
				Replicate::to_clients(NetworkTarget::All),
			));
			spawned += 1;
		}
	}

	info!("🤖 Spawned {} procedural sleepers", spawned);
}

pub fn spawn_procedural_terminals(commands: &mut Commands, level_graph: &LevelGraph) {
	let mut spawned = 0usize;

	for zone in level_graph.zones.values() {
		if zone.is_objective {
			let terminal_id = format!("TERM_{}", zone.id.0);
			let description = format!("Objective Terminal - Zone {}", zone.id.0);
			let position = zone.position + Vec3::new(0.0, 1.0, 0.0);

			let entity = commands
				.spawn((
					Name::new(format!("ProceduralTerminal_{}", zone.id.0)),
					Position::new(position),
					Rotation::default(),
					LinearVelocity::default(),
					TerminalConsole::new(&terminal_id, zone.id, &description),
					TerminalState {
						terminal_id: terminal_id.clone(),
						zone_id: zone.id,
						..default()
					},
					ProceduralTerminalMarker,
					CharacterMarker,
					Replicate::to_clients(NetworkTarget::All),
				))
				.id();

			commands.entity(entity).insert(NpcPhysicsBundle::default());
			spawned += 1;
		}
	}

	for zone in level_graph.zones.values() {
		if zone.is_keycard && zone.zone_type == ZoneType::Utility {
			let terminal_id = format!("TERM_KC_{}", zone.id.0);
			let description = format!("Keycard Terminal - Zone {}", zone.id.0);
			let position = zone.position + Vec3::new(0.0, 1.0, 0.0);

			let entity = commands
				.spawn((
					Name::new(format!("ProceduralTerminal_{}", zone.id.0)),
					Position::new(position),
					Rotation::default(),
					LinearVelocity::default(),
					TerminalConsole::new(&terminal_id, zone.id, &description),
					TerminalState {
						terminal_id: terminal_id.clone(),
						zone_id: zone.id,
						..default()
					},
					ProceduralTerminalMarker,
					CharacterMarker,
					Replicate::to_clients(NetworkTarget::All),
				))
				.id();

			commands.entity(entity).insert(NpcPhysicsBundle::default());
			spawned += 1;
		}
	}

	if spawned == 0 {
		for zone in level_graph.zones.values() {
			if zone.zone_type == ZoneType::Objective {
				let terminal_id = format!("TERM_{}", zone.id.0);
				let description = format!("Objective Terminal - Zone {}", zone.id.0);
				let position = zone.position + Vec3::new(0.0, 1.0, 0.0);

				let entity = commands
					.spawn((
						Name::new(format!("ProceduralTerminal_{}", zone.id.0)),
						Position::new(position),
						Rotation::default(),
						LinearVelocity::default(),
						TerminalConsole::new(&terminal_id, zone.id, &description),
						TerminalState {
							terminal_id: terminal_id.clone(),
							zone_id: zone.id,
							..default()
						},
						ProceduralTerminalMarker,
						CharacterMarker,
						Replicate::to_clients(NetworkTarget::All),
					))
					.id();

				commands.entity(entity).insert(NpcPhysicsBundle::default());
				spawned += 1;
			}
		}
	}

	info!("🖥️  Spawned {} procedural terminals", spawned);
}

pub fn build_procedural_runtime_content(commands: &mut Commands, level_graph: &LevelGraph) {
	setup_procedural_navmesh(commands, level_graph);
	spawn_procedural_connection_lights(commands, level_graph);
	spawn_procedural_sleepers(commands, level_graph);
	spawn_procedural_terminals(commands, level_graph);
}

#[cfg(test)]
mod tests {
	use super::{
		ProceduralConnectionLightMarker, ProceduralNavMeshMarker, build_procedural_runtime_content,
	};
	use crate::level::generation::{LevelConfig, LevelGraph, generate_level};
	use crate::sleeper::Sleeper;
	use crate::terminal::TerminalConsole;
	use bevy::prelude::{App, Commands, MinimalPlugins, Res, Resource, Update};
	use bevy::state::app::StatesPlugin;
	use lightyear::prelude::server::ServerPlugins;
	use std::time::Duration;

	#[derive(Resource, Clone)]
	struct TestLevelGraph(LevelGraph);

	fn build_runtime_content_system(mut commands: Commands, level_graph: Res<TestLevelGraph>) {
		build_procedural_runtime_content(&mut commands, &level_graph.0);
	}

	#[test]
	fn procedural_runtime_content_spawns_navmesh_lights_sleepers_and_terminals() {
		let mut app = App::new();
		app.add_plugins(MinimalPlugins);
		app.add_plugins(StatesPlugin);
		app.add_plugins(ServerPlugins {
			tick_duration: Duration::from_millis(16),
		});
		app.insert_resource(TestLevelGraph(generate_level(LevelConfig {
			seed: 77,
			target_zone_count: 12,
			min_zone_spacing: 55.0,
			max_depth: 7,
		})));
		app.add_systems(Update, build_runtime_content_system);
		app.finish();
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

		let sleeper_count = world
			.query_filtered::<bevy::prelude::Entity, bevy::prelude::With<Sleeper>>()
			.iter(world)
			.count();
		assert!(
			sleeper_count >= 2,
			"Expected at least two procedural sleepers, found {}",
			sleeper_count
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

		let terminal_count = world
			.query_filtered::<bevy::prelude::Entity, bevy::prelude::With<TerminalConsole>>()
			.iter(world)
			.count();
		assert!(
			terminal_count >= 1,
			"Expected at least one procedural terminal, found {}",
			terminal_count
		);
	}

	#[test]
	fn procedural_sleepers_have_health_and_navigation_components() {
		let mut app = App::new();
		app.add_plugins(MinimalPlugins);
		app.add_plugins(StatesPlugin);
		app.add_plugins(ServerPlugins {
			tick_duration: Duration::from_millis(16),
		});
		app.insert_resource(TestLevelGraph(generate_level(LevelConfig {
			seed: 1337,
			target_zone_count: 10,
			min_zone_spacing: 55.0,
			max_depth: 6,
		})));
		app.add_systems(Update, build_runtime_content_system);
		app.finish();
		app.update();

		let world = app.world_mut();
		let mut sleeper_query = world.query_filtered::<(
			&crate::navigation::SimpleNavigationAgent,
			&crate::components::health::Health,
		), bevy::prelude::With<Sleeper>>();

		let mut checked = 0usize;
		for (agent, health) in sleeper_query.iter(world) {
			checked += 1;
			assert!(
				agent.current_target.is_none(),
				"Sleepers should start dormant with no navigation target"
			);
			assert!(
				health.current > 0.0,
				"Sleepers should have positive health"
			);
		}

		assert!(
			checked >= 2,
			"Expected to validate at least two sleepers, validated {}",
			checked
		);
	}
}
