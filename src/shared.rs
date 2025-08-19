//! Shared logic that runs on both client and server for consistent simulation.

use crate::player::PlayerAction;
use crate::protocol::*;
use avian3d::prelude::*;
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

/// Local weapon fire event (different from the networked WeaponFireMessage)
#[derive(Event, Debug)]
pub struct LocalWeaponFireEvent {
    pub player_id: u64,
    pub origin: Vec3,
    pub direction: Vec3,
    pub damage: f32,
}

#[derive(Clone)]
pub struct SharedPlugin;

impl Plugin for SharedPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<LocalWeaponFireEvent>();
        app.add_systems(
            FixedUpdate,
            (
                shared_player_movement,
                shared_avian_player_movement,
                shared_weapon_system,
            ),
        );
    }
}

// Movement constants
pub const PLAYER_SPEED: f32 = 5.0;
pub const PLAYER_SPRINT_SPEED: f32 = 8.0;
pub const PLAYER_CROUCH_SPEED: f32 = 2.0;
pub const PLAYER_JUMP_FORCE: f32 = 8.0;
pub const MOUSE_SENSITIVITY: f32 = 0.002;

// Stamina constants
pub const STAMINA_DRAIN_RATE: f32 = 20.0;
pub const STAMINA_REGEN_RATE: f32 = 15.0;
pub const MAX_STAMINA: f32 = 100.0;

// Weapon constants
pub const WEAPON_DAMAGE: f32 = 25.0;

// This system defines how we update the player's positions when we receive an input
// It must be shared between client and server for prediction to work correctly
pub fn shared_player_movement(
    mut player_query: Query<(
        &mut PlayerPosition,
        &mut PlayerRotation,
        &mut PlayerStamina,
        &ActionState<PlayerAction>,
    )>,
    time: Res<Time>,
) {
    for (mut position, mut rotation, mut stamina, action_state) in player_query.iter_mut() {
        let mut movement = Vec3::ZERO;

        // Handle movement input
        let move_data = action_state.axis_pair(&PlayerAction::Move);
        if move_data != Vec2::ZERO {
            movement.x = move_data.x;
            movement.z = move_data.y;
        }

        // Handle sprint and crouch
        let is_sprinting = action_state.pressed(&PlayerAction::Sprint) && stamina.current > 0.0;
        let is_crouching = action_state.pressed(&PlayerAction::Crouch);

        // Calculate speed based on state
        let speed = if is_sprinting {
            PLAYER_SPRINT_SPEED
        } else if is_crouching {
            PLAYER_CROUCH_SPEED
        } else {
            PLAYER_SPEED
        };

        // Apply movement relative to rotation
        if movement != Vec3::ZERO {
            let forward = rotation.0.mul_vec3(Vec3::NEG_Z);
            let right = rotation.0.mul_vec3(Vec3::X);
            let movement_world =
                (right * movement.x + forward * movement.z) * speed * time.delta_secs();
            position.0 += movement_world;
        }

        // Handle look input (rotation)
        let look_delta = action_state.axis_pair(&PlayerAction::Look);
        if look_delta != Vec2::ZERO {
            if look_delta != Vec2::ZERO {
                // Yaw rotation (around Y axis)
                let yaw_delta = -look_delta.x * MOUSE_SENSITIVITY;
                let yaw_rotation = Quat::from_rotation_y(yaw_delta);
                rotation.0 = yaw_rotation * rotation.0;

                // Pitch is handled by the camera system locally
            }
        }

        // Update stamina
        if is_sprinting {
            stamina.current -= STAMINA_DRAIN_RATE * time.delta_secs();
            stamina.current = stamina.current.max(0.0);
        } else {
            stamina.current += STAMINA_REGEN_RATE * time.delta_secs();
            stamina.current = stamina.current.min(MAX_STAMINA);
        }
    }
}

/// New shared movement system using Avian3D Position and Rotation components
pub fn shared_avian_player_movement(
    mut players: Query<
        (
            &ActionState<PlayerAction>,
            &mut LinearVelocity,
            &mut PlayerStamina,
        ),
        With<PlayerId>,
    >,
    time: Res<Time>,
) {
    for (action_state, mut velocity, mut stamina) in players.iter_mut() {
        let mut direction = Vec3::ZERO;

        let move_vector = action_state.axis_pair(&PlayerAction::Move);
        if move_vector != Vec2::ZERO {
            direction.x = move_vector.x;
            direction.z = -move_vector.y; // Invert Y for 3D movement
        }

        // Apply speed modifiers based on player state
        let mut speed = PLAYER_SPEED;

        if action_state.pressed(&PlayerAction::Sprint) && stamina.current > 0.0 {
            speed = PLAYER_SPRINT_SPEED;
            stamina.current -= STAMINA_DRAIN_RATE * time.delta_secs();
            stamina.current = stamina.current.max(0.0);
        } else {
            stamina.current += STAMINA_REGEN_RATE * time.delta_secs();
            stamina.current = stamina.current.min(stamina.max);
        }

        if action_state.pressed(&PlayerAction::Crouch) {
            speed = PLAYER_CROUCH_SPEED;
        }

        // Apply movement
        if direction.length() > 0.0 {
            direction = direction.normalize();
            velocity.x = direction.x * speed;
            velocity.z = direction.z * speed;
        } else {
            // Apply friction when not moving
            velocity.x *= 0.8;
            velocity.z *= 0.8;
        }

        // Handle jumping
        if action_state.just_pressed(&PlayerAction::Jump) {
            velocity.y = PLAYER_JUMP_FORCE;
        }
    }
}

/// Shared weapon system for handling firing
pub fn shared_weapon_system(
    mut players: Query<(
        &ActionState<PlayerAction>,
        &mut NetworkedWeapon,
        &PlayerId,
        &Position,
        &Rotation,
    )>,
    mut weapon_fire_events: EventWriter<LocalWeaponFireEvent>,
) {
    for (action_state, mut weapon, player_id, position, rotation) in players.iter_mut() {
        // Handle primary fire
        if action_state.just_pressed(&PlayerAction::PrimaryFire)
            && weapon.ammo > 0
            && !weapon.is_reloading
        {
            weapon.ammo -= 1;

            // Get forward direction for firing (convert from Quat to Vec3)
            let forward = rotation.0 * Vec3::NEG_Z;

            weapon_fire_events.write(LocalWeaponFireEvent {
                player_id: player_id.0,
                origin: position.0,
                direction: forward,
                damage: WEAPON_DAMAGE,
            });
        }

        // Handle reload
        if action_state.just_pressed(&PlayerAction::Reload)
            && weapon.ammo < weapon.max_ammo
            && !weapon.is_reloading
        {
            weapon.is_reloading = true;
            // Reload timing would be handled by a separate system
        }
    }
}
