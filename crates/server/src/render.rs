use bevy::input::mouse::MouseMotion;
use bevy::math::EulerRot;
use bevy::prelude::{
    App, ButtonInput, Camera, Camera3d, Commands, Component, Entity, KeyCode, MouseButton, Name,
    MessageReader, Plugin, Query, Res, Startup, Time, Transform, Update, Vec3, With,
};
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};
use shared::render::{add_npc_visuals, add_player_visuals};

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_camera_if_none_exists);
        app.add_systems(Update, update_free_camera);
        app.add_observer(add_player_visuals);
        app.add_observer(add_npc_visuals);
        app.add_plugins((EguiPlugin::default(), WorldInspectorPlugin::default()));
    }
}

#[derive(Component)]
struct FreeCamera {
    movement_speed: f32,
    fast_multiplier: f32,
    look_sensitivity: f32,
}

impl Default for FreeCamera {
    fn default() -> Self {
        Self {
            movement_speed: 16.0,
            fast_multiplier: 3.0,
            look_sensitivity: 0.0018,
        }
    }
}

fn spawn_camera_if_none_exists(
    mut commands: Commands,
    existing_cameras: Query<Entity, With<Camera3d>>,
) {
    if existing_cameras.is_empty() {
        commands.spawn((
            Camera3d::default(),
            Camera {
                order: 0,
                ..Default::default()
            },
            FreeCamera::default(),
            Transform::from_xyz(-50.0, 50.0, 50.0).looking_at(Vec3::ZERO, Vec3::Y),
            Name::new("RenderCamera"),
        ));
    }
}

fn update_free_camera(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut mouse_motion: MessageReader<MouseMotion>,
    mut camera_query: Query<(&mut Transform, &FreeCamera), With<Camera3d>>,
) {
    let Ok((mut transform, free_camera)) = camera_query.single_mut() else {
        return;
    };

    let mut speed = free_camera.movement_speed;
    if keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight) {
        speed *= free_camera.fast_multiplier;
    }

    let (mut yaw, mut pitch, _) = transform.rotation.to_euler(EulerRot::YXZ);

    if mouse_buttons.pressed(MouseButton::Right) {
        let mut mouse_delta = Vec3::ZERO;
        for event in mouse_motion.read() {
            mouse_delta.x += event.delta.x;
            mouse_delta.y += event.delta.y;
        }

        yaw -= mouse_delta.x * free_camera.look_sensitivity;
        pitch -= mouse_delta.y * free_camera.look_sensitivity;
        pitch = pitch.clamp(-1.54, 1.54);

        transform.rotation = bevy::prelude::Quat::from_euler(EulerRot::YXZ, yaw, pitch, 0.0);
    }

    let forward = transform.forward();
    let mut forward_flat = Vec3::new(forward.x, 0.0, forward.z);
    if forward_flat.length_squared() > 0.0 {
        forward_flat = forward_flat.normalize();
    }
    let right = transform.right();
    let mut right_flat = Vec3::new(right.x, 0.0, right.z);
    if right_flat.length_squared() > 0.0 {
        right_flat = right_flat.normalize();
    }

    let mut direction = Vec3::ZERO;
    if keys.pressed(KeyCode::KeyW) {
        direction += forward_flat;
    }
    if keys.pressed(KeyCode::KeyS) {
        direction -= forward_flat;
    }
    if keys.pressed(KeyCode::KeyD) {
        direction += right_flat;
    }
    if keys.pressed(KeyCode::KeyA) {
        direction -= right_flat;
    }
    if keys.pressed(KeyCode::Space) {
        direction += Vec3::Y;
    }
    if keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight) {
        direction -= Vec3::Y;
    }

    if direction.length_squared() > 0.0 {
        transform.translation += direction.normalize() * speed * time.delta_secs();
    }
}
