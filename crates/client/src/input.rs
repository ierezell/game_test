use avian3d::prelude::*;
use bevy::prelude::*;

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
    trigger: Trigger<OnAdd, (PlayerId, Predicted)>,
    player_query: Query<(&Name, &PlayerColor), (With<Predicted>, With<Controlled>)>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let entity = trigger.target();
    if let Ok((name, color)) = player_query.single() {
        commands.entity(entity).insert((
            Mesh3d(meshes.add(Capsule3d::new(PLAYER_CAPSULE_RADIUS, PLAYER_CAPSULE_HEIGHT))),
            MeshMaterial3d(materials.add(color.0)),
            // PlayerPhysicsBundle::default(),
        ));

        let input_map = get_player_input_map();

        commands.entity(entity).insert(input_map);
        info!(
            "🚀 PREDICTED SPAWN! Entity: {:?} Player: {:?} ",
            entity, name,
        );
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
            &mut Position,
            &mut Rotation,
            &mut LinearVelocity,
            &ActionState<PlayerAction>,
        ),
        (With<PlayerId>, With<Predicted>, With<Controlled>),
    >,
) {
    for (entity, mut position, mut rotation, mut velocity, action_state) in player_query.iter_mut()
    {
        let move_axis_pair = action_state.axis_pair(&PlayerAction::Move);
        let look_axis_pair = action_state.axis_pair(&PlayerAction::Look);

        // move_axis.length_squared() > DEADZONE * DEADZONE || look_axis.length_squared() > DEADZONE * DEADZONE || !action_state.get_pressed().is_empty()
        if move_axis_pair != Vec2::ZERO
            || !action_state.get_pressed().is_empty()
            || look_axis_pair != Vec2::ZERO
        {
            info!(
                "🎮 Player input: Entity {:?}, Move: {:?}, Look: {:?}",
                entity, move_axis_pair, look_axis_pair
            );
        }

        shared_player_movement(
            &time,
            action_state,
            &mut position,
            &mut rotation,
            &mut velocity,
        );
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
