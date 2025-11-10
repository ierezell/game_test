pub mod lobby;
pub mod local;

pub use lobby::ClientLobbyPlugin;
pub use local::LocalMenuPlugin;

use bevy::prelude::Resource;

#[derive(Resource)]
pub struct AutoHost(pub bool);

#[derive(Resource)]
pub struct AutoStart(pub bool);

#[derive(Resource)]
pub struct AutoJoin(pub bool);
