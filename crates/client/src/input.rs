use avian3d::prelude::*;
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

use lightyear::prelude::Controlled;

use lightyear::prelude::*;

use shared::input::{PlayerAction, PlayerVelocity, apply_player_action};
use shared::scene::{PlayerColor, PlayerMarker, PlayerPhysicsBundle};

pub struct ClientInputPlugin;

impl Plugin for ClientInputPlugin {
    fn build(&self, app: &mut App) {
        // Movement system runs in FixedUpdate
        app.add_systems(FixedUpdate, client_player_movement);
        app.add_observer(handle_player_spawn);
    }
}

fn handle_player_spawn(
    trigger: Trigger<OnAdd, PlayerMarker>,
    mut player_query: Query<(
        &mut PlayerColor,
        Has<Predicted>,
        Has<Interpolated>,
        Has<Controlled>,
    )>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let entity = trigger.target();

    if let Ok((mut color, is_predicted, is_interpolated, is_controlled)) =
        player_query.get_mut(entity)
    {
        info!(
            "🎮 PLAYER SPAWNED: {:?} - Predicted: {}, Interpolated: {}, Controlled: {}",
            entity, is_predicted, is_interpolated, is_controlled
        );

        if is_predicted {
            let hsva = bevy::color::Hsva {
                saturation: 0.4,
                ..bevy::color::Hsva::from(color.0)
            };
            color.0 = Color::from(hsva);

            // Add InputMap for controlled predicted entities (local player input)
            if is_controlled {
                info!(
                    "🎯 Adding InputMap to controlled and predicted entity: {:?}",
                    entity
                );

                // Following lightyear examples pattern - add InputMap component
                let input_map = InputMap::<PlayerAction>::new([
                    (PlayerAction::Jump, KeyCode::Space),
                    (PlayerAction::Shoot, KeyCode::Enter),
                ])
                .with_dual_axis(PlayerAction::Move, VirtualDPad::wasd())
                .with_dual_axis(PlayerAction::Move, VirtualDPad::arrow_keys());

                commands.entity(entity).insert(input_map);
            } else {
                info!("🌐 Remote player predicted for us: {:?}", entity);
            }
        } else if is_interpolated {
            let hsva = bevy::color::Hsva {
                value: 0.8,
                ..bevy::color::Hsva::from(color.0)
            };
            color.0 = Color::from(hsva);
        }

        commands
            .entity(entity)
            .insert((PlayerPhysicsBundle::default(), PlayerVelocity::default()));

        commands.entity(entity).insert((
            Mesh3d(meshes.add(Capsule3d::new(
                shared::input::PLAYER_CAPSULE_RADIUS,
                shared::input::PLAYER_CAPSULE_HEIGHT,
            ))),
            MeshMaterial3d(materials.add(color.0)),
        ));

        info!("✅ Player entity setup complete for {:?}", entity);
    } else {
        error!("❌ Failed to get PlayerColor for entity {:?}", entity);
    }
}

/// The client input only gets applied to predicted entities that we own
/// This works because we only predict the user's controlled entity.
fn client_player_movement(
    time: Res<Time>,
    mut player_query: Query<
        (
            Entity,
            &mut Position,
            &mut PlayerVelocity,
            &ActionState<PlayerAction>,
        ),
        (With<PlayerMarker>, With<Predicted>),
    >,
) {
    for (entity, mut position, mut velocity, action_state) in player_query.iter_mut() {
        let axis_pair = action_state.axis_pair(&PlayerAction::Move);
        if axis_pair != Vec2::ZERO || !action_state.get_pressed().is_empty() {
            info!(
                "🎮 CLIENT PREDICTED: Entity {:?}, Axis: {:?}, Pressed: {:?}",
                entity,
                axis_pair,
                action_state.get_pressed()
            );
        }

        apply_player_action(&time, action_state, &mut position, &mut velocity);
    }
}
