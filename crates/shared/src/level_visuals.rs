// Level Visual Building - Creates 3D meshes for zones and doors
//
// This module is responsible for spawning the visual representation of the
// procedurally generated level. It creates simple placeholder meshes that can
// later be replaced with high-poly industrial assets.

use avian3d::prelude::{Collider, RigidBody};
use bevy::prelude::*;

use crate::bulkhead_door::{BulkheadDoor, DoorState};
use crate::level_generation::{LevelGraph, Zone, ZoneId, ZoneType};
use crate::navigation::NavigationObstacle;

// ============================================================================
// ZONE VISUAL COMPONENTS
// ============================================================================

/// Marker component for zone visual entities
#[derive(Component, Debug)]
pub struct ZoneVisual {
    pub zone_id: ZoneId,
}

/// Marker component for door visual entities
#[derive(Component, Debug)]
pub struct DoorVisual {
    pub zone_a: ZoneId,
    pub zone_b: ZoneId,
}

// ============================================================================
// GTFO-STYLE ATMOSPHERE
// ============================================================================

/// Setup atmospheric lighting for GTFO aesthetic
/// PURE DARKNESS - flashlight only
pub fn setup_atmosphere(mut commands: Commands) {
    // Pure black - no ambient light at all
    commands.insert_resource(AmbientLight {
        color: Color::BLACK,
        brightness: 0.0,
        ..default()
    });

    commands.insert_resource(ClearColor(Color::BLACK));  // Black background

    info!("Pure darkness atmosphere - flashlight only");
}

/// Spawn emergency lights in a zone
#[allow(dead_code)]
fn spawn_zone_lighting(commands: &mut Commands, zone: &Zone) {
    let light_color = match zone.zone_type {
        ZoneType::Hub => Color::srgb(0.9, 0.9, 0.7),
        ZoneType::Objective => Color::srgb(0.3, 0.9, 0.3),
        ZoneType::Corridor => Color::srgb(0.9, 0.3, 0.3),
        _ => Color::srgb(0.7, 0.8, 0.9),
    };

    // Only enable shadows for Hub zones (important areas)
    let enable_shadows = matches!(zone.zone_type, ZoneType::Hub | ZoneType::Objective);

    commands.spawn((
        PointLight {
            color: light_color,
            intensity: 50000.0,
            range: zone.size.x.max(zone.size.z) * 0.8,
            radius: 1.5,
            shadows_enabled: enable_shadows,
            ..default()
        },
        Transform::from_translation(zone.position + Vec3::new(0.0, zone.size.y * 0.4, 0.0)),
        Name::new(format!("Light_Zone_{}", zone.id.0)),
    ));

    // Add spot light only for objective zones (not corridors)
    if zone.zone_type == ZoneType::Objective {
        commands.spawn((
            SpotLight {
                color: Color::srgb(0.3, 1.0, 0.3),
                intensity: 20000.0,
                range: 12.0,
                radius: 0.8,
                shadows_enabled: false,
                outer_angle: 0.7,
                inner_angle: 0.5,
                ..default()
            },
            Transform::from_translation(zone.position + Vec3::new(0.0, zone.size.y * 0.4, 0.0))
                .looking_at(zone.position, Vec3::Y),
            Name::new(format!("SpotLight_Zone_{}", zone.id.0)),
        ));
    }
}

// ============================================================================
// ZONE BUILDING SYSTEM
// ============================================================================

/// Build visual representation of a zone
///
/// Creates simple box geometry for now - can be replaced with detailed meshes later
pub fn build_zone_visual(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    zone: &Zone,
) {
    // Floor
    let floor_thickness = 0.5;
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(zone.size.x, floor_thickness, zone.size.z))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.01, 0.01, 0.01),  // Nearly black - invisible without light
            unlit: false,  // PBR lighting - only visible when lit by flashlight
            ..default()
        })),
        Transform::from_translation(zone.position + Vec3::new(0.0, -floor_thickness / 2.0, 0.0)),
        RigidBody::Static,
        Collider::cuboid(zone.size.x, floor_thickness, zone.size.z),
        ZoneVisual { zone_id: zone.id },
        Name::new(format!("Floor_Zone_{}", zone.id.0)),
    ));

    // Ceiling (optional, for enclosed feeling)
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(zone.size.x, 0.3, zone.size.z))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.01, 0.01, 0.01),  // Nearly black - invisible without light
            unlit: false,  // PBR lighting - only visible when lit by flashlight
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
                base_color: Color::srgb(0.01, 0.01, 0.01),  // Nearly black - invisible without light
                unlit: false,  // PBR lighting - only visible when lit by flashlight
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

    // Spawn lighting for this zone
    // spawn_zone_lighting(commands, zone); // DISABLED: Only flashlight lighting

    info!("Built visual for zone {} ({:?})", zone.id.0, zone.zone_type);
}

/// Build visual representation of a bulkhead door
///
/// Creates a simple sliding door mesh - can be replaced with detailed model later
pub fn build_door_visual(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    door_position: Vec3,
    door_rotation: Quat,
    zone_a_id: u32,
    zone_b_id: u32,
) -> Entity {
    let door_size = Vec3::new(4.0, 6.0, 0.3); // Wide enough for player passage

    commands
        .spawn((
            Mesh3d(meshes.add(Cuboid::from_size(door_size))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.01, 0.01, 0.01),  // Nearly black - invisible without light
                unlit: false,  // PBR lighting - only visible when lit by flashlight
                ..default()
            })),
            Transform::from_translation(door_position).with_rotation(door_rotation),
            RigidBody::Kinematic, // Kinematic so it can move but isn't affected by physics
            Collider::cuboid(door_size.x, door_size.y, door_size.z),
            BulkheadDoor::new(zone_a_id, zone_b_id),
            DoorState::Closed,
            DoorVisual {
                zone_a: ZoneId(zone_a_id),
                zone_b: ZoneId(zone_b_id),
            },
            Name::new(format!("Door_{}_{}", zone_a_id, zone_b_id)),
        ))
        .id()
}

// ============================================================================
// LEVEL BUILDING SYSTEM
// ============================================================================

/// Build the complete visual representation of the level graph
///
/// This should be called on both server and client after level generation
pub fn build_level_visuals(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    level_graph: LevelGraph,
) {
    info!(
        "Building visual representation for level with {} zones",
        level_graph.zones.len()
    );

    // Setup atmospheric lighting
    setup_atmosphere(commands.reborrow());

    // Build visuals for all zones
    for zone in level_graph.zones.values() {
        build_zone_visual(&mut commands, &mut meshes, &mut materials, zone);
    }

    // Build visuals for all doors/connections
    for connection in level_graph.connections.iter() {
        build_door_visual(
            &mut commands,
            &mut meshes,
            &mut materials,
            connection.door_position,
            connection.door_rotation,
            connection.from_zone.0,
            connection.to_zone.0,
        );
    }

    info!("Level visuals built successfully");
}

// ============================================================================
// PLUGIN
// ============================================================================

pub struct LevelVisualsPlugin;

impl Plugin for LevelVisualsPlugin {
    fn build(&self, _app: &mut App) {
        info!("Level Visuals plugin initialized");
        // Systems are called on-demand when level is generated
    }
}
