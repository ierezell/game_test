use avian3d::prelude::Rotation;
use bevy::prelude::{FixedUpdate, IntoScheduleConfigs, Plugin, Query, With};

use shared::{
    movement::{PhysicsConfig, update_ground_detection, apply_movement},
    camera::FpsCamera,
    protocol::PlayerId,
};

pub struct ServerInputPlugin;

impl Plugin for ServerInputPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_resource::<PhysicsConfig>();
        app.add_systems(FixedUpdate, (
            update_ground_detection,  // Detect ground first
            apply_movement,            // Then apply movement
            update_camera_rotation_server,  // Update rotation from camera
        ).chain());
    }
}

/// Server system: Update entity Rotation from FpsCamera yaw
fn update_camera_rotation_server(
    mut query: Query<(&FpsCamera, &mut Rotation), With<PlayerId>>,
) {
    for (camera, mut rotation) in query.iter_mut() {
        rotation.0 = bevy::prelude::Quat::from_euler(
            bevy::prelude::EulerRot::YXZ,
            camera.yaw,
            0.0,
            0.0,
        );
    }
}
