use avian3d::prelude::*;
use bevy::prelude::*;
use leafwing_input_manager::prelude::ActionState;
use serde::{Deserialize, Serialize};

use crate::inputs::input::PlayerAction;

pub const WALK_SPEED: f32 = 60.0;
pub const RUN_SPEED: f32 = 100.0;
pub const AIR_SPEED_CAP: f32 = 15.0;
pub const AIR_ACCELERATION: f32 = 15.0;
pub const MAX_AIR_SPEED: f32 = 50.0;
pub const ACCELERATION: f32 = 8.0;
pub const FRICTION: f32 = 10.0;
pub const JUMP_SPEED: f32 = 8.5;
pub const GRAVITY: f32 = 9.1;
pub const TRACTION_NORMAL_CUTOFF: f32 = 0.7;
pub const FRICTION_SPEED_CUTOFF: f32 = 0.1;
pub const STOP_SPEED: f32 = 1.0;
pub const GROUNDED_DISTANCE: f32 = 0.3;

/// Ground detection state - separated for testability
#[derive(Component, Reflect, Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct GroundState {
    pub is_grounded: bool,
    pub ground_normal: Vec3,
    pub ground_distance: f32,
    pub ground_tick: u8,
}

pub fn detect_ground(
    entity: Entity,
    collider: &Collider,
    position: Vec3,
    rotation: Quat,
    spatial_query: &SpatialQueryPipeline,
) -> GroundState {
    let filter = SpatialQueryFilter::default().with_excluded_entities([entity]);
    let detection_distance = GROUNDED_DISTANCE.max(2.0);

    if let Some(hit) = spatial_query.cast_shape(
        collider,
        position,
        rotation,
        -Dir3::Y,
        &ShapeCastConfig::from_max_distance(detection_distance),
        &filter,
    ) {
        let has_traction = Vec3::dot(hit.normal1, Vec3::Y) > TRACTION_NORMAL_CUTOFF;
        let is_grounded = hit.distance <= GROUNDED_DISTANCE;

        GroundState {
            is_grounded: is_grounded && has_traction,
            ground_normal: hit.normal1,
            ground_distance: hit.distance,
            ground_tick: 0, // Will be updated by caller
        }
    } else {
        GroundState::default()
    }
}

/// Calculate acceleration for desired movement direction
pub fn calculate_acceleration(
    wish_direction: Vec3,
    wish_speed: f32,
    acceleration: f32,
    current_velocity: Vec3,
    dt: f32,
) -> Vec3 {
    let velocity_projection = Vec3::dot(current_velocity, wish_direction);
    let add_speed = wish_speed - velocity_projection;

    if add_speed <= 0.0 {
        return Vec3::ZERO;
    }

    let acceleration_speed = (acceleration * wish_speed * dt).min(add_speed);
    wish_direction * acceleration_speed
}

/// Apply friction to ground movement
pub fn apply_ground_friction(velocity: &mut LinearVelocity, dt: f32) {
    let lateral_speed = velocity.0.xz().length();

    if lateral_speed > FRICTION_SPEED_CUTOFF {
        let control = lateral_speed.max(STOP_SPEED);
        let drop = control * FRICTION * dt;
        let new_speed = ((lateral_speed - drop) / lateral_speed).max(0.0);
        velocity.0.x *= new_speed;
        velocity.0.z *= new_speed;
    } else {
        velocity.0.x = 0.0;
        velocity.0.z = 0.0;
    }
}

pub fn remove_ground_penetration(velocity: &mut LinearVelocity, ground_normal: Vec3) {
    let normal_velocity = Vec3::dot(velocity.0, ground_normal) * ground_normal;
    if Vec3::dot(normal_velocity, ground_normal) < 0.0 {
        velocity.0 -= normal_velocity;
    }
}

pub fn clamp_max_velocity(velocity: &mut LinearVelocity, max_velocity: f32) {
    let speed = velocity.0.length();
    if speed > max_velocity {
        velocity.0 = velocity.0.normalize() * max_velocity;
    }
}

pub fn get_wish_direction(
    input: Vec2,
    yaw: f32,
    forward_speed: f32,
    side_speed: f32,
) -> (Vec3, f32) {
    let forward = Vec3::new(0.0, 0.0, -input.y);
    let right = Vec3::new(input.x, 0.0, 0.0);

    let move_to_world = Mat3::from_rotation_y(yaw);
    let world_forward = move_to_world * forward * forward_speed;
    let world_right = move_to_world * right * side_speed;

    let mut wish_direction = world_forward + world_right;
    let wish_speed = wish_direction.length();

    if wish_speed > f32::EPSILON {
        wish_direction /= wish_speed;
    }

    (wish_direction, wish_speed)
}

pub fn update_ground_detection(
    spatial_query: Res<SpatialQueryPipeline>,
    mut query: Query<(Entity, &Position, &Rotation, &Collider, &mut GroundState)>,
) {
    for (entity, position, rotation, collider, mut ground_state) in query.iter_mut() {
        let detected = detect_ground(entity, collider, position.0, rotation.0, &spatial_query);

        ground_state.is_grounded = detected.is_grounded;
        ground_state.ground_normal = detected.ground_normal;
        ground_state.ground_distance = detected.ground_distance;

        if detected.is_grounded {
            ground_state.ground_tick = ground_state.ground_tick.saturating_add(1);
        } else {
            ground_state.ground_tick = 0;
        }
    }
}

/// System: Apply movement based on input and ground state
pub fn apply_movement(
    time: Res<Time>,
    mut query: Query<(
        &ActionState<PlayerAction>,
        &GroundState,
        &Rotation,
        &mut LinearVelocity,
    )>,
) {
    let dt = time.delta_secs();

    for (action_state, ground_state, rotation, mut velocity) in query.iter_mut() {
        // Get input
        let move_input = action_state.axis_pair(&PlayerAction::Move);
        let (yaw, _, _) = rotation.0.to_euler(EulerRot::YXZ);

        // DEBUG: Log when movement is applied
        if move_input.length() > 0.1 {
            bevy::log::debug!(
                "apply_movement: input={:?}, camera.yaw={:.2}, grounded={}, velocity={:?}",
                move_input,
                yaw,
                ground_state.is_grounded,
                velocity.0
            );
        }
        let is_sprinting = action_state.pressed(&PlayerAction::Sprint);
        let is_jumping = action_state.pressed(&PlayerAction::Jump);

        // Calculate wish direction using camera yaw for camera-relative movement
        let (wish_direction, mut wish_speed) = get_wish_direction(move_input, yaw, 100.0, 60.0);

        // Apply speed limits
        let max_speed = if is_sprinting { RUN_SPEED } else { WALK_SPEED };
        wish_speed = wish_speed.min(max_speed);

        // Ground movement
        if ground_state.is_grounded {
            apply_ground_friction(&mut velocity, dt);

            let add =
                calculate_acceleration(wish_direction, wish_speed, ACCELERATION, velocity.0, dt);
            velocity.0 += add;

            remove_ground_penetration(&mut velocity, ground_state.ground_normal);

            if is_jumping {
                velocity.0.y = JUMP_SPEED;
            }
        } else {
            // Air movement
            wish_speed = wish_speed.min(AIR_SPEED_CAP);
            let mut add = calculate_acceleration(
                wish_direction,
                wish_speed,
                AIR_ACCELERATION,
                velocity.0,
                dt,
            );

            add.y = -GRAVITY * dt;
            velocity.0 += add;

            let air_speed = velocity.0.xz().length();
            if air_speed > MAX_AIR_SPEED {
                let ratio = MAX_AIR_SPEED / air_speed;
                velocity.0.x *= ratio;
                velocity.0.z *= ratio;
            }
        }

        clamp_max_velocity(&mut velocity, 50.0);
    }
}

#[cfg(test)]
mod tests {
    use super::{
        GroundState, LinearVelocity, apply_ground_friction, calculate_acceleration,
        clamp_max_velocity, get_wish_direction,
    };
    use crate::inputs::input::PlayerAction;
    use crate::inputs::look::update_player_rotation_from_input;
    use crate::protocol::{CharacterMarker, PlayerId};
    use avian3d::prelude::{Position, Rotation};
    use bevy::prelude::{
        App, FixedUpdate, IntoScheduleConfigs, MinimalPlugins, Res, Time, Update, Vec2, Vec3,
    };
    use lightyear::prelude::{Controlled, PeerId, Predicted};
    use leafwing_input_manager::prelude::ActionState;

    fn integrate_position(mut q: bevy::prelude::Query<(&mut Position, &LinearVelocity)>, time: Res<Time>) {
        for (mut position, velocity) in q.iter_mut() {
            position.0 += velocity.0 * time.delta_secs();
        }
    }

    fn step(app: &mut App, dt: std::time::Duration) {
        app.insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(dt));
        app.update();
    }

    #[test]
    fn acceleration_only_when_needed() {
        let wish_direction = Vec3::new(1.0, 0.0, 0.0);
        let add = calculate_acceleration(wish_direction, 10.0, 8.0, Vec3::ZERO, 0.1);
        assert!(add.x > 0.0);

        let saturated = calculate_acceleration(wish_direction, 10.0, 8.0, Vec3::new(12.0, 0.0, 0.0), 0.1);
        assert_eq!(saturated, Vec3::ZERO);
    }

    #[test]
    fn ground_friction_reduces_lateral_speed() {
        let mut velocity = LinearVelocity(Vec3::new(8.0, 0.0, 0.0));
        apply_ground_friction(&mut velocity, 0.1);
        assert!(velocity.0.x < 8.0);
        assert_eq!(velocity.0.y, 0.0);
    }

    #[test]
    fn velocity_clamp_enforces_max_speed() {
        let mut velocity = LinearVelocity(Vec3::new(30.0, 40.0, 0.0));
        clamp_max_velocity(&mut velocity, 10.0);
        assert!((velocity.0.length() - 10.0).abs() < 0.001);
    }

    #[test]
    fn wish_direction_uses_yaw_rotation() {
        let (dir, speed) = get_wish_direction(Vec2::new(0.0, 1.0), std::f32::consts::FRAC_PI_2, 100.0, 60.0);
        assert!(speed > 0.0);
        assert!(dir.x.abs() > 0.9, "Direction should rotate into x axis, got {:?}", dir);
    }

    #[test]
    fn keyboard_forward_then_mouse_turn_then_forward_changes_path() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, update_player_rotation_from_input);
        app.add_systems(FixedUpdate, (super::apply_movement, integrate_position).chain());

        let mut action_state = ActionState::<PlayerAction>::default();
        action_state.enable();
        action_state.set_axis_pair(&PlayerAction::Move, Vec2::new(0.0, 1.0));
        action_state.set_axis_pair(&PlayerAction::Look, Vec2::ZERO);

        let player = app.world_mut().spawn((
            PlayerId(PeerId::Netcode(1)),
            Predicted,
            Controlled,
            action_state,
            GroundState {
                is_grounded: true,
                ground_normal: Vec3::Y,
                ground_distance: 0.0,
                ground_tick: 1,
            },
            LinearVelocity(Vec3::ZERO),
            Position::new(Vec3::ZERO),
            Rotation::default(),
            CharacterMarker,
        )).id();

        for _ in 0..30 {
            step(&mut app, std::time::Duration::from_millis(16));
        }

        let pos_after_first_forward = app
            .world()
            .get::<Position>(player)
            .expect("Player should have Position")
            .0;
        assert!(
            pos_after_first_forward.z < -0.5,
            "First forward movement should move mostly on -Z axis"
        );

        {
            let world = app.world_mut();
            let mut action = world
                .get_mut::<ActionState<PlayerAction>>(player)
                .expect("Player should have ActionState");
            action.set_axis_pair(&PlayerAction::Look, Vec2::new(-785.0, 0.0));
        }
        step(&mut app, std::time::Duration::from_millis(16));
        {
            let world = app.world_mut();
            let mut action = world
                .get_mut::<ActionState<PlayerAction>>(player)
                .expect("Player should have ActionState");
            action.set_axis_pair(&PlayerAction::Look, Vec2::ZERO);
            action.set_axis_pair(&PlayerAction::Move, Vec2::new(0.0, 1.0));
        }

        for _ in 0..30 {
            step(&mut app, std::time::Duration::from_millis(16));
        }

        let pos_after_second_forward = app
            .world()
            .get::<Position>(player)
            .expect("Player should still have Position")
            .0;

        let second_segment = pos_after_second_forward - pos_after_first_forward;

        assert!(
            second_segment.x.abs() > 0.5,
            "Second forward after yaw turn should add significant lateral displacement, delta={:?}",
            second_segment
        );
        assert!(
            second_segment.x.abs() > second_segment.z.abs() * 0.5,
            "After yaw turn, lateral movement should dominate enough, delta={:?}",
            second_segment
        );
    }
}
