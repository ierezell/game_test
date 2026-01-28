use bevy::prelude::*;
use shared::components::weapons::HitEvent;

pub struct GunEffectsPlugin;

impl Plugin for GunEffectsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            show_hit_markers,
            cleanup_old_hit_markers,
        ));
    }
}

#[derive(Component)]
struct HitMarker {
    timer: Timer,
}

/// Show visual feedback when shots hit
fn show_hit_markers(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    hit_events: Query<&HitEvent, Added<HitEvent>>,
) {
    for hit_event in hit_events.iter() {
        // Spawn a small sphere at the hit point as visual feedback
        commands.spawn((
            Mesh3d(meshes.add(Sphere::new(0.1))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(1.0, 0.3, 0.0),
                emissive: LinearRgba::rgb(10.0, 3.0, 0.0),
                ..default()
            })),
            Transform::from_translation(hit_event.hit_point),
            HitMarker {
                timer: Timer::from_seconds(0.2, TimerMode::Once),
            },
            Name::new("HitMarker"),
        ));
        
        info!("💥 Hit marker spawned at {:?}", hit_event.hit_point);
    }
}

/// Remove hit markers after their timer expires
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
