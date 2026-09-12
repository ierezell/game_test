use bevy::prelude::*;

use crate::navigation::NavigationObstacle;
use crate::level::generation::{
    LevelGraph, Zone, ZoneType,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SocketSide {
    North,
    South,
    East,
    West,
}

impl SocketSide {
    pub fn as_vec3(self) -> Vec3 {
        match self {
            Self::North => Vec3::new(0.0, 0.0, 1.0),
            Self::South => Vec3::new(0.0, 0.0, -1.0),
            Self::East => Vec3::new(1.0, 0.0, 0.0),
            Self::West => Vec3::new(-1.0, 0.0, 0.0),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RoomSocket {
    pub side: SocketSide,
    pub offset: Vec3,
    pub door_type: crate::level::generation::DoorType,
}

#[derive(Debug, Clone)]
pub struct RoomPrefab {
    pub name: String,
    pub zone_type: ZoneType,
    pub half_size: Vec3,
    pub floor_height: f32,
    pub wall_height: f32,
    pub sockets: Vec<RoomSocket>,
    pub mesh_color: Color,
}

impl RoomPrefab {
    pub fn new_hub() -> Self {
        Self {
            name: "HubRoom".to_string(),
            zone_type: ZoneType::Hub,
            half_size: Vec3::new(20.0, 5.0, 20.0),
            floor_height: 0.5,
            wall_height: 10.0,
            sockets: vec![
                RoomSocket {
                    side: SocketSide::North,
                    offset: Vec3::new(0.0, 5.0, 20.0),
                    door_type: crate::level::generation::DoorType::Normal,
                },
                RoomSocket {
                    side: SocketSide::East,
                    offset: Vec3::new(20.0, 5.0, 0.0),
                    door_type: crate::level::generation::DoorType::Normal,
                },
            ],
            mesh_color: Color::srgb(0.16, 0.17, 0.19),
        }
    }

    pub fn new_corridor() -> Self {
        Self {
            name: "CorridorRoom".to_string(),
            zone_type: ZoneType::Corridor,
            half_size: Vec3::new(8.0, 5.0, 20.0),
            floor_height: 0.5,
            wall_height: 10.0,
            sockets: vec![
                RoomSocket {
                    side: SocketSide::North,
                    offset: Vec3::new(0.0, 5.0, 20.0),
                    door_type: crate::level::generation::DoorType::Normal,
                },
                RoomSocket {
                    side: SocketSide::South,
                    offset: Vec3::new(0.0, 5.0, -20.0),
                    door_type: crate::level::generation::DoorType::Normal,
                },
            ],
            mesh_color: Color::srgb(0.11, 0.09, 0.09),
        }
    }

    pub fn new_objective() -> Self {
        Self {
            name: "ObjectiveRoom".to_string(),
            zone_type: ZoneType::Objective,
            half_size: Vec3::new(20.0, 5.0, 20.0),
            floor_height: 0.5,
            wall_height: 10.0,
            sockets: vec![RoomSocket {
                side: SocketSide::South,
                offset: Vec3::new(0.0, 5.0, -20.0),
                door_type: crate::level::generation::DoorType::Bulkhead,
            }],
            mesh_color: Color::srgb(0.08, 0.12, 0.08),
        }
    }

    pub fn new_industrial() -> Self {
        Self {
            name: "IndustrialRoom".to_string(),
            zone_type: ZoneType::Industrial,
            half_size: Vec3::new(25.0, 5.0, 25.0),
            floor_height: 0.5,
            wall_height: 10.0,
            sockets: vec![
                RoomSocket {
                    side: SocketSide::North,
                    offset: Vec3::new(0.0, 5.0, 25.0),
                    door_type: crate::level::generation::DoorType::Normal,
                },
                RoomSocket {
                    side: SocketSide::East,
                    offset: Vec3::new(25.0, 5.0, 0.0),
                    door_type: crate::level::generation::DoorType::Normal,
                },
            ],
            mesh_color: Color::srgb(0.18, 0.16, 0.13),
        }
    }

    pub fn new_utility() -> Self {
        Self {
            name: "UtilityRoom".to_string(),
            zone_type: ZoneType::Utility,
            half_size: Vec3::new(15.0, 5.0, 15.0),
            floor_height: 0.5,
            wall_height: 10.0,
            sockets: vec![RoomSocket {
                side: SocketSide::North,
                offset: Vec3::new(0.0, 5.0, 15.0),
                door_type: crate::level::generation::DoorType::Normal,
            }],
            mesh_color: Color::srgb(0.12, 0.15, 0.16),
        }
    }

    pub fn new_storage() -> Self {
        Self {
            name: "StorageRoom".to_string(),
            zone_type: ZoneType::Storage,
            half_size: Vec3::new(18.0, 5.0, 18.0),
            floor_height: 0.5,
            wall_height: 10.0,
            sockets: vec![RoomSocket {
                side: SocketSide::North,
                offset: Vec3::new(0.0, 5.0, 18.0),
                door_type: crate::level::generation::DoorType::Normal,
            }],
            mesh_color: Color::srgb(0.13, 0.13, 0.12),
        }
    }

    pub fn for_zone_type(zone_type: ZoneType) -> Self {
        match zone_type {
            ZoneType::Hub => Self::new_hub(),
            ZoneType::Corridor => Self::new_corridor(),
            ZoneType::Objective => Self::new_objective(),
            ZoneType::Industrial => Self::new_industrial(),
            ZoneType::Utility => Self::new_utility(),
            ZoneType::Storage => Self::new_storage(),
        }
    }

    pub fn socket_position(&self, position: Vec3, rotation: Quat) -> Vec<Vec3> {
        self.sockets
            .iter()
            .map(|s| position + rotation * s.offset)
            .collect()
    }
}

#[derive(Component, Debug)]
pub struct RoomPrefabTag {
    pub prefab_name: String,
    pub zone_id: crate::level::generation::ZoneId,
}

#[derive(Component, Debug)]
pub struct RoomSocketTag {
    pub side: SocketSide,
    pub zone_id: crate::level::generation::ZoneId,
}

pub fn spawn_room_prefab(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    prefab: &RoomPrefab,
    zone: &Zone,
) -> Entity {
    let half_size = prefab.half_size;
    let wall_height = prefab.wall_height;
    let floor_height = prefab.floor_height;

    let room_entity = commands
        .spawn((
            RoomPrefabTag {
                prefab_name: prefab.name.clone(),
                zone_id: zone.id,
            },
            Transform::from_translation(zone.position).with_rotation(zone.rotation),
            Name::new(format!("Room_{}_{}", prefab.name, zone.id.0)),
        ))
        .id();

    let floor_pos = zone.position + Vec3::new(0.0, -floor_height / 2.0, 0.0);
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(
            half_size.x * 2.0,
            floor_height,
            half_size.z * 2.0,
        ))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: prefab.mesh_color,
            perceptual_roughness: 0.9,
            ..default()
        })),
        Transform::from_translation(floor_pos).with_rotation(zone.rotation),
        NavigationObstacle,
        Name::new(format!("Floor_Room_{}_{}", prefab.name, zone.id.0)),
    ));

    let ceiling_pos =
        zone.position + Vec3::new(0.0, wall_height + floor_height / 2.0, 0.0);
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(
            half_size.x * 2.0,
            floor_height,
            half_size.z * 2.0,
        ))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: prefab.mesh_color.with_alpha(0.8),
            perceptual_roughness: 0.9,
            ..default()
        })),
        Transform::from_translation(ceiling_pos).with_rotation(zone.rotation),
        Name::new(format!("Ceiling_Room_{}_{}", prefab.name, zone.id.0)),
    ));

    let wall_thickness = 1.0;
    let wall_specs = [
        (
            Vec3::new(half_size.x, wall_height / 2.0, 0.0),
            Vec3::new(wall_thickness, wall_height, half_size.z * 2.0),
            "East",
            SocketSide::East,
        ),
        (
            Vec3::new(-half_size.x, wall_height / 2.0, 0.0),
            Vec3::new(wall_thickness, wall_height, half_size.z * 2.0),
            "West",
            SocketSide::West,
        ),
        (
            Vec3::new(0.0, wall_height / 2.0, half_size.z),
            Vec3::new(half_size.x * 2.0, wall_height, wall_thickness),
            "North",
            SocketSide::North,
        ),
        (
            Vec3::new(0.0, wall_height / 2.0, -half_size.z),
            Vec3::new(half_size.x * 2.0, wall_height, wall_thickness),
            "South",
            SocketSide::South,
        ),
    ];

    for (anchor, size, side_name, socket_side) in wall_specs {
        let has_socket = prefab
            .sockets
            .iter()
            .any(|s| s.side == socket_side);

        if has_socket {
            let socket = prefab
                .sockets
                .iter()
                .find(|s| s.side == socket_side)
                .unwrap();

            let door_width = 6.0;
            let door_center = if side_name == "East" || side_name == "West" {
                socket.offset.z
            } else {
                socket.offset.x
            };

            let opening_half = door_width / 2.0;
            let left_center = door_center - opening_half - wall_thickness / 2.0;
            let right_center = door_center + opening_half + wall_thickness / 2.0;

            let left_length = (half_size.z.max(half_size.x) - left_center.max(0.0)).max(0.0);
            let right_length = (half_size.z.max(half_size.x) + right_center).max(0.0);

            if left_length > 0.5 || right_length > 0.5 {
                let left_pos = zone.position
                    + zone.rotation
                        * (anchor + Vec3::new(
                            if side_name == "North" || side_name == "South" {
                                left_center
                            } else {
                                0.0
                            },
                            0.0,
                            if side_name == "East" || side_name == "West" {
                                left_center
                            } else {
                                0.0
                            },
                        ));
                commands.spawn((
                    Mesh3d(meshes.add(Cuboid::new(
                        if side_name == "North" || side_name == "South" {
                            left_length
                        } else {
                            wall_thickness
                        },
                        wall_height,
                        if side_name == "East" || side_name == "West" {
                            left_length
                        } else {
                            wall_thickness
                        },
                    ))),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color: prefab.mesh_color.with_alpha(0.9),
                        perceptual_roughness: 0.92,
                        ..default()
                    })),
                    Transform::from_translation(left_pos).with_rotation(zone.rotation),
                    NavigationObstacle,
                    Name::new(format!(
                        "Wall_{}_Left_{}_Zone_{}",
                        side_name, prefab.name, zone.id.0
                    )),
                ));
            }

            if right_length > 0.5 {
                let right_pos = zone.position
                    + zone.rotation
                        * (anchor + Vec3::new(
                            if side_name == "North" || side_name == "South" {
                                right_center
                            } else {
                                0.0
                            },
                            0.0,
                            if side_name == "East" || side_name == "West" {
                                right_center
                            } else {
                                0.0
                            },
                        ));
                commands.spawn((
                    Mesh3d(meshes.add(Cuboid::new(
                        if side_name == "North" || side_name == "South" {
                            right_length
                        } else {
                            wall_thickness
                        },
                        wall_height,
                        if side_name == "East" || side_name == "West" {
                            right_length
                        } else {
                            wall_thickness
                        },
                    ))),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color: prefab.mesh_color.with_alpha(0.9),
                        perceptual_roughness: 0.92,
                        ..default()
                    })),
                    Transform::from_translation(right_pos).with_rotation(zone.rotation),
                    NavigationObstacle,
                    Name::new(format!(
                        "Wall_{}_Right_{}_Zone_{}",
                        side_name, prefab.name, zone.id.0
                    )),
                ));
            }
        } else {
            let wall_pos = zone.position + zone.rotation * anchor;
            commands.spawn((
                Mesh3d(meshes.add(Cuboid::new(size.x, size.y, size.z))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: prefab.mesh_color.with_alpha(0.9),
                    perceptual_roughness: 0.92,
                    ..default()
                })),
                Transform::from_translation(wall_pos).with_rotation(zone.rotation),
                NavigationObstacle,
                Name::new(format!(
                    "Wall_{}_{}_Zone_{}",
                    side_name, prefab.name, zone.id.0
                )),
            ));
        }
    }

    for socket in &prefab.sockets {
        let socket_pos = zone.position + zone.rotation * socket.offset;
        commands.spawn((
            Transform::from_translation(socket_pos).with_rotation(zone.rotation),
            RoomSocketTag {
                side: socket.side,
                zone_id: zone.id,
            },
            Name::new(format!(
                "Socket_{:?}_Zone_{}",
                socket.side, zone.id.0
            )),
        ));
    }

    room_entity
}

pub fn build_prefab_level(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    level_graph: &LevelGraph,
) {
    for zone in level_graph.zones.values() {
        let prefab = RoomPrefab::for_zone_type(zone.zone_type);
        spawn_room_prefab(
            &mut commands,
            &mut meshes,
            &mut materials,
            &prefab,
            zone,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hub_prefab_has_correct_sockets() {
        let prefab = RoomPrefab::new_hub();
        assert_eq!(prefab.sockets.len(), 2);
        assert!(prefab.sockets.iter().any(|s| s.side == SocketSide::North));
        assert!(prefab.sockets.iter().any(|s| s.side == SocketSide::East));
    }

    #[test]
    fn corridor_prefab_has_opposite_end_sockets() {
        let prefab = RoomPrefab::new_corridor();
        assert!(prefab.sockets.iter().any(|s| s.side == SocketSide::North));
        assert!(prefab.sockets.iter().any(|s| s.side == SocketSide::South));
    }

    #[test]
    fn objective_prefab_has_bulkhead_socket() {
        let prefab = RoomPrefab::new_objective();
        assert_eq!(prefab.sockets.len(), 1);
        assert_eq!(prefab.sockets[0].side, SocketSide::South);
    }
}
