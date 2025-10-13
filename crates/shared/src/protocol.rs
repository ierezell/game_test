use crate::input::PlayerAction;
use crate::scene::{FloorMarker, WallMarker};
use avian3d::prelude::{LinearVelocity, Position, Rotation};
use bevy::{
    log::debug,
    prelude::{App, Color, Component, Name, Plugin, default},
};

use lightyear::input::config::InputConfig;
use lightyear::prelude::PeerId;
use lightyear::prelude::input::leafwing;
use lightyear::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PlayerId(pub PeerId);

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PlayerColor(pub Color);

#[derive(Clone)]
pub struct ProtocolPlugin;

impl Plugin for ProtocolPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(leafwing::InputPlugin::<PlayerAction> {
            config: InputConfig::<PlayerAction> {
                rebroadcast_inputs: false,
                lag_compensation: true,
                ..default()
            },
        });

        app.register_component::<PlayerId>().add_prediction();
        app.register_component::<FloorMarker>().add_prediction();
        app.register_component::<WallMarker>().add_prediction();
        app.register_component::<PlayerColor>().add_prediction();
        app.register_component::<Name>().add_prediction();

        app.register_component::<Rotation>().add_prediction();
        app.register_component::<Position>().add_prediction();
        app.register_component::<LinearVelocity>().add_prediction();

        debug!("✅ Protocol plugin initialized with components, messages, inputs, and events");
    }
}
