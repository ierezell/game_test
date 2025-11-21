use bevy::prelude::{
    App, Assets, Commands, Mesh, Plugin, ResMut, Single, StandardMaterial, Update,
};
use bevy::state::commands::CommandsStatesExt;
use shared::create_static_level::setup_static_level;

use crate::ClientGameState;
use lightyear::prelude::MessageReceiver;

use shared::protocol::StartLoadingGameEvent;

pub struct GameClientPlugin;

impl Plugin for GameClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, handle_static_world);
    }
}

fn handle_static_world(
    mut receiver: Single<&mut MessageReceiver<StartLoadingGameEvent>>,
    mut commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    materials: Option<ResMut<Assets<StandardMaterial>>>,
) {
    if receiver.has_messages() {
        setup_static_level(commands.reborrow(), meshes, materials, None);
        receiver.receive().for_each(drop);
        commands.set_state(ClientGameState::Playing);
    }
}
