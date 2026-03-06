mod game;
mod npc;
mod player;

use bevy::{
	ecs::schedule::IntoScheduleConfigs,
	prelude::{App, FixedUpdate, Plugin, Update},
	state::{condition::in_state, state::OnEnter},
};
use shared::gym::{spawn_gym_patrolling_npc_entities, update_gym_wandering_npc_targets};

use self::game::generate_and_build_level;
use self::npc::{mark_dead_npcs_for_respawn, respawn_dead_npcs};
use self::player::{handle_player_death, spawn_late_joining_players};

use crate::ServerGameState;

pub struct ServerEntitiesPlugin;

impl Plugin for ServerEntitiesPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(
			FixedUpdate,
			(
				spawn_late_joining_players,
				handle_player_death,
				mark_dead_npcs_for_respawn,
				respawn_dead_npcs,
			)
				.run_if(in_state(ServerGameState::Playing)),
		);
		app.add_systems(OnEnter(ServerGameState::Loading), generate_and_build_level);
		app.add_systems(
			OnEnter(ServerGameState::Playing),
			spawn_gym_patrolling_npc_entities,
		);
		app.add_systems(
			Update,
			update_gym_wandering_npc_targets.run_if(in_state(ServerGameState::Playing)),
		);
	}
}

