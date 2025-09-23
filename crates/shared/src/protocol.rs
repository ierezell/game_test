use crate::input::PlayerAction;
use crate::scene::*;
use avian3d::prelude::{LinearVelocity, Position, Rotation};
use bevy::{
    log::debug,
    prelude::{App, Color, Component, Name, Plugin, Reflect, default},
};

use lightyear::input::prelude::InputConfig;
use lightyear::prelude::PeerId;
use lightyear::prelude::input::leafwing::InputPlugin;
use lightyear::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq, Reflect)]
pub struct PlayerId(pub PeerId);

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PlayerColor(pub Color);

#[derive(Clone)]
pub struct ProtocolPlugin;

impl Plugin for ProtocolPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputPlugin::<PlayerAction> {
            config: InputConfig::<PlayerAction> {
                rebroadcast_inputs: false,
                lag_compensation: true,
                ..default()
            },
        });
        app.register_component::<PlayerId>()
            .add_prediction(PredictionMode::Once)
            // .add_interpolation(InterpolationMode::Once)
            ;

        app.register_component::<FloorMarker>();
        app.register_component::<WallMarker>();
        app.register_component::<PlayerColor>()
            .add_prediction(PredictionMode::Once)
            // .add_interpolation(InterpolationMode::Once)
            ;

        app.register_component::<Name>()
            .add_prediction(PredictionMode::Once)
            // .add_interpolation(InterpolationMode::Once)
            ;

        app.register_component::<Rotation>()
                        // .add_delta_compression()
                        .add_prediction(PredictionMode::Full)
                        // .add_interpolation(InterpolationMode::Full)
                        // .add_linear_interpolation_fn()
                        ;

        app.register_component::<Position>()
            // .add_delta_compression()
            .add_prediction(PredictionMode::Full)
            .add_should_rollback(|old: &Position, new: &Position| {
                (old.0.y - new.0.y).abs() > 2.0
            })
            // .add_interpolation(InterpolationMode::Full)
            // .add_linear_interpolation_fn()
            ;

        app.register_component::<LinearVelocity>()
            .add_prediction(PredictionMode::Full)
            .add_should_rollback(|old: &LinearVelocity, new: &LinearVelocity| {
                (old.0.y - new.0.y).abs() > 0.5
            });
        // .add_interpolation(InterpolationMode::Full);
        // .add_interpolation_fn(|a, b, t| LinearVelocity(a.0.lerp(b.0, t)))

        debug!("✅ Protocol plugin initialized with components, messages, inputs, and events");
    }
}
