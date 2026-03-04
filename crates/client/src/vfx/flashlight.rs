use bevy::prelude::*;
use leafwing_input_manager::prelude::ActionState;
use lightyear::prelude::{Controlled, Interpolated, Predicted};
use shared::components::flashlight::PlayerFlashlight;
use shared::inputs::input::{PLAYER_CAPSULE_HEIGHT, PlayerAction};
use shared::protocol::PlayerId;

pub struct ClientFlashlightPlugin;

#[derive(Component)]
struct FlashlightBeam;

#[derive(Component)]
struct HasFlashlightBeam;

impl Plugin for ClientFlashlightPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                handle_flashlight_toggle,
                spawn_flashlight_beam,
                update_flashlight_beam,
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
        if action_state.disabled() {
            continue;
        }

        if action_state.just_pressed(&PlayerAction::ToggleFlashlight) {
            flashlight.toggle();
            info!(
                "🔦 Flashlight toggled: {}",
                if flashlight.is_on { "ON" } else { "OFF" }
            );
        }
    }
}

fn spawn_flashlight_beam(
    mut commands: Commands,
    player_query: Query<
        (Entity, &PlayerFlashlight, Has<Controlled>),
        (
            Or<(With<Predicted>, With<Interpolated>)>,
            With<PlayerId>,
            Without<HasFlashlightBeam>,
        ),
    >,
) {
    for (player_entity, flashlight, is_controlled) in player_query.iter() {
        let beam_intensity = if flashlight.is_on {
            flashlight.intensity
        } else {
            0.0
        };
        let beam_name = if is_controlled {
            "LocalPlayerFlashlightBeam"
        } else {
            "RemotePlayerFlashlightBeam"
        };

        let beam_entity = commands
            .spawn((
                FlashlightBeam,
                SpotLight {
                    color: Color::srgb(1.0, 0.95, 0.7),
                    intensity: beam_intensity,
                    range: flashlight.range,
                    radius: 0.1,
                    shadows_enabled: is_controlled,
                    outer_angle: flashlight.outer_angle,
                    inner_angle: flashlight.inner_angle,
                    ..default()
                },
                Transform::from_translation(Vec3::new(0.0, PLAYER_CAPSULE_HEIGHT * 0.8, 0.2)),
                Name::new(beam_name),
            ))
            .id();

        commands
            .entity(player_entity)
            .add_child(beam_entity)
            .insert(HasFlashlightBeam);
        info!("🔦 Spawned flashlight beam for player {:?}", player_entity);
    }
}

fn update_flashlight_beam(
    mut flashlight_query: Query<&mut SpotLight, With<FlashlightBeam>>,
    player_query: Query<
        (&PlayerFlashlight, &Children),
        (
            Or<(With<Predicted>, With<Interpolated>)>,
            With<PlayerId>,
            Changed<PlayerFlashlight>,
        ),
    >,
) {
    for (flashlight, children) in player_query.iter() {
        for child in children.iter() {
            if let Ok(mut spotlight) = flashlight_query.get_mut(child) {
                spotlight.intensity = if flashlight.is_on {
                    flashlight.intensity
                } else {
                    0.0
                };
                spotlight.range = flashlight.range;
                spotlight.outer_angle = flashlight.outer_angle;
                spotlight.inner_angle = flashlight.inner_angle;
            }
        }
    }
}
