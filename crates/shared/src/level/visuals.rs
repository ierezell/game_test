use avian3d::prelude::{Collider, RigidBody};
use bevy::prelude::*;

use crate::level::generation::{LevelGraph, Zone, ZoneId, ZoneType};
use crate::navigation::NavigationObstacle;
#[derive(Component, Debug)]
pub struct ZoneVisual {
    pub zone_id: ZoneId,
}

#[derive(Component, Debug)]
pub struct DoorVisual {
    pub zone_a: ZoneId,
    pub zone_b: ZoneId,
}

fn spawn_zone_lighting(commands: &mut Commands, zone: &Zone) {
    let light_color = match zone.zone_type {
        ZoneType::Hub => Color::srgb(0.9, 0.9, 0.7),
        ZoneType::Objective => Color::srgb(0.3, 0.9, 0.3),
        ZoneType::Corridor => Color::srgb(0.9, 0.3, 0.3),
        _ => Color::srgb(0.7, 0.8, 0.9),
    };

    commands.spawn((
        PointLight {
            color: light_color,
            intensity: 50000.0,
            range: zone.size.x.max(zone.size.z) * 0.8,
            radius: 1.5,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_translation(zone.position + Vec3::new(0.0, zone.size.y * 0.4, 0.0)),
        Name::new(format!("Light_Zone_{}", zone.id.0)),
    ));
}

pub fn build_zone_visual(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    zone: &Zone,
) {
    let floor_thickness = 0.5;
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(zone.size.x, floor_thickness, zone.size.z))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.01, 0.01, 0.01),
            unlit: false,
            ..default()
        })),
        Transform::from_translation(zone.position + Vec3::new(0.0, -floor_thickness / 2.0, 0.0)),
        RigidBody::Static,
        Collider::cuboid(zone.size.x, floor_thickness, zone.size.z),
        ZoneVisual { zone_id: zone.id },
        Name::new(format!("Floor_Zone_{}", zone.id.0)),
    ));

    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(zone.size.x, 0.3, zone.size.z))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.01, 0.01, 0.01),
            unlit: false,
            ..default()
        })),
        Transform::from_translation(zone.position + Vec3::new(0.0, zone.size.y, 0.0)),
        ZoneVisual { zone_id: zone.id },
        Name::new(format!("Ceiling_Zone_{}", zone.id.0)),
    ));

    let wall_thickness = 0.5;
    let wall_positions = [
        (
            Vec3::new(zone.size.x / 2.0, zone.size.y / 2.0, 0.0),
            Vec3::new(wall_thickness, zone.size.y, zone.size.z),
        ),
        (
            Vec3::new(-zone.size.x / 2.0, zone.size.y / 2.0, 0.0),
            Vec3::new(wall_thickness, zone.size.y, zone.size.z),
        ),
        (
            Vec3::new(0.0, zone.size.y / 2.0, zone.size.z / 2.0),
            Vec3::new(zone.size.x, zone.size.y, wall_thickness),
        ),
        (
            Vec3::new(0.0, zone.size.y / 2.0, -zone.size.z / 2.0),
            Vec3::new(zone.size.x, zone.size.y, wall_thickness),
        ),
    ];

    for (i, (offset, wall_size)) in wall_positions.iter().enumerate() {
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::from_size(*wall_size))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.01, 0.01, 0.01),
                unlit: false,
                ..default()
            })),
            Transform::from_translation(zone.position + *offset),
            RigidBody::Static,
            Collider::cuboid(wall_size.x, wall_size.y, wall_size.z),
            NavigationObstacle,
            ZoneVisual { zone_id: zone.id },
            Name::new(format!("Wall_{}_Zone_{}", i, zone.id.0)),
        ));
    }

    // Add lighting to the zone
    spawn_zone_lighting(commands, zone);

    info!("Built visual for zone {} ({:?})", zone.id.0, zone.zone_type);
}

pub fn build_level_visuals(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    materials: Option<ResMut<Assets<StandardMaterial>>>,
    level_graph: LevelGraph,
) {
    if materials.is_none() {
        warn!("No materials asset provided - level visuals will not be built");
        return;
    }

    let mut materials = materials.unwrap();

    info!(
        "Building visual representation for level with {} zones",
        level_graph.zones.len()
    );

    for zone in level_graph.zones.values() {
        build_zone_visual(&mut commands, &mut meshes, &mut materials, zone);
    }

    info!("Level visuals built successfully");
}
