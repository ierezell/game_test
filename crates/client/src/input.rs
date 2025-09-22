use avian3d::prelude::*;
use bevy::prelude::*;

use bevy::window::{CursorGrabMode, PrimaryWindow};
use leafwing_input_manager::prelude::*;

use lightyear::prelude::*;
use shared::input::{
    PLAYER_CAPSULE_HEIGHT, PLAYER_CAPSULE_RADIUS, PlayerAction, shared_player_movement,
};
use shared::protocol::{PlayerColor, PlayerId};

pub struct ClientInputPlugin;

impl Plugin for ClientInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, client_player_movement);
        app.add_observer(handle_player_spawn);
        app.add_observer(handle_other_players_spawn);
    }
}

fn handle_player_spawn(
    trigger: Trigger<OnAdd, (Predicted, Controlled, PlayerId)>,
    player_query: Query<
        (&Name, &PlayerColor, &PlayerId),
        (With<Predicted>, With<Controlled>, With<PlayerId>),
    >,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    local_player_id: Res<crate::app::LocalPlayerId>,
) {
    let entity = trigger.target();
    if let Ok((name, color, player_id)) = player_query.get(entity) {
        if player_id.0.to_bits() == local_player_id.0 {
            info!(
                "🚀 Attaching mesh and input map to PREDICTED player: {:?} ({:?})",
                entity, name
            );
            commands.entity(entity).insert((
                Mesh3d(meshes.add(Capsule3d::new(PLAYER_CAPSULE_RADIUS, PLAYER_CAPSULE_HEIGHT))),
                MeshMaterial3d(materials.add(color.0)),
            ));

            let input_map = get_player_input_map();
            commands.entity(entity).insert(input_map);
        }
    }
}

fn handle_other_players_spawn(
    trigger: Trigger<OnAdd, (PlayerId, Interpolated)>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    player_query: Query<(&Name, &PlayerColor), With<Interpolated>>,
) {
    let entity = trigger.target();
    if let Ok((name, color)) = player_query.get(entity) {
        commands.entity(entity).insert((
            Mesh3d(meshes.add(Capsule3d::new(PLAYER_CAPSULE_RADIUS, PLAYER_CAPSULE_HEIGHT))),
            MeshMaterial3d(materials.add(color.0)),
        ));
        info!(
            "🚀 INTERPOLATED SPAWN! Entity: {:?} Player: {:?}",
            entity, name
        );
    }
}

fn client_player_movement(
    time: Res<Time>,
    mut player_query: Query<
        (
            Entity,
            &mut Rotation,
            &mut LinearVelocity,
            &ActionState<PlayerAction>,
        ),
        (With<PlayerId>, With<Predicted>, With<Controlled>),
    >,
) {
    for (entity, mut rotation, mut velocity, action_state) in player_query.iter_mut() {
        let move_axis_pair = action_state.axis_pair(&PlayerAction::Move);
        let look_axis_pair = action_state.axis_pair(&PlayerAction::Look);

        // move_axis.length_squared() > DEADZONE * DEADZONE || look_axis.length_squared() > DEADZONE * DEADZONE || !action_state.get_pressed().is_empty()
        if move_axis_pair != Vec2::ZERO
            || !action_state.get_pressed().is_empty()
            || look_axis_pair != Vec2::ZERO
        {
            debug!(
                "🎮 Player input: Entity {:?}, Move: {:?}, Look: {:?}",
                entity, move_axis_pair, look_axis_pair
            );
        }

        shared_player_movement(&time, action_state, &mut rotation, &mut velocity);
    }
}

fn get_player_input_map() -> InputMap<PlayerAction> {
    let input_map = InputMap::<PlayerAction>::new([
        (PlayerAction::Jump, KeyCode::Space),
        (PlayerAction::Shoot, KeyCode::Enter),
    ])
    .with_dual_axis(PlayerAction::Move, VirtualDPad::wasd())
    .with_dual_axis(PlayerAction::Move, VirtualDPad::arrow_keys())
    .with_dual_axis(PlayerAction::Look, MouseMove::default());

    input_map
}

pub fn is_cursor_locked(window_query: &Query<&Window, With<PrimaryWindow>>) -> bool {
    window_query.single().map_or(false, |w| {
        w.cursor_options.grab_mode == CursorGrabMode::Locked
    })
}
