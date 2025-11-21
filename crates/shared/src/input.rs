use bevy::prelude::Reflect;
use bevy::prelude::*;

use avian3d::prelude::*;
use leafwing_input_manager::Actionlike;
use leafwing_input_manager::prelude::ActionState;
use serde::{Deserialize, Serialize};

#[derive(
    Clone, Copy, PartialEq, Eq, Hash, Debug, Reflect, Serialize, Deserialize, Actionlike, Default,
)]
pub enum PlayerAction {
    #[default]
    #[actionlike(DualAxis)]
    Move,

    #[actionlike(DualAxis)]
    Look,

    #[actionlike(Button)]
    Jump,

    #[actionlike(Button)]
    Sprint,

    #[actionlike(Button)]
    Shoot,

    #[actionlike(Button)]
    Aim,
}

pub const PLAYER_CAPSULE_RADIUS: f32 = 0.5;
pub const PLAYER_CAPSULE_HEIGHT: f32 = 1.5;
pub const MAX_SPEED: f32 = 5.0;
pub const JUMP_HEIGHT: f32 = 1.5;
const LOOK_DEADZONE_SQUARED: f32 = 0.000001; // 0.001^2
pub const MOUSE_SENSITIVITY: f32 = 0.0025;
const MOVEMENT_DEADZONE_SQUARED: f32 = 0.000001;
pub const PITCH_LIMIT_RADIANS: f32 = std::f32::consts::FRAC_PI_2 - 0.01;
pub const ROTATION_SMOOTHING_RATE: f32 = 25.0; // Higher = more responsive
pub const MOVEMENT_SPEED: f32 = 10.0;
pub const FLOAT_HEIGHT: f32 = 1.5; // Must be greater than distance from center to bottom of collider
#[derive(Component, Reflect, Serialize, Deserialize)]
pub struct FpsController {
    pub gravity: f32,
    pub walk_speed: f32,
    pub run_speed: f32,
    pub forward_speed: f32,
    pub side_speed: f32,
    pub air_speed_cap: f32,
    pub air_acceleration: f32,
    pub max_air_speed: f32,
    pub acceleration: f32,
    pub friction: f32,
    pub traction_normal_cutoff: f32,
    pub friction_speed_cutoff: f32,
    pub jump_speed: f32,
    pub stop_speed: f32,
    pub sensitivity: f32,
    pub pitch: f32,
    pub yaw: f32,
    pub ground_tick: u8,
    pub grounded_distance: f32,
}

impl Default for FpsController {
    fn default() -> Self {
        Self {
            gravity: 9.1,
            walk_speed: 100.0,
            run_speed: 150.0,
            forward_speed: 100.0,
            side_speed: 60.0,
            air_speed_cap: 20.0,
            air_acceleration: 20.0,
            max_air_speed: 60.0,
            acceleration: 10.0,
            friction: 10.0,
            traction_normal_cutoff: 0.7,
            friction_speed_cutoff: 0.1,
            jump_speed: 8.5,
            stop_speed: 1.0,
            sensitivity: 0.001,
            pitch: 0.0,
            yaw: 0.0,
            ground_tick: 0,
            grounded_distance: 0.3,
        }
    }
}

/// Get movement direction from input action state
pub fn get_movement_direction(action_state: &ActionState<PlayerAction>) -> Vec2 {
    let move_input = action_state.axis_pair(&PlayerAction::Move);
    if move_input.length_squared() < MOVEMENT_DEADZONE_SQUARED {
        Vec2::ZERO
    } else {
        move_input.clamp_length_max(1.0)
    }
}

/// Get mouse look delta from input action state
pub fn get_mouse_look_delta(action_state: &ActionState<PlayerAction>) -> Vec2 {
    let look_input = action_state.axis_pair(&PlayerAction::Look);
    if look_input.length_squared() < LOOK_DEADZONE_SQUARED {
        Vec2::ZERO
    } else {
        look_input
    }
}

/// Update camera look from mouse input
pub fn update_look(controller: &mut FpsController, action_state: &ActionState<PlayerAction>) {
    let mouse_delta = get_mouse_look_delta(action_state);

    controller.yaw -= mouse_delta.x * controller.sensitivity;
    controller.pitch -= mouse_delta.y * controller.sensitivity;
    controller.pitch = controller
        .pitch
        .clamp(-PITCH_LIMIT_RADIANS, PITCH_LIMIT_RADIANS);
}
/// Shared player movement function for compatibility
pub fn shared_player_movement(
    time: Time,
    spatial_query: SpatialQueryPipeline,
    entity: Entity,
    action_state: &ActionState<PlayerAction>,
    controller: &mut FpsController,
    transform: &mut Transform,
    velocity: &mut LinearVelocity,
    collider: &Collider,
) {
    update_look(controller, action_state);

    let dt = time.delta_secs();

    // Get movement input
    let move_input = get_movement_direction(action_state);
    let is_sprinting = action_state.pressed(&PlayerAction::Sprint);
    let is_jumping = action_state.pressed(&PlayerAction::Jump);

    // Create movement direction in world space
    // move_input.x = A/D (left/right), move_input.y = W/S (forward/backward)
    let forward = Vec3::new(0.0, 0.0, -move_input.y); // W = forward (-Z), S = backward (+Z)
    let right = Vec3::new(move_input.x, 0.0, 0.0); // D = right (+X), A = left (-X)

    let move_to_world = Mat3::from_rotation_y(controller.yaw);
    let world_forward = move_to_world * forward * controller.forward_speed;
    let world_right = move_to_world * right * controller.side_speed;
    let mut wish_direction = world_forward + world_right;
    let mut wish_speed = wish_direction.length();

    if wish_speed > f32::EPSILON {
        wish_direction /= wish_speed; // Normalize
    }

    let max_speed = if is_sprinting {
        controller.run_speed
    } else {
        controller.walk_speed
    };

    wish_speed = wish_speed.min(max_speed);

    // Ground detection using shape cast
    let filter = SpatialQueryFilter::default().with_excluded_entities([entity]);

    // Use a larger detection distance to catch ground even when spawning high
    let detection_distance = controller.grounded_distance.max(2.0);

    if let Some(hit) = spatial_query.cast_shape(
        collider,
        transform.translation,
        transform.rotation,
        -Dir3::Y,
        &ShapeCastConfig::from_max_distance(detection_distance),
        &filter,
    ) {
        let has_traction = Vec3::dot(hit.normal1, Vec3::Y) > controller.traction_normal_cutoff;
        let is_actually_grounded = hit.distance <= controller.grounded_distance;

        // Ground movement - allow immediate movement even on first ground contact
        if has_traction && (is_actually_grounded || controller.ground_tick >= 1) {
            let lateral_speed = velocity.0.xz().length();
            if lateral_speed > controller.friction_speed_cutoff {
                let control = lateral_speed.max(controller.stop_speed);
                let drop = control * controller.friction * dt;
                let new_speed = ((lateral_speed - drop) / lateral_speed).max(0.0);
                velocity.0.x *= new_speed;
                velocity.0.z *= new_speed;
            } else {
                velocity.0.x = 0.0;
                velocity.0.z = 0.0;
            }

            // Snap to ground if far away (helps with initial spawn)
            if controller.ground_tick == 1 || hit.distance > controller.grounded_distance * 2.0 {
                velocity.0.y = -hit.distance.max(0.1); // Ensure downward movement
            }
        }

        let mut add = calculate_acceleration(
            wish_direction,
            wish_speed,
            controller.acceleration,
            velocity.0,
            dt,
        );

        if !has_traction {
            add.y -= controller.gravity * dt;
        }

        velocity.0 += add;

        if has_traction {
            // Remove velocity into the ground only if actually close to ground
            if is_actually_grounded {
                let linear_velocity = velocity.0;
                velocity.0 -= Vec3::dot(linear_velocity, hit.normal1) * hit.normal1;
            }

            if is_jumping && is_actually_grounded {
                velocity.0.y = controller.jump_speed;
            }
        }

        // Only increment ground tick if we're actually grounded (not just detecting ground far below)
        if is_actually_grounded {
            controller.ground_tick = controller.ground_tick.saturating_add(1);
        }
    } else {
        // Air movement
        controller.ground_tick = 0;
        wish_speed = wish_speed.min(controller.air_speed_cap);

        let mut add = calculate_acceleration(
            wish_direction,
            wish_speed,
            controller.air_acceleration,
            velocity.0,
            dt,
        );

        add.y = -controller.gravity * dt;
        velocity.0 += add;

        let air_speed = velocity.0.xz().length();
        if air_speed > controller.max_air_speed {
            let ratio = controller.max_air_speed / air_speed;
            velocity.0.x *= ratio;
            velocity.0.z *= ratio;
        }
    }

    // Update transform rotation
    transform.rotation = Quat::from_euler(EulerRot::YXZ, controller.yaw, 0.0, 0.0);
}

/// Calculate acceleration for movement
fn calculate_acceleration(
    wish_direction: Vec3,
    wish_speed: f32,
    acceleration: f32,
    velocity: Vec3,
    dt: f32,
) -> Vec3 {
    let velocity_projection = Vec3::dot(velocity, wish_direction);
    let add_speed = wish_speed - velocity_projection;

    if add_speed <= 0.0 {
        return Vec3::ZERO;
    }

    let acceleration_speed = (acceleration * wish_speed * dt).min(add_speed);
    wish_direction * acceleration_speed
}
