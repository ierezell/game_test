use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_enhanced_input::prelude::{Action, Actions};
use serde::{Deserialize, Serialize};

use crate::components::stamina::Stamina;
use crate::inputs::look::update_player_rotation_from_input;
use crate::inputs::{Jump, Move, PlayerActions, Sprint};

pub const WALK_SPEED: f32 = 20.0;
pub const RUN_SPEED: f32 = 40.0;
pub const AIR_SPEED_CAP: f32 = 15.0;
pub const AIR_ACCELERATION: f32 = 25.0;
pub const MAX_AIR_SPEED: f32 = 50.0;
pub const ACCELERATION: f32 = 14.0;
pub const FRICTION: f32 = 15.0;
pub const JUMP_SPEED: f32 = 8.5;
pub const GRAVITY: f32 = 9.1;
pub const TRACTION_NORMAL_CUTOFF: f32 = 0.7;
pub const FRICTION_SPEED_CUTOFF: f32 = 0.5;
pub const STOP_SPEED: f32 = 5.0;
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
    spatial_query: &SpatialQuery,
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
            ground_tick: 0,
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
    input: &Action<Move>,
    yaw: f32,
    forward_speed: f32,
    side_speed: f32,
) -> (Vec3, f32) {
    let input_vec: Vec2 = **input;
    let forward = Vec3::new(0.0, 0.0, -input_vec.y);
    let right = Vec3::new(input_vec.x, 0.0, 0.0);

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
    spatial_query: SpatialQuery,
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
///
/// Supports two Action<T> access patterns:
/// 1. Production: `Action<T>` spawned as related children of `Actions<PlayerActions>`
///    via the `actions!` macro — resolved by iterating the relationship.
/// 2. Legacy/test: `Action<T>` placed directly as a component on the entity
///    — resolved via a fallback query on the entity itself.
#[allow(clippy::collapsible_if)]
pub fn apply_movement(
    time: Res<Time>,
    mut player_query: Query<
        (
            Option<&Actions<PlayerActions>>,
            Entity,
            &GroundState,
            &Rotation,
            Option<&mut Stamina>,
            &mut LinearVelocity,
        ),
        With<PlayerActions>,
    >,
    move_query: Query<&Action<Move>>,
    sprint_query: Query<&Action<Sprint>>,
    jump_query: Query<&Action<Jump>>,
) {
    let dt = time.delta_secs();

    for (actions, entity, ground_state, rotation, mut stamina, mut velocity) in
        player_query.iter_mut()
    {
        // In bevy_enhanced_input 0.26, `actions!` spawns each `Action<T>` as a
        // *related child* of the `Actions<PlayerActions>` context rather than as
        // a component on the player entity. Read them back through the
        // relationship; otherwise WASD/mouse look never reach the movement code
        // at runtime (the parent entity has no `Action<T>` of its own).
        //
        // When `actions` is `None` (tests that insert `Action<T>` directly on the
        // entity without the `actions!` macro), fall back to reading them from
        // the entity itself.
        let mut move_action: Option<&Action<Move>> = None;
        let mut is_sprinting = false;
        let mut is_jumping = false;

        if let Some(actions) = actions {
            for action_entity in actions.iter() {
                if move_action.is_none() {
                    if let Ok(action) = move_query.get(action_entity) {
                        move_action = Some(action);
                    }
                }
                if let Ok(sprint) = sprint_query.get(action_entity) {
                    is_sprinting = **sprint;
                }
                if let Ok(jump) = jump_query.get(action_entity) {
                    is_jumping = **jump;
                }
            }
        } else {
            // Legacy/test path: Action<T> components are directly on the entity
            if let Ok(action) = move_query.get(entity) {
                move_action = Some(action);
            }
            if let Ok(sprint) = sprint_query.get(entity) {
                is_sprinting = **sprint;
            }
            if let Ok(jump) = jump_query.get(entity) {
                is_jumping = **jump;
            }
        }

        let Some(move_action) = move_action else {
            continue;
        };

        // Get movement input from bevy_enhanced_input - deref to output type
        let move_input: Vec2 = **move_action;

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

        // Calculate wish direction using camera yaw for camera-relative movement
        let (wish_direction, mut wish_speed) = get_wish_direction(move_action, yaw, 100.0, 60.0);

        // Apply speed limits
        let stamina_allows_sprint = stamina.as_ref().map(|s| s.can_sprint()).unwrap_or(true);
        let max_speed = if is_sprinting && stamina_allows_sprint {
            RUN_SPEED
        } else {
            WALK_SPEED
        };
        wish_speed = wish_speed.min(max_speed);

        // Stamina drain/regen (server-authoritative; client predicted via same logic)
        if let Some(stam) = stamina.as_mut() {
            let now = time.elapsed_secs();
            if is_sprinting && stamina_allows_sprint && ground_state.is_grounded {
                let amount = stam.drain_rate * dt;
                stam.drain(amount, now);
            } else if stam.can_regenerate(now) {
                let amount = stam.regen_rate * dt;
                stam.regenerate(amount, now);
            }
        }

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

/// Integrate `LinearVelocity` into avian `Position` and mirror the result onto
/// the bevy `Transform` so that child entities (e.g. the first-person camera)
/// follow the character.
///
/// This only touches entities that are *not* driven by avian's own simulation
/// (`RigidBody::Dynamic`/`Static`/`Kinematic`). For simulated bodies avian
/// advances the position itself and `PhysicsTransformConfig` syncs it back to
/// the `Transform`, so we leave them alone here to avoid double integration.
pub fn integrate_position_from_velocity(
    time: Res<Time>,
    mut query: Query<
        (
            &mut Position,
            &Rotation,
            &LinearVelocity,
            Option<&mut Transform>,
        ),
        Without<RigidBody>,
    >,
) {
    let dt = time.delta_secs();
    for (mut position, rotation, velocity, mut transform) in query.iter_mut() {
        position.0 += velocity.0 * dt;
        if let Some(transform) = transform.as_mut() {
            transform.translation = position.0;
            transform.rotation = rotation.0;
        }
    }
}

/// Wires the character movement systems into the app.
///
/// `headless` disables the mouse-driven camera look so headless / server-less
/// runs do not try to consume live cursor input. Movement, ground detection and
/// transform integration always run (they can be driven by injected actions,
/// e.g. in tests or by an RL agent).
pub struct MovementPlugin {
    pub headless: bool,
}

impl MovementPlugin {
    pub fn new(headless: bool) -> Self {
        Self { headless }
    }
}

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        let headless = self.headless;
        app.add_systems(
            FixedUpdate,
            (
                update_ground_detection,
                update_player_rotation_from_input.run_if(move || !headless),
                apply_movement,
                integrate_position_from_velocity,
            )
                .chain(),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::{
        GroundState, LinearVelocity, apply_ground_friction, calculate_acceleration,
        clamp_max_velocity, get_wish_direction,
    };
    use crate::inputs::look::update_player_rotation_from_input;
    use crate::inputs::{Jump, Move, PlayerActions, Sprint, get_player_actions};
    use crate::protocol::{CharacterMarker, PlayerId};
    use avian3d::prelude::{Position, Rotation};
    use bevy::prelude::{
        App, FixedUpdate, GamepadAxis, IntoScheduleConfigs, KeyCode, MinimalPlugins, Quat, Res,
        Time, Update, Vec2, Vec3,
    };
    use bevy_enhanced_input::prelude::*;
    use lightyear::prelude::{Controlled, PeerId, Predicted};

    fn integrate_position(
        mut q: bevy::prelude::Query<(&mut Position, &LinearVelocity)>,
        time: Res<Time>,
    ) {
        for (mut position, velocity) in q.iter_mut() {
            position.0 += velocity.0 * time.delta_secs();
        }
    }

    fn step(app: &mut App, dt: std::time::Duration) {
        app.insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(dt));
        app.update();
    }

    /// Regression test for WASD binding bug: in bevy_enhanced_input 0.26,
    /// binding individual KeyCode entries to a DualAxis (Vec2) action without
    /// modifiers causes each key's Bool value to convert to Vec2::X. This means
    /// pressing W, A, S, or D all produced Vec2::X (rightward movement) instead
    /// of proper forward/left/backward/right directions.
    ///
    /// This test verifies that get_wish_direction correctly interprets each
    /// WASD-style input vector as orthogonal movement when yaw = 0:
    ///   W (0, +1) → forward (-Z)
    ///   A (-1, 0) → left (-X)
    ///   S (0, -1) → backward (+Z)
    ///   D (+1, 0) → right (+X)
    #[test]
    #[allow(clippy::explicit_auto_deref)]
    fn wasd_input_vectors_produce_orthogonal_directions() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(EnhancedInputPlugin)
            .add_input_context::<PlayerActions>()
            .finish();

        let entity = app
            .world_mut()
            .spawn((PlayerActions, Action::<Move>::default()))
            .id();

        let yaw = 0.0;

        // W → forward (-Z)
        {
            let mut action = app.world_mut().get_mut::<Action<Move>>(entity).unwrap();
            **action = Vec2::new(0.0, 1.0);
            let (dir, speed) = get_wish_direction(&*action, yaw, 100.0, 60.0);
            assert!(speed > 0.0);
            assert!(dir.x.abs() < 0.1, "W should not produce X movement");
            assert!(
                dir.z < -0.9,
                "W should move along -Z (forward), got {:?}",
                dir
            );
        }

        // A → left (-X)
        {
            let mut action = app.world_mut().get_mut::<Action<Move>>(entity).unwrap();
            **action = Vec2::new(-1.0, 0.0);
            let (dir, speed) = get_wish_direction(&*action, yaw, 100.0, 60.0);
            assert!(speed > 0.0);
            assert!(dir.x < -0.9, "A should move along -X (left), got {:?}", dir);
            assert!(dir.z.abs() < 0.1, "A should not produce Z movement");
        }

        // S → backward (+Z)
        {
            let mut action = app.world_mut().get_mut::<Action<Move>>(entity).unwrap();
            **action = Vec2::new(0.0, -1.0);
            let (dir, speed) = get_wish_direction(&*action, yaw, 100.0, 60.0);
            assert!(speed > 0.0);
            assert!(dir.x.abs() < 0.1, "S should not produce X movement");
            assert!(
                dir.z > 0.9,
                "S should move along +Z (backward), got {:?}",
                dir
            );
        }

        // D → right (+X)
        {
            let mut action = app.world_mut().get_mut::<Action<Move>>(entity).unwrap();
            **action = Vec2::new(1.0, 0.0);
            let (dir, speed) = get_wish_direction(&*action, yaw, 100.0, 60.0);
            assert!(speed > 0.0);
            assert!(dir.x > 0.9, "D should move along +X (right), got {:?}", dir);
            assert!(dir.z.abs() < 0.1, "D should not produce Z movement");
        }
    }

    /// Regression test for the WASD KeyCode binding bug in bevy_enhanced_input 0.26.
    /// Verifies that the production get_player_actions() bundle correctly maps:
    ///   W → +Y (via SwizzleAxis::YXZ)
    ///   A → -X (via Negate)
    ///   S → -Y (via Negate + SwizzleAxis::YXZ)
    ///   D → +X (no modifier)
    /// Without these modifiers, all keys would default to Vec2::X (+X), causing
    /// all WASD keys to move the character rightward.
    #[test]
    fn production_wasd_bindings_have_correct_axis_modifiers() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(EnhancedInputPlugin)
            .add_input_context::<PlayerActions>()
            .finish();

        let entity = app.world_mut().spawn(get_player_actions()).id();

        let actions = app
            .world()
            .get::<Actions<PlayerActions>>(entity)
            .expect("PlayerActions entity should have Actions relationship");

        let move_action_entity = *actions
            .iter()
            .find(|e| app.world().get::<Action<Move>>(**e).is_some())
            .expect("Move action should exist as a related child entity");

        let bindings = app
            .world()
            .get::<Bindings>(move_action_entity)
            .expect("Move action should have Bindings relationship");

        let binding_count = bindings.iter().count();
        assert!(
            binding_count >= 4,
            "Move action should have at least 4 key bindings (W,A,S,D), got {}",
            binding_count
        );

        let mut found_w = false;
        let mut found_a = false;
        let mut found_s = false;
        let mut found_d = false;

        for &binding_entity in bindings.iter() {
            let binding = app.world().get::<Binding>(binding_entity);
            if binding.is_none() {
                continue;
            }

            let binding_key = match binding.unwrap() {
                Binding::Keyboard { key, .. } => Some(key),
                _ => None,
            };

            let has_swizzle_yxz = app
                .world()
                .get::<SwizzleAxis>(binding_entity)
                .is_some_and(|s| matches!(s, SwizzleAxis::YXZ));
            let has_negate = app.world().get::<Negate>(binding_entity).is_some();

            match binding_key {
                Some(KeyCode::KeyW) => {
                    found_w = true;
                    assert!(
                        has_swizzle_yxz,
                        "W binding must have SwizzleAxis::YXZ to map to Y axis"
                    );
                }
                Some(KeyCode::KeyA) => {
                    found_a = true;
                    assert!(
                        has_negate,
                        "A binding must have Negate to produce -X (left)"
                    );
                }
                Some(KeyCode::KeyS) => {
                    found_s = true;
                    assert!(
                        has_negate && has_swizzle_yxz,
                        "S binding must have both Negate and SwizzleAxis::YXZ"
                    );
                }
                Some(KeyCode::KeyD) => {
                    found_d = true;
                    assert!(
                        !has_swizzle_yxz && !has_negate,
                        "D binding should have no modifiers (default +X)"
                    );
                }
                _ => {}
            }
        }

        assert!(found_w, "W binding not found");
        assert!(found_a, "A binding not found");
        assert!(found_s, "S binding not found");
        assert!(found_d, "D binding not found");
    }

    #[test]
    fn acceleration_only_when_needed() {
        let wish_direction = Vec3::new(1.0, 0.0, 0.0);
        let add = calculate_acceleration(wish_direction, 10.0, 8.0, Vec3::ZERO, 0.1);
        assert!(add.x > 0.0);

        let saturated =
            calculate_acceleration(wish_direction, 10.0, 8.0, Vec3::new(12.0, 0.0, 0.0), 0.1);
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
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(EnhancedInputPlugin)
            .add_input_context::<PlayerActions>()
            .finish();

        let entity = app
            .world_mut()
            .spawn((
                PlayerActions,
                actions!(PlayerActions[
                    (Action::<Move>::new(), bindings![
                        (KeyCode::KeyW, SwizzleAxis::YXZ),
                        (KeyCode::KeyA, Negate::all()),
                        (KeyCode::KeyS, Negate::all(), SwizzleAxis::YXZ),
                        KeyCode::KeyD,
                        GamepadAxis::LeftStickX,
                        (GamepadAxis::LeftStickY, SwizzleAxis::YXZ),
                    ]),
                ]),
                Action::<Move>::default(),
            ))
            .id();

        let mut move_action = app.world_mut().get_mut::<Action<Move>>(entity).unwrap();
        **move_action = Vec2::new(0.0, 1.0);

        let action_ref = app.world().get::<Action<Move>>(entity).unwrap();
        let (dir, speed) = get_wish_direction(action_ref, std::f32::consts::FRAC_PI_2, 100.0, 60.0);
        assert!(speed > 0.0);
        assert!(
            dir.x.abs() > 0.9,
            "Direction should rotate into x axis, got {:?}",
            dir
        );
    }

    #[test]
    fn diagonal_input_is_normalized_to_unit_length() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(EnhancedInputPlugin)
            .add_input_context::<PlayerActions>()
            .finish();

        let entity = app
            .world_mut()
            .spawn((PlayerActions, Action::<Move>::default()))
            .id();

        // Set diagonal input: forward + right
        {
            let mut move_action = app.world_mut().get_mut::<Action<Move>>(entity).unwrap();
            **move_action = Vec2::new(1.0, 1.0);
        }

        let action_ref = app.world().get::<Action<Move>>(entity).unwrap();
        let (dir, speed) = get_wish_direction(action_ref, 0.0, 100.0, 60.0);

        // Combined speed should be the magnitude of the vector sum, not the sum of speeds
        let expected_magnitude = (100.0_f32.powi(2) + 60.0_f32.powi(2)).sqrt();
        assert!(
            (speed - expected_magnitude).abs() < 0.001,
            "Diagonal speed should be sqrt(100^2 + 60^2) = {}, got {:?}",
            expected_magnitude,
            speed
        );

        // Direction should be normalized to unit length
        assert!(
            (dir.length() - 1.0).abs() < 0.001,
            "Diagonal direction should be normalized to unit length, got {:?}",
            dir
        );
    }

    #[test]
    fn keyboard_forward_then_mouse_turn_then_forward_changes_path() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(EnhancedInputPlugin)
            .add_input_context::<PlayerActions>()
            .finish();
        app.add_systems(Update, update_player_rotation_from_input);
        app.add_systems(
            FixedUpdate,
            (super::apply_movement, integrate_position).chain(),
        );

        let entity = app
            .world_mut()
            .spawn((
                PlayerActions,
                Action::<Move>::default(),
                Action::<Sprint>::default(),
                Action::<Jump>::default(),
                PlayerId(PeerId::Netcode(1)),
                Predicted,
                Controlled,
                CharacterMarker,
                GroundState {
                    is_grounded: true,
                    ground_normal: Vec3::Y,
                    ground_distance: 0.0,
                    ground_tick: 1,
                },
                LinearVelocity(Vec3::ZERO),
                Position::new(Vec3::ZERO),
                Rotation::default(),
            ))
            .id();

        // Set movement input
        let mut move_action = app.world_mut().get_mut::<Action<Move>>(entity).unwrap();
        **move_action = Vec2::new(0.0, 1.0);

        for _ in 0..30 {
            step(&mut app, std::time::Duration::from_millis(16));
        }

        let pos_after_first_forward = app
            .world()
            .get::<Position>(entity)
            .expect("Player should have Position")
            .0;
        assert!(
            pos_after_first_forward.z < -0.5,
            "First forward movement should move mostly on -Z axis"
        );

        // Simulate mouse turn (yaw rotation)
        {
            let mut rotation = app.world_mut().get_mut::<Rotation>(entity).unwrap();
            rotation.0 = Quat::from_rotation_y(std::f32::consts::FRAC_PI_2);
        }

        step(&mut app, std::time::Duration::from_millis(16));
        {
            let mut move_action = app.world_mut().get_mut::<Action<Move>>(entity).unwrap();
            **move_action = Vec2::new(0.0, 1.0);
        }

        for _ in 0..30 {
            step(&mut app, std::time::Duration::from_millis(16));
        }

        let pos_after_second_forward = app
            .world()
            .get::<Position>(entity)
            .expect("Player should still have Position")
            .0;

        let second_segment = pos_after_second_forward - pos_after_first_forward;

        assert!(
            second_segment.x.abs() > 0.5,
            "Second forward after yaw turn should add significant lateral displacement, delta={:?}",
            second_segment
        );

        let first_dir = pos_after_first_forward.normalize_or_zero();
        let second_dir = second_segment.normalize_or_zero();
        let heading_dot = first_dir.dot(second_dir).abs();

        assert!(
            heading_dot < 0.95,
            "After yaw turn, heading should change materially, dot={}, first_dir={:?}, second_dir={:?}",
            heading_dot,
            first_dir,
            second_dir
        );
    }

    #[test]
    fn sprint_drain_reduces_stamina_and_marks_exhausted() {
        use crate::FIXED_TIMESTEP_HZ;
        use crate::components::stamina::STAMINA_DRAIN_RATE;
        use crate::components::stamina::Stamina;

        let mut stamina = Stamina::default();
        let dt = 1.0 / FIXED_TIMESTEP_HZ as f32;

        let drains_per_tick = STAMINA_DRAIN_RATE * dt;
        stamina.drain(drains_per_tick, 0.0);
        assert!(
            stamina.current < crate::components::stamina::STAMINA_MAX,
            "Stamina should decrease after drain"
        );
        assert!(!stamina.exhausted, "Should not be exhausted while > 0");

        for _ in 0..1000 {
            stamina.drain(drains_per_tick, 0.5);
        }
        assert!(
            stamina.exhausted,
            "Should be exhausted when stamina reaches 0"
        );
        assert!(
            !stamina.can_sprint(),
            "Cannot sprint when stamina is depleted"
        );
    }

    #[test]
    fn stamina_regenerates_after_delay_without_draining() {
        use crate::components::stamina::STAMINA_MAX;
        use crate::components::stamina::STAMINA_REGEN_DELAY;
        use crate::components::stamina::Stamina;

        let mut stamina = Stamina {
            current: 0.0,
            exhausted: true,
            last_drain_time: 0.0,
            ..Stamina::default()
        };

        assert!(
            !stamina.can_regenerate(0.5),
            "Should not regenerate within regen_delay"
        );

        let now = STAMINA_REGEN_DELAY + 0.01;
        assert!(
            stamina.can_regenerate(now),
            "Should regenerate after regen_delay"
        );

        stamina.regenerate(STAMINA_MAX, now);
        assert!(stamina.current > 0.0);
        assert!(
            !stamina.exhausted,
            "Should not be exhausted after full regeneration"
        );
    }

    #[test]
    fn exhausted_player_cannot_sprint() {
        use crate::components::stamina::Stamina;

        let stamina = Stamina {
            current: 0.0,
            exhausted: true,
            ..Stamina::default()
        };

        assert!(!stamina.can_sprint());
    }
}
