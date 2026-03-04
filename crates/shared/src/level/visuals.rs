use bevy::prelude::*;

use crate::level::generation::{
    LevelGraph, WALL_SIDE_EAST, WALL_SIDE_NORTH, WALL_SIDE_SOUTH, WALL_SIDE_WEST, WALL_THICKNESS,
    Zone, ZoneId, ZoneType, collect_zone_wall_segments,
};

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
            intensity: 42000.0,
            range: zone.size.x.max(zone.size.z) * 0.8,
            radius: 1.5,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_translation(zone.position + Vec3::new(0.0, zone.size.y * 0.4, 0.0)),
        Name::new(format!("Light_Zone_{}", zone.id.0)),
    ));

    let use_haze = matches!(
        zone.zone_type,
        ZoneType::Utility | ZoneType::Industrial | ZoneType::Storage | ZoneType::Corridor
    ) && zone.id.0.is_multiple_of(2);

    if use_haze {
        let corner_offsets = [
            Vec3::new(zone.size.x * 0.35, zone.size.y * 0.22, zone.size.z * 0.35),
            Vec3::new(-zone.size.x * 0.35, zone.size.y * 0.22, -zone.size.z * 0.35),
        ];

        for (index, offset) in corner_offsets.iter().enumerate() {
            let light_position = zone.position + zone.rotation * *offset;
            commands.spawn((
                PointLight {
                    color: Color::srgb(0.45, 0.55, 0.65),
                    intensity: 3800.0,
                    range: 12.0,
                    radius: 0.35,
                    shadows_enabled: false,
                    ..default()
                },
                Transform::from_translation(light_position),
                Name::new(format!("Haze_{}_Zone_{}", index, zone.id.0)),
            ));
        }
    }
}

fn zone_surface_material(zone_type: ZoneType) -> StandardMaterial {
    match zone_type {
        ZoneType::Hub => StandardMaterial {
            base_color: Color::srgb(0.16, 0.17, 0.19),
            perceptual_roughness: 0.92,
            metallic: 0.05,
            ..default()
        },
        ZoneType::Corridor => StandardMaterial {
            base_color: Color::srgb(0.11, 0.09, 0.09),
            perceptual_roughness: 0.96,
            metallic: 0.02,
            ..default()
        },
        ZoneType::Utility => StandardMaterial {
            base_color: Color::srgb(0.12, 0.15, 0.16),
            perceptual_roughness: 0.9,
            metallic: 0.08,
            ..default()
        },
        ZoneType::Industrial => StandardMaterial {
            base_color: Color::srgb(0.18, 0.16, 0.13),
            perceptual_roughness: 0.88,
            metallic: 0.12,
            ..default()
        },
        ZoneType::Objective => StandardMaterial {
            base_color: Color::srgb(0.08, 0.12, 0.08),
            perceptual_roughness: 0.94,
            metallic: 0.03,
            ..default()
        },
        ZoneType::Storage => StandardMaterial {
            base_color: Color::srgb(0.13, 0.13, 0.12),
            perceptual_roughness: 0.95,
            metallic: 0.04,
            ..default()
        },
    }
}

pub fn build_zone_visual(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    zone: &Zone,
    level_graph: &LevelGraph,
) {
    let floor_thickness = 0.5;
    let floor_position = zone.position + Vec3::new(0.0, -floor_thickness / 2.0, 0.0);
    let base_material = zone_surface_material(zone.zone_type);
    let wall_material = StandardMaterial {
        base_color: base_material.base_color.with_alpha(1.0),
        perceptual_roughness: (base_material.perceptual_roughness + 0.05).min(1.0),
        metallic: base_material.metallic,
        ..default()
    };
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(zone.size.x, floor_thickness, zone.size.z))),
        MeshMaterial3d(materials.add(base_material.clone())),
        Transform::from_translation(floor_position).with_rotation(zone.rotation),
        ZoneVisual { zone_id: zone.id },
        Name::new(format!("Floor_Zone_{}", zone.id.0)),
    ));

    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(zone.size.x, 0.3, zone.size.z))),
        MeshMaterial3d(materials.add(base_material)),
        Transform::from_translation(zone.position + Vec3::new(0.0, zone.size.y, 0.0))
            .with_rotation(zone.rotation),
        ZoneVisual { zone_id: zone.id },
        Name::new(format!("Ceiling_Zone_{}", zone.id.0)),
    ));

    let wall_segments = collect_zone_wall_segments(zone, level_graph);
    let half_x = zone.size.x * 0.5;
    let half_z = zone.size.z * 0.5;

    let side_specs = [
        (
            WALL_SIDE_EAST,
            Vec3::new(half_x, zone.size.y * 0.5, 0.0),
            true,
            "East",
        ),
        (
            WALL_SIDE_WEST,
            Vec3::new(-half_x, zone.size.y * 0.5, 0.0),
            true,
            "West",
        ),
        (
            WALL_SIDE_NORTH,
            Vec3::new(0.0, zone.size.y * 0.5, half_z),
            false,
            "North",
        ),
        (
            WALL_SIDE_SOUTH,
            Vec3::new(0.0, zone.size.y * 0.5, -half_z),
            false,
            "South",
        ),
    ];

    for (side_index, wall_anchor, span_on_z, side_name) in side_specs {
        for (segment_index, (segment_center, segment_length)) in
            wall_segments[side_index].iter().enumerate()
        {
            let local_offset = if span_on_z {
                wall_anchor + Vec3::new(0.0, 0.0, *segment_center)
            } else {
                wall_anchor + Vec3::new(*segment_center, 0.0, 0.0)
            };

            let wall_size = if span_on_z {
                Vec3::new(WALL_THICKNESS, zone.size.y, *segment_length)
            } else {
                Vec3::new(*segment_length, zone.size.y, WALL_THICKNESS)
            };

            let wall_position = zone.position + zone.rotation * local_offset;
            commands.spawn((
                Mesh3d(meshes.add(Cuboid::from_size(wall_size))),
                MeshMaterial3d(materials.add(wall_material.clone())),
                Transform::from_translation(wall_position).with_rotation(zone.rotation),
                ZoneVisual { zone_id: zone.id },
                Name::new(format!(
                    "Wall_{}_{}_Zone_{}",
                    side_name, segment_index, zone.id.0
                )),
            ));
        }
    }

    // Add lighting to the zone
    spawn_zone_lighting(commands, zone);

    info!("Built visual for zone {} ({:?})", zone.id.0, zone.zone_type);
}

pub fn build_level_visuals(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    materials: Option<ResMut<Assets<StandardMaterial>>>,
    level_graph: &LevelGraph,
) {
    let Some(mut materials) = materials else {
        warn!("No materials asset provided - level visuals will not be built");
        return;
    };

    info!(
        "Building visual representation for level with {} zones",
        level_graph.zones.len()
    );

    commands.spawn((
        AmbientLight {
            color: Color::srgb(0.16, 0.2, 0.24),
            brightness: 24.0,
            ..default()
        },
        Name::new("ProceduralAmbientLight"),
    ));

    for zone in level_graph.zones.values() {
        build_zone_visual(
            &mut commands,
            &mut meshes,
            &mut materials,
            zone,
            level_graph,
        );
    }

    info!("Level visuals built successfully");
}
