use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;
use lightyear::prelude::{Controlled, ControlledBy, Interpolated, Predicted};
use shared::components::flashlight::PlayerFlashlight;
use shared::inputs::{PLAYER_CAPSULE_HEIGHT, ToggleFlashlight};
use shared::protocol::PlayerId;

pub struct ClientFlashlightPlugin;

#[derive(Component)]
struct FlashlightBeam;

#[derive(Component)]
struct HasFlashlightBeam;

impl Plugin for ClientFlashlightPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(handle_flashlight_toggle);
        app.add_systems(Update, (spawn_flashlight_beam, update_flashlight_beam));
    }
}

fn handle_flashlight_toggle(
    trigger: On<Start<ToggleFlashlight>>,
    mut flashlight_query: Query<&mut PlayerFlashlight, (With<Predicted>, Without<Interpolated>)>,
) {
    if let Ok(mut flashlight) = flashlight_query.get_mut(trigger.context) {
        flashlight.toggle();
        info!(
            "🔦 Flashlight toggled: {}",
            if flashlight.is_on { "ON" } else { "OFF" }
        );
    }
}

fn spawn_flashlight_beam(
    mut commands: Commands,
    player_query: Query<
        (Entity, &PlayerFlashlight, Has<Controlled>),
        (
            Or<(With<Predicted>, With<Interpolated>, With<ControlledBy>)>,
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
                    shadow_maps_enabled: is_controlled,
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
            Or<(With<Predicted>, With<Interpolated>, With<ControlledBy>)>,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flashlight_toggle_sets_correct_values() {
        let mut flashlight = PlayerFlashlight::new();
        assert!(flashlight.is_on);

        flashlight.toggle();
        assert!(!flashlight.is_on);
    }

    #[test]
    fn beam_intensity_matches_flashlight_state() {
        let mut flashlight = PlayerFlashlight::new();
        flashlight.is_on = true;
        let intensity_on = if flashlight.is_on {
            flashlight.intensity
        } else {
            0.0
        };

        flashlight.is_on = false;
        let intensity_off = if flashlight.is_on {
            flashlight.intensity
        } else {
            0.0
        };

        assert_eq!(intensity_on, 1400000.0);
        assert_eq!(intensity_off, 0.0);
    }

    #[test]
    fn spawn_flashlight_beam_requires_player_flashlight() {
        use lightyear::prelude::{PeerId, Predicted};
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, spawn_flashlight_beam);

        // Spawn a player entity with PlayerFlashlight and PlayerId and Predicted
        let _player = app
            .world_mut()
            .spawn((
                PlayerFlashlight::new(),
                PlayerId(PeerId::Netcode(1)),
                Predicted,
                Transform::default(),
                bevy::ecs::name::Name::new("Player"),
            ))
            .id();

        // Run the system - it should spawn a flashlight beam
        app.update();
        app.update();

        // Verify a SpotLight was spawned as a child
        let mut child_query = app.world_mut().query::<&SpotLight>();
        let spot_light_count = child_query.iter(app.world()).count();
        assert!(
            spot_light_count >= 1,
            "Expected at least one SpotLight to be spawned, got {}",
            spot_light_count
        );

        // Verify the child has the FlashlightBeam marker
        let mut beam_query = app.world_mut().query::<&FlashlightBeam>();
        let beam_count = beam_query.iter(app.world()).count();
        assert!(
            beam_count >= 1,
            "Expected at least one FlashlightBeam marker, got {}",
            beam_count
        );
    }

    #[test]
    fn update_flashlight_beam_updates_intensity_on_change() {
        use lightyear::prelude::{PeerId, Predicted};
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, spawn_flashlight_beam);
        app.add_systems(Update, update_flashlight_beam);

        // Spawn a player entity
        let player = app
            .world_mut()
            .spawn((
                PlayerFlashlight::new(),
                PlayerId(PeerId::Netcode(1)),
                Predicted,
                Transform::default(),
                bevy::ecs::name::Name::new("Player"),
            ))
            .id();

        // Initial update to spawn the beam
        app.update();

        // Turn off the flashlight
        {
            let mut flashlight = app.world_mut().get_mut::<PlayerFlashlight>(player).unwrap();
            flashlight.is_on = false;
        }
        app.update();

        // Verify the spotlight intensity is 0
        let mut spotlight_q = app.world_mut().query::<&SpotLight>();
        let all_zero = spotlight_q
            .iter(app.world())
            .all(|light| light.intensity == 0.0);
        assert!(
            all_zero,
            "All spotlights should have 0 intensity when flashlight is off"
        );
    }
}
