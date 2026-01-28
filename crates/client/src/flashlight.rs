use avian3d::prelude::Rotation;
use bevy::prelude::*;
use leafwing_input_manager::prelude::ActionState;
use lightyear::prelude::{Controlled, Predicted};
use shared::camera::FpsCamera;
use shared::components::flashlight::{FlashlightBeam, PlayerFlashlight};
use shared::input::{PlayerAction, PLAYER_CAPSULE_HEIGHT};
use shared::protocol::PlayerId;

pub struct FlashlightPlugin;

impl Plugin for FlashlightPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            handle_flashlight_toggle,
            spawn_flashlight_beam_predicted,
            spawn_flashlight_beam_interpolated,
            update_flashlight_beam_predicted,
            update_flashlight_beam_interpolated,
        ).chain());
    }
}

/// System to handle flashlight toggle input
fn handle_flashlight_toggle(
    mut player_query: Query<
        (&mut PlayerFlashlight, &ActionState<PlayerAction>),
        (With<Predicted>, With<Controlled>, With<PlayerId>),
    >,
) {
    for (mut flashlight, action_state) in player_query.iter_mut() {
        if action_state.just_pressed(&PlayerAction::ToggleFlashlight) {
            flashlight.toggle();
            info!("🔦 Flashlight toggled: {}", if flashlight.is_on { "ON" } else { "OFF" });
        }
    }
}

/// System to spawn flashlight beam for the local predicted player
fn spawn_flashlight_beam_predicted(
    mut commands: Commands,
    player_query: Query<
        (Entity, &PlayerFlashlight),
        (
            With<Predicted>,
            With<Controlled>,
            With<PlayerId>,
            Without<Children>,
        ),
    >,
) {
    for (player_entity, flashlight) in player_query.iter() {
        // Spawn a spotlight as a child of the player
        let beam_entity = commands.spawn((
            FlashlightBeam,
            SpotLight {
                color: Color::srgb(1.0, 0.95, 0.7),
                intensity: flashlight.intensity,
                range: flashlight.range,
                radius: 0.1,
                shadows_enabled: true,
                outer_angle: flashlight.outer_angle,
                inner_angle: flashlight.inner_angle,
                ..default()
            },
            Transform::default(),
            Name::new("LocalPlayerFlashlightBeam"),
        )).id();

        commands.entity(player_entity).add_child(beam_entity);
        info!("🔦 Spawned local flashlight beam for player {:?}", player_entity);
    }
}

/// System to spawn flashlight beam for interpolated (remote) players
fn spawn_flashlight_beam_interpolated(
    mut commands: Commands,
    player_query: Query<
        (Entity, &PlayerFlashlight),
        (
            With<lightyear::prelude::Interpolated>,
            With<PlayerId>,
            Without<Children>,
        ),
    >,
) {
    for (player_entity, flashlight) in player_query.iter() {
        let beam_entity = commands.spawn((
            FlashlightBeam,
            SpotLight {
                color: Color::srgb(1.0, 0.95, 0.7),
                intensity: flashlight.intensity,
                range: flashlight.range,
                radius: 0.1,
                shadows_enabled: true,
                outer_angle: flashlight.outer_angle,
                inner_angle: flashlight.inner_angle,
                ..default()
            },
            Transform::default(),
            Name::new("RemotePlayerFlashlightBeam"),
        )).id();

        commands.entity(player_entity).add_child(beam_entity);
        info!("🔦 Spawned remote flashlight beam for player {:?}", player_entity);
    }
}

/// System to update local player's flashlight beam
fn update_flashlight_beam_predicted(
    mut flashlight_query: Query<&mut SpotLight, With<FlashlightBeam>>,
    player_query: Query<
        (&PlayerFlashlight, &FpsCamera, &Children),
        (
            With<Predicted>,
            With<Controlled>,
            With<PlayerId>,
        ),
    >,
    mut transform_query: Query<&mut Transform>,
) {
    for (flashlight, camera, children) in player_query.iter() {
        for child in children.iter() {
            if let Ok(mut spotlight) = flashlight_query.get_mut(child) {
                // Update intensity based on flashlight state
                spotlight.intensity = if flashlight.is_on { flashlight.intensity } else { 0.0 };
                
                // Update transform to point forward from camera position
                if let Ok(mut transform) = transform_query.get_mut(child) {
                    // Position at camera height (player eye level)
                    let camera_offset = Vec3::new(0.0, PLAYER_CAPSULE_HEIGHT * 0.8, 0.0);
                    transform.translation = camera_offset;
                    
                    // CRITICAL: Only apply PITCH rotation!
                    // The parent player entity already handles YAW rotation
                    // Applying both would cause double-rotation on yaw axis
                    transform.rotation = Quat::from_rotation_x(camera.pitch);
                }
            }
        }
    }
}
/// System to update remote player's flashlight beam
fn update_flashlight_beam_interpolated(
    mut flashlight_query: Query<&mut SpotLight, With<FlashlightBeam>>,
    player_query: Query<
        (&PlayerFlashlight, &Rotation, &Children),
        (
            With<lightyear::prelude::Interpolated>,
            With<PlayerId>,
        ),
    >,
    mut transform_query: Query<&mut Transform>,
) {
    for (flashlight, rotation, children) in player_query.iter() {
        for child in children.iter() {
            if let Ok(mut spotlight) = flashlight_query.get_mut(child) {
                // Update intensity based on replicated flashlight state
                spotlight.intensity = if flashlight.is_on { flashlight.intensity } else { 0.0 };
                
                // Update transform for remote player
                if let Ok(mut transform) = transform_query.get_mut(child) {
                    // Position at player eye level
                    let camera_offset = Vec3::new(0.0, PLAYER_CAPSULE_HEIGHT * 0.8, 0.0);
                    transform.translation = camera_offset;
                    
                    // For interpolated players, use their rotation directly
                    // Since interpolated players don't have FpsCamera, we use the entity's rotation
                    transform.rotation = rotation.0;
                }
            }
        }
    }
}