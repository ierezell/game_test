use avian3d::prelude::Position;
use bevy::prelude::*;
use shared::components::weapons::HitEvent;

pub struct GunEffectsPlugin;

impl Plugin for GunEffectsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (show_hit_markers, cleanup_old_hit_markers));
    }
}

#[derive(Component)]
struct HitMarker {
    timer: Timer,
}

fn show_hit_markers(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    hit_events: Query<&HitEvent, Added<HitEvent>>,
    shooter_positions: Query<&Position>,
) {
    for hit_event in hit_events.iter() {
        // Offset markers slightly toward the shooter so they stay visible on impact surfaces.
        let marker_position = shooter_positions
            .get(hit_event.shooter)
            .map(|pos| {
                let shot_direction = (hit_event.hit_point - pos.0).normalize_or_zero();
                if shot_direction.length_squared() > 0.0 {
                    hit_event.hit_point - (shot_direction * 0.08)
                } else {
                    hit_event.hit_point
                }
            })
            .unwrap_or(hit_event.hit_point);

        commands.spawn((
            Mesh3d(meshes.add(Sphere::new(0.14))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.2, 1.0, 0.9),
                emissive: LinearRgba::rgb(2.0, 10.0, 9.0),
                ..default()
            })),
            Transform::from_translation(marker_position),
            HitMarker {
                timer: Timer::from_seconds(0.2, TimerMode::Once),
            },
            Name::new("HitMarker"),
        ));

        info!("💥 Hit marker spawned at {:?}", hit_event.hit_point);
    }
}

fn cleanup_old_hit_markers(
    mut commands: Commands,
    mut markers: Query<(Entity, &mut HitMarker)>,
    time: Res<Time>,
) {
    for (entity, mut marker) in markers.iter_mut() {
        marker.timer.tick(time.delta());
        if marker.timer.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}
