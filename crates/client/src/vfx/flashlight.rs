use avian3d::prelude::Rotation;
use bevy::prelude::*;
use leafwing_input_manager::prelude::ActionState;
use lightyear::prelude::{Controlled, Predicted};
use shared::components::flashlight::PlayerFlashlight;
use shared::inputs::input::{PLAYER_CAPSULE_HEIGHT, PlayerAction};
use shared::protocol::PlayerId;

pub struct ClientFlashlightPlugin;

impl Plugin for ClientFlashlightPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                handle_flashlight_toggle,
                spawn_flashlight_beam_predicted,
                spawn_flashlight_beam_interpolated,
                update_flashlight_beam_predicted,
                update_flashlight_beam_interpolated,
            )
                .chain(),
        );
    }
}

fn handle_flashlight_toggle(
    mut player_query: Query<
        (&mut PlayerFlashlight, &ActionState<PlayerAction>),
        (With<Predicted>, With<Controlled>, With<PlayerId>),
    >,
) {
    for (mut flashlight, action_state) in player_query.iter_mut() {
        if action_state.just_pressed(&PlayerAction::ToggleFlashlight) {
            flashlight.toggle();
            info!(
                "🔦 Flashlight toggled: {}",
                if flashlight.is_on { "ON" } else { "OFF" }
            );
        }
    }
}

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
        let beam_entity = commands
            .spawn((
                PlayerFlashlight::new(),
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
            ))
            .id();

        commands.entity(player_entity).add_child(beam_entity);
        info!(
            "🔦 Spawned local flashlight beam for player {:?}",
            player_entity
        );
    }
}

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
        let beam_entity = commands
            .spawn((
                PlayerFlashlight::new(),
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
            ))
            .id();

        commands.entity(player_entity).add_child(beam_entity);
        info!(
            "🔦 Spawned remote flashlight beam for player {:?}",
            player_entity
        );
    }
}

fn update_flashlight_beam_predicted(
    mut flashlight_query: Query<&mut SpotLight, With<PlayerFlashlight>>,
    player_query: Query<
        (&PlayerFlashlight, &Rotation, &Children),
        (With<Predicted>, With<Controlled>, With<PlayerId>),
    >,
    mut transform_query: Query<&mut Transform>,
) {
    for (flashlight, player_rotation, children) in player_query.iter() {
        for child in children.iter() {
            if let Ok(mut spotlight) = flashlight_query.get_mut(child) {
                spotlight.intensity = if flashlight.is_on {
                    flashlight.intensity
                } else {
                    0.0
                };

                if let Ok(mut transform) = transform_query.get_mut(child) {
                    let camera_offset = Vec3::new(0.0, PLAYER_CAPSULE_HEIGHT * 0.8, 0.0);
                    transform.translation = camera_offset;
                    transform.rotation = Quat::from_rotation_x(player_rotation.x);
                }
            }
        }
    }
}

fn update_flashlight_beam_interpolated(
    mut flashlight_query: Query<&mut SpotLight, With<PlayerFlashlight>>,
    player_query: Query<
        (&PlayerFlashlight, &Rotation, &Children),
        (With<lightyear::prelude::Interpolated>, With<PlayerId>),
    >,
    mut transform_query: Query<&mut Transform>,
) {
    for (flashlight, rotation, children) in player_query.iter() {
        for child in children.iter() {
            if let Ok(mut spotlight) = flashlight_query.get_mut(child) {
                spotlight.intensity = if flashlight.is_on {
                    flashlight.intensity
                } else {
                    0.0
                };

                if let Ok(mut transform) = transform_query.get_mut(child) {
                    let camera_offset = Vec3::new(0.0, PLAYER_CAPSULE_HEIGHT * 0.8, 0.0);
                    transform.translation = camera_offset;
                    transform.rotation = rotation.0;
                }
            }
        }
    }
}
