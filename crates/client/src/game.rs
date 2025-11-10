use bevy::prelude::{
    App, Assets, Commands, Mesh, Plugin, ResMut, Single, StandardMaterial, Update, info,
};
use bevy::state::commands::CommandsStatesExt;
use shared::level::create_static::setup_static_level;

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
        info!("🌍 Client Static world setup complete, Playing...");
        // Now we wait for the server to create dynamic entities like players as
        // all the other plugins await replicated entities to trigger.
    }
}
