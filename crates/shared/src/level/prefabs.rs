use avian3d::prelude::{Collider, Position, RigidBody, Rotation};
use bevy::prelude::*;
use rand::rngs::StdRng;
use rand::{Rng, RngExt, SeedableRng};
use std::sync::LazyLock;

use crate::level::generation::{
    DoorType, LevelGraph, Zone, ZoneConnection, ZoneId, ZoneType, WALL_THICKNESS,
};
use crate::navigation::NavigationObstacle;

const DOOR_OPENING_WIDTH: f32 = 6.0;
const DOOR_EDGE_MARGIN: f32 = 1.0;
const FLOOR_THICKNESS: f32 = 0.5;

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

    pub fn opposite(self) -> Self {
        match self {
            Self::North => Self::South,
            Self::South => Self::North,
            Self::East => Self::West,
            Self::West => Self::East,
        }
    }

    pub fn from_local_direction(local_direction: Vec3) -> Self {
        if local_direction.x.abs() >= local_direction.z.abs() {
            if local_direction.x >= 0.0 {
                Self::East
            } else {
                Self::West
            }
        } else if local_direction.z >= 0.0 {
            Self::North
        } else {
            Self::South
        }
    }
}

#[derive(Debug, Clone)]
pub struct RoomSocket {
    pub side: SocketSide,
    pub offset: Vec3,
    pub door_type: DoorType,
}

impl RoomSocket {
    pub fn edge_local(self) -> Vec3 {
        Vec3::new(self.offset.x, 0.0, self.offset.z)
    }
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
    pub fn half_extent_local(&self, side: SocketSide) -> Vec3 {
        match side {
            SocketSide::East => Vec3::new(self.half_size.x, 0.0, 0.0),
            SocketSide::West => Vec3::new(-self.half_size.x, 0.0, 0.0),
            SocketSide::North => Vec3::new(0.0, 0.0, self.half_size.z),
            SocketSide::South => Vec3::new(0.0, 0.0, -self.half_size.z),
        }
    }

    pub fn wall_half_span(&self, side: SocketSide) -> f32 {
        match side {
            SocketSide::East | SocketSide::West => self.half_size.z,
            SocketSide::North | SocketSide::South => self.half_size.x,
        }
    }

    pub fn socket_position(&self, position: Vec3, rotation: Quat) -> Vec<Vec3> {
        self.sockets
            .iter()
            .map(|s| position + rotation * s.offset)
            .collect()
    }

    pub fn has_socket(&self, side: SocketSide) -> bool {
        self.sockets.iter().any(|s| s.side == side)
    }
}


fn base_socket_offset(side: SocketSide, half_size: Vec3) -> Vec3 {
    Vec3::new(
        match side {
            SocketSide::East => half_size.x,
            SocketSide::West => -half_size.x,
            SocketSide::North | SocketSide::South => 0.0,
        },
        half_size.y,
        match side {
            SocketSide::North => half_size.z,
            SocketSide::South => -half_size.z,
            SocketSide::East | SocketSide::West => 0.0,
        },
    )
}

fn base_socket(side: SocketSide, half_size: Vec3, door_type: DoorType) -> RoomSocket {
    RoomSocket {
        side,
        offset: base_socket_offset(side, half_size),
        door_type,
    }
}

fn shifted_socket(side: SocketSide, half_size: Vec3, shift: f32, door_type: DoorType) -> RoomSocket {
    let mut offset = base_socket_offset(side, half_size);
    match side {
        SocketSide::East | SocketSide::West => offset.z += shift,
        SocketSide::North | SocketSide::South => offset.x += shift,
    }
    RoomSocket {
        side,
        offset,
        door_type,
    }
}

pub fn new_hub_variants() -> Vec<RoomPrefab> {
    vec![
        RoomPrefab {
            name: "HubRoom_A".to_string(),
            zone_type: ZoneType::Hub,
            half_size: Vec3::new(30.0, 5.0, 30.0),
            floor_height: 0.5,
            wall_height: 10.0,
            sockets: vec![
                base_socket(SocketSide::North, Vec3::new(30.0, 5.0, 30.0), DoorType::Normal),
                base_socket(SocketSide::East, Vec3::new(30.0, 5.0, 30.0), DoorType::Normal),
            ],
            mesh_color: Color::srgb(0.16, 0.17, 0.19),
        },
        RoomPrefab {
            name: "HubRoom_B".to_string(),
            zone_type: ZoneType::Hub,
            half_size: Vec3::new(33.0, 5.0, 28.0),
            floor_height: 0.5,
            wall_height: 10.0,
            sockets: vec![
                shifted_socket(SocketSide::North, Vec3::new(33.0, 5.0, 28.0), -3.0, DoorType::Normal),
                shifted_socket(SocketSide::East, Vec3::new(33.0, 5.0, 28.0), 3.0, DoorType::Normal),
                base_socket(SocketSide::South, Vec3::new(33.0, 5.0, 28.0), DoorType::Normal),
            ],
            mesh_color: Color::srgb(0.18, 0.17, 0.20),
        },
        RoomPrefab {
            name: "HubRoom_C".to_string(),
            zone_type: ZoneType::Hub,
            half_size: Vec3::new(28.0, 5.0, 33.0),
            floor_height: 0.5,
            wall_height: 10.0,
            sockets: vec![
                base_socket(SocketSide::North, Vec3::new(28.0, 5.0, 33.0), DoorType::Normal),
                base_socket(SocketSide::East, Vec3::new(28.0, 5.0, 33.0), DoorType::Normal),
                base_socket(SocketSide::West, Vec3::new(28.0, 5.0, 33.0), DoorType::Normal),
            ],
            mesh_color: Color::srgb(0.15, 0.18, 0.17),
        },
        RoomPrefab {
            name: "HubRoom_D".to_string(),
            zone_type: ZoneType::Hub,
            half_size: Vec3::new(36.0, 5.0, 36.0),
            floor_height: 0.5,
            wall_height: 10.0,
            sockets: vec![
                base_socket(SocketSide::North, Vec3::new(36.0, 5.0, 36.0), DoorType::Normal),
                base_socket(SocketSide::South, Vec3::new(36.0, 5.0, 36.0), DoorType::Normal),
                base_socket(SocketSide::East, Vec3::new(36.0, 5.0, 36.0), DoorType::Normal),
                base_socket(SocketSide::West, Vec3::new(36.0, 5.0, 36.0), DoorType::Normal),
            ],
            mesh_color: Color::srgb(0.14, 0.16, 0.20),
        },
    ]
}

pub fn new_corridor_variants() -> Vec<RoomPrefab> {
    vec![
        RoomPrefab {
            name: "CorridorRoom_A".to_string(),
            zone_type: ZoneType::Corridor,
            half_size: Vec3::new(5.0, 5.0, 25.0),
            floor_height: 0.5,
            wall_height: 10.0,
            sockets: vec![
                base_socket(SocketSide::North, Vec3::new(5.0, 5.0, 25.0), DoorType::Normal),
                base_socket(SocketSide::South, Vec3::new(5.0, 5.0, 25.0), DoorType::Normal),
            ],
            mesh_color: Color::srgb(0.11, 0.09, 0.09),
        },
        RoomPrefab {
            name: "CorridorRoom_B".to_string(),
            zone_type: ZoneType::Corridor,
            half_size: Vec3::new(6.0, 5.0, 23.0),
            floor_height: 0.5,
            wall_height: 10.0,
            sockets: vec![
                shifted_socket(SocketSide::North, Vec3::new(6.0, 5.0, 23.0), 2.0, DoorType::Normal),
                shifted_socket(SocketSide::South, Vec3::new(6.0, 5.0, 23.0), -2.0, DoorType::Normal),
            ],
            mesh_color: Color::srgb(0.13, 0.10, 0.10),
        },
        RoomPrefab {
            name: "CorridorRoom_C".to_string(),
            zone_type: ZoneType::Corridor,
            half_size: Vec3::new(7.0, 5.0, 27.0),
            floor_height: 0.5,
            wall_height: 10.0,
            sockets: vec![
                base_socket(SocketSide::North, Vec3::new(7.0, 5.0, 27.0), DoorType::Normal),
                base_socket(SocketSide::South, Vec3::new(7.0, 5.0, 27.0), DoorType::Normal),
            ],
            mesh_color: Color::srgb(0.12, 0.11, 0.09),
        },
        RoomPrefab {
            name: "CorridorRoom_D".to_string(),
            zone_type: ZoneType::Corridor,
            half_size: Vec3::new(8.0, 5.0, 26.0),
            floor_height: 0.5,
            wall_height: 10.0,
            sockets: vec![
                base_socket(SocketSide::North, Vec3::new(8.0, 5.0, 26.0), DoorType::Normal),
                base_socket(SocketSide::South, Vec3::new(8.0, 5.0, 26.0), DoorType::Normal),
            ],
            mesh_color: Color::srgb(0.10, 0.08, 0.08),
        },
    ]
}

pub fn new_industrial_variants() -> Vec<RoomPrefab> {
    vec![
        RoomPrefab {
            name: "IndustrialRoom_A".to_string(),
            zone_type: ZoneType::Industrial,
            half_size: Vec3::new(35.0, 5.0, 35.0),
            floor_height: 0.5,
            wall_height: 10.0,
            sockets: vec![
                base_socket(SocketSide::North, Vec3::new(35.0, 5.0, 35.0), DoorType::Normal),
                base_socket(SocketSide::East, Vec3::new(35.0, 5.0, 35.0), DoorType::Normal),
            ],
            mesh_color: Color::srgb(0.18, 0.16, 0.13),
        },
        RoomPrefab {
            name: "IndustrialRoom_B".to_string(),
            zone_type: ZoneType::Industrial,
            half_size: Vec3::new(38.0, 5.0, 32.0),
            floor_height: 0.5,
            wall_height: 10.0,
            sockets: vec![
                shifted_socket(SocketSide::North, Vec3::new(38.0, 5.0, 32.0), -4.0, DoorType::Normal),
                base_socket(SocketSide::East, Vec3::new(38.0, 5.0, 32.0), DoorType::Normal),
                base_socket(SocketSide::South, Vec3::new(38.0, 5.0, 32.0), DoorType::Alarm),
            ],
            mesh_color: Color::srgb(0.20, 0.15, 0.11),
        },
        RoomPrefab {
            name: "IndustrialRoom_C".to_string(),
            zone_type: ZoneType::Industrial,
            half_size: Vec3::new(33.0, 5.0, 38.0),
            floor_height: 0.5,
            wall_height: 10.0,
            sockets: vec![
                base_socket(SocketSide::North, Vec3::new(33.0, 5.0, 38.0), DoorType::Normal),
                base_socket(SocketSide::South, Vec3::new(33.0, 5.0, 38.0), DoorType::Normal),
                base_socket(SocketSide::West, Vec3::new(33.0, 5.0, 38.0), DoorType::Normal),
            ],
            mesh_color: Color::srgb(0.17, 0.17, 0.14),
        },
        RoomPrefab {
            name: "IndustrialRoom_D".to_string(),
            zone_type: ZoneType::Industrial,
            half_size: Vec3::new(40.0, 5.0, 40.0),
            floor_height: 0.5,
            wall_height: 10.0,
            sockets: vec![
                base_socket(SocketSide::North, Vec3::new(40.0, 5.0, 40.0), DoorType::Normal),
                base_socket(SocketSide::East, Vec3::new(40.0, 5.0, 40.0), DoorType::Normal),
                base_socket(SocketSide::South, Vec3::new(40.0, 5.0, 40.0), DoorType::Normal),
            ],
            mesh_color: Color::srgb(0.16, 0.14, 0.12),
        },
        RoomPrefab {
            name: "IndustrialRoom_E".to_string(),
            zone_type: ZoneType::Industrial,
            half_size: Vec3::new(37.0, 5.0, 37.0),
            floor_height: 0.5,
            wall_height: 10.0,
            sockets: vec![
                shifted_socket(SocketSide::North, Vec3::new(37.0, 5.0, 37.0), 5.0, DoorType::Normal),
                shifted_socket(SocketSide::East, Vec3::new(37.0, 5.0, 37.0), 5.0, DoorType::Normal),
                shifted_socket(SocketSide::West, Vec3::new(37.0, 5.0, 37.0), -5.0, DoorType::Normal),
            ],
            mesh_color: Color::srgb(0.19, 0.18, 0.15),
        },
    ]
}

pub fn new_objective_variants() -> Vec<RoomPrefab> {
    vec![
        RoomPrefab {
            name: "ObjectiveRoom_A".to_string(),
            zone_type: ZoneType::Objective,
            half_size: Vec3::new(20.0, 5.0, 20.0),
            floor_height: 0.5,
            wall_height: 10.0,
            sockets: vec![
                base_socket(SocketSide::South, Vec3::new(20.0, 5.0, 20.0), DoorType::Bulkhead),
            ],
            mesh_color: Color::srgb(0.08, 0.12, 0.08),
        },
        RoomPrefab {
            name: "ObjectiveRoom_B".to_string(),
            zone_type: ZoneType::Objective,
            half_size: Vec3::new(22.0, 5.0, 19.0),
            floor_height: 0.5,
            wall_height: 10.0,
            sockets: vec![
                base_socket(SocketSide::South, Vec3::new(22.0, 5.0, 19.0), DoorType::Bulkhead),
                base_socket(SocketSide::East, Vec3::new(22.0, 5.0, 19.0), DoorType::Normal),
            ],
            mesh_color: Color::srgb(0.07, 0.14, 0.07),
        },
        RoomPrefab {
            name: "ObjectiveRoom_C".to_string(),
            zone_type: ZoneType::Objective,
            half_size: Vec3::new(19.0, 5.0, 21.0),
            floor_height: 0.5,
            wall_height: 10.0,
            sockets: vec![
                shifted_socket(SocketSide::South, Vec3::new(19.0, 5.0, 21.0), 3.0, DoorType::Bulkhead),
                base_socket(SocketSide::North, Vec3::new(19.0, 5.0, 21.0), DoorType::Normal),
            ],
            mesh_color: Color::srgb(0.09, 0.10, 0.06),
        },
        RoomPrefab {
            name: "ObjectiveRoom_D".to_string(),
            zone_type: ZoneType::Objective,
            half_size: Vec3::new(24.0, 5.0, 24.0),
            floor_height: 0.5,
            wall_height: 10.0,
            sockets: vec![
                base_socket(SocketSide::South, Vec3::new(24.0, 5.0, 24.0), DoorType::Bulkhead),
                base_socket(SocketSide::East, Vec3::new(24.0, 5.0, 24.0), DoorType::Normal),
                base_socket(SocketSide::West, Vec3::new(24.0, 5.0, 24.0), DoorType::Normal),
            ],
            mesh_color: Color::srgb(0.06, 0.11, 0.06),
        },
    ]
}

pub fn new_utility_variants() -> Vec<RoomPrefab> {
    vec![
        RoomPrefab {
            name: "UtilityRoom_A".to_string(),
            zone_type: ZoneType::Utility,
            half_size: Vec3::new(15.0, 5.0, 15.0),
            floor_height: 0.5,
            wall_height: 10.0,
            sockets: vec![
                base_socket(SocketSide::North, Vec3::new(15.0, 5.0, 15.0), DoorType::Normal),
            ],
            mesh_color: Color::srgb(0.12, 0.15, 0.16),
        },
        RoomPrefab {
            name: "UtilityRoom_B".to_string(),
            zone_type: ZoneType::Utility,
            half_size: Vec3::new(14.0, 5.0, 17.0),
            floor_height: 0.5,
            wall_height: 10.0,
            sockets: vec![
                base_socket(SocketSide::North, Vec3::new(14.0, 5.0, 17.0), DoorType::Normal),
                base_socket(SocketSide::West, Vec3::new(14.0, 5.0, 17.0), DoorType::Keycard),
            ],
            mesh_color: Color::srgb(0.13, 0.14, 0.17),
        },
        RoomPrefab {
            name: "UtilityRoom_C".to_string(),
            zone_type: ZoneType::Utility,
            half_size: Vec3::new(16.0, 5.0, 16.0),
            floor_height: 0.5,
            wall_height: 10.0,
            sockets: vec![
                shifted_socket(SocketSide::North, Vec3::new(16.0, 5.0, 16.0), 4.0, DoorType::Normal),
                shifted_socket(SocketSide::East, Vec3::new(16.0, 5.0, 16.0), 3.0, DoorType::Normal),
            ],
            mesh_color: Color::srgb(0.11, 0.13, 0.18),
        },
        RoomPrefab {
            name: "UtilityRoom_D".to_string(),
            zone_type: ZoneType::Utility,
            half_size: Vec3::new(13.0, 5.0, 18.0),
            floor_height: 0.5,
            wall_height: 10.0,
            sockets: vec![
                base_socket(SocketSide::North, Vec3::new(13.0, 5.0, 18.0), DoorType::Normal),
                base_socket(SocketSide::South, Vec3::new(13.0, 5.0, 18.0), DoorType::Normal),
            ],
            mesh_color: Color::srgb(0.10, 0.16, 0.15),
        },
    ]
}

pub fn new_storage_variants() -> Vec<RoomPrefab> {
    vec![
        RoomPrefab {
            name: "StorageRoom_A".to_string(),
            zone_type: ZoneType::Storage,
            half_size: Vec3::new(20.0, 5.0, 20.0),
            floor_height: 0.5,
            wall_height: 10.0,
            sockets: vec![
                base_socket(SocketSide::North, Vec3::new(20.0, 5.0, 20.0), DoorType::Normal),
            ],
            mesh_color: Color::srgb(0.13, 0.13, 0.12),
        },
        RoomPrefab {
            name: "StorageRoom_B".to_string(),
            zone_type: ZoneType::Storage,
            half_size: Vec3::new(22.0, 5.0, 18.0),
            floor_height: 0.5,
            wall_height: 10.0,
            sockets: vec![
                base_socket(SocketSide::North, Vec3::new(22.0, 5.0, 18.0), DoorType::Normal),
                base_socket(SocketSide::East, Vec3::new(22.0, 5.0, 18.0), DoorType::Alarm),
            ],
            mesh_color: Color::srgb(0.14, 0.12, 0.11),
        },
        RoomPrefab {
            name: "StorageRoom_C".to_string(),
            zone_type: ZoneType::Storage,
            half_size: Vec3::new(19.0, 5.0, 21.0),
            floor_height: 0.5,
            wall_height: 10.0,
            sockets: vec![
                shifted_socket(SocketSide::North, Vec3::new(19.0, 5.0, 21.0), -3.0, DoorType::Normal),
                base_socket(SocketSide::South, Vec3::new(19.0, 5.0, 21.0), DoorType::Normal),
            ],
            mesh_color: Color::srgb(0.12, 0.13, 0.10),
        },
        RoomPrefab {
            name: "StorageRoom_D".to_string(),
            zone_type: ZoneType::Storage,
            half_size: Vec3::new(21.0, 5.0, 21.0),
            floor_height: 0.5,
            wall_height: 10.0,
            sockets: vec![
                base_socket(SocketSide::North, Vec3::new(21.0, 5.0, 21.0), DoorType::Normal),
                base_socket(SocketSide::East, Vec3::new(21.0, 5.0, 21.0), DoorType::Normal),
                base_socket(SocketSide::West, Vec3::new(21.0, 5.0, 21.0), DoorType::Normal),
            ],
            mesh_color: Color::srgb(0.15, 0.14, 0.13),
        },
    ]
}

pub fn variants_for_zone_type(zone_type: ZoneType) -> &'static [RoomPrefab] {
    match zone_type {
        ZoneType::Hub => &*HUB_VARIANTS,
        ZoneType::Corridor => &*CORRIDAL_VARIANTS,
        ZoneType::Industrial => &*INDUSTRIAL_VARIANTS,
        ZoneType::Objective => &*OBJECTIVE_VARIANTS,
        ZoneType::Utility => &*UTILITY_VARIANTS,
        ZoneType::Storage => &*STORAGE_VARIANTS,
    }
}

pub static HUB_VARIANTS: LazyLock<Vec<RoomPrefab>> = LazyLock::new(new_hub_variants);
pub static CORRIDAL_VARIANTS: LazyLock<Vec<RoomPrefab>> = LazyLock::new(new_corridor_variants);
pub static INDUSTRIAL_VARIANTS: LazyLock<Vec<RoomPrefab>> = LazyLock::new(new_industrial_variants);
pub static OBJECTIVE_VARIANTS: LazyLock<Vec<RoomPrefab>> = LazyLock::new(new_objective_variants);
pub static UTILITY_VARIANTS: LazyLock<Vec<RoomPrefab>> = LazyLock::new(new_utility_variants);
pub static STORAGE_VARIANTS: LazyLock<Vec<RoomPrefab>> = LazyLock::new(new_storage_variants);

pub fn new_hub() -> RoomPrefab {
    variants_for_zone_type(ZoneType::Hub)[0].clone()
}

pub fn new_corridor() -> RoomPrefab {
    variants_for_zone_type(ZoneType::Corridor)[0].clone()
}

pub fn new_objective() -> RoomPrefab {
    variants_for_zone_type(ZoneType::Objective)[0].clone()
}

pub fn new_industrial() -> RoomPrefab {
    variants_for_zone_type(ZoneType::Industrial)[0].clone()
}

pub fn new_utility() -> RoomPrefab {
    variants_for_zone_type(ZoneType::Utility)[0].clone()
}

pub fn new_storage() -> RoomPrefab {
    variants_for_zone_type(ZoneType::Storage)[0].clone()
}

pub fn for_zone_type(zone_type: ZoneType) -> RoomPrefab {
    variants_for_zone_type(zone_type)[0].clone()
}

pub fn select_variant(
    zone_type: ZoneType,
    required_sides: &[SocketSide],
    rng: &mut impl Rng,
) -> RoomPrefab {
    let variants = variants_for_zone_type(zone_type);
    let variants: Vec<RoomPrefab> = variants.to_vec();
    let required_set: std::collections::HashSet<SocketSide> =
        required_sides.iter().copied().collect();

    let perfect: Vec<RoomPrefab> = variants
        .iter()
        .filter(|v| required_set.iter().all(|s| v.has_socket(*s)))
        .cloned()
        .collect();

    let pool = if perfect.is_empty() {
        variants.as_slice()
    } else {
        perfect.as_slice()
    };

    let idx = rng.random_range(0..pool.len());
    pool[idx].clone()
}

#[derive(Component, Debug)]
pub struct RoomPrefabTag {
    pub prefab_name: String,
    pub zone_id: ZoneId,
}

#[derive(Component, Debug)]
pub struct RoomSocketTag {
    pub side: SocketSide,
    pub zone_id: ZoneId,
}

#[derive(Component, Debug)]
pub struct ConnectorTag {
    pub connection_index: usize,
}

fn wall_half_span_for_side(half_size: Vec3, side: SocketSide) -> f32 {
    match side {
        SocketSide::East | SocketSide::West => half_size.z,
        SocketSide::North | SocketSide::South => half_size.x,
    }
}

fn opening_coord_for_side(socket_offset: Vec3, side: SocketSide) -> f32 {
    match side {
        SocketSide::East | SocketSide::West => socket_offset.z,
        SocketSide::North | SocketSide::South => socket_offset.x,
    }
}

fn build_wall_segments_prefab(
    half_size: Vec3,
    socket_offsets_on_side: &[Vec3],
    side: SocketSide,
) -> Vec<(f32, f32)> {
    let half_span = wall_half_span_for_side(half_size, side);
    if half_span <= 0.0 {
        return Vec::new();
    }

    if socket_offsets_on_side.is_empty() {
        return vec![(0.0, half_span * 2.0)];
    }

    let opening_half_width = DOOR_OPENING_WIDTH * 0.5 + WALL_THICKNESS / 2.0;

    let mut ranges: Vec<(f32, f32)> = socket_offsets_on_side
        .iter()
        .map(|offset| {
            let coord = opening_coord_for_side(*offset, side);
            let clamped_coord = coord.clamp(-(half_span - DOOR_EDGE_MARGIN), half_span - DOOR_EDGE_MARGIN);
            (
                (clamped_coord - opening_half_width).clamp(-half_span, half_span),
                (clamped_coord + opening_half_width).clamp(-half_span, half_span),
            )
        })
        .filter(|(start, end)| end > start)
        .collect();

    if ranges.is_empty() {
        return vec![(0.0, half_span * 2.0)];
    }

    ranges.sort_by(|(a_start, _), (b_start, _)| {
        a_start
            .partial_cmp(b_start)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let mut merged: Vec<(f32, f32)> = Vec::with_capacity(ranges.len());
    for (start, end) in ranges {
        if let Some((_, prev_end)) = merged.last_mut()
            && start <= *prev_end
        {
            *prev_end = prev_end.max(end);
        } else {
            merged.push((start, end));
        }
    }

    let mut segments = Vec::new();
    let mut cursor = -half_span;

    for (open_start, open_end) in merged {
        let segment_length = open_start - cursor;
        if segment_length >= 0.5 {
            segments.push((cursor + segment_length * 0.5, segment_length));
        }
        cursor = cursor.max(open_end);
    }

    let tail_length = half_span - cursor;
    if tail_length >= 0.5 {
        segments.push((cursor + tail_length * 0.5, tail_length));
    }

    segments
}

fn spawn_wall_segment(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    wall_size: Vec3,
    wall_position: Vec3,
    rotation: Quat,
    wall_color: Color,
    zone_id: ZoneId,
    name: &str,
) {
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::from_size(wall_size))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: wall_color,
            perceptual_roughness: 0.92,
            ..default()
        })),
        Transform::from_translation(wall_position).with_rotation(rotation),
        NavigationObstacle,
        Name::new(format!("{}_Zone_{}", name, zone_id.0)),
    ));
}

fn spawn_room_walls(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    prefab: &RoomPrefab,
    room_position: Vec3,
    rotation: Quat,
    zone_id: ZoneId,
) {
    let half_size = prefab.half_size;
    let wall_height = prefab.wall_height;
    let wall_thickness = 1.0;
    let wall_color = prefab.mesh_color.with_alpha(0.9);

    let side_specs = [
        (SocketSide::East, Vec3::new(half_size.x, wall_height * 0.5, 0.0), "East"),
        (SocketSide::West, Vec3::new(-half_size.x, wall_height * 0.5, 0.0), "West"),
        (SocketSide::North, Vec3::new(0.0, wall_height * 0.5, half_size.z), "North"),
        (SocketSide::South, Vec3::new(0.0, wall_height * 0.5, -half_size.z), "South"),
    ];

    for (side, anchor, side_name) in side_specs {
        let socket_offsets_on_side: Vec<Vec3> = prefab
            .sockets
            .iter()
            .filter(|s| s.side == side)
            .map(|s| s.offset)
            .collect();

        let segments = build_wall_segments_prefab(half_size, &socket_offsets_on_side, side);

        for (segment_index, (segment_center, segment_length)) in segments.iter().enumerate() {
            let local_offset = match side {
                SocketSide::East | SocketSide::West => Vec3::new(0.0, 0.0, *segment_center),
                SocketSide::North | SocketSide::South => Vec3::new(*segment_center, 0.0, 0.0),
            };

            let wall_size = match side {
                SocketSide::East | SocketSide::West => Vec3::new(wall_thickness, wall_height, *segment_length),
                SocketSide::North | SocketSide::South => Vec3::new(*segment_length, wall_height, wall_thickness),
            };

            let wall_pos = room_position + rotation * (anchor + local_offset);
            spawn_wall_segment(
                commands,
                meshes,
                materials,
                wall_size,
                wall_pos,
                rotation,
                wall_color,
                zone_id,
                &format!("Wall_{}_{}_{}", side_name, segment_index, prefab.name),
            );
        }
    }
}

pub fn spawn_room_prefab(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    prefab: &RoomPrefab,
    zone: &Zone,
) -> Entity {
    let room_position = zone.position;
    let room_entity = commands
        .spawn((
            RoomPrefabTag {
                prefab_name: prefab.name.clone(),
                zone_id: zone.id,
            },
            Transform::from_translation(room_position).with_rotation(zone.rotation),
            Name::new(format!("Room_{}_{}", prefab.name, zone.id.0)),
        ))
        .id();

    let half_size = prefab.half_size;
    let wall_height = prefab.wall_height;
    let floor_height = prefab.floor_height;

    let floor_pos = room_position + Vec3::new(0.0, -floor_height / 2.0, 0.0);
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(half_size.x * 2.0, floor_height, half_size.z * 2.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: prefab.mesh_color,
            perceptual_roughness: 0.9,
            ..default()
        })),
        Transform::from_translation(floor_pos).with_rotation(zone.rotation),
        NavigationObstacle,
        Name::new(format!("Floor_Room_{}_{}", prefab.name, zone.id.0)),
    ));

    let ceiling_pos = room_position + Vec3::new(0.0, wall_height + floor_height / 2.0, 0.0);
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(half_size.x * 2.0, floor_height, half_size.z * 2.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: prefab.mesh_color.with_alpha(0.8),
            perceptual_roughness: 0.9,
            ..default()
        })),
        Transform::from_translation(ceiling_pos).with_rotation(zone.rotation),
        Name::new(format!("Ceiling_Room_{}_{}", prefab.name, zone.id.0)),
    ));

    spawn_room_walls(commands, meshes, materials, prefab, room_position, zone.rotation, zone.id);

    for socket in &prefab.sockets {
        let socket_pos = room_position + zone.rotation * socket.offset;
        commands.spawn((
            Transform::from_translation(socket_pos).with_rotation(zone.rotation),
            RoomSocketTag {
                side: socket.side,
                zone_id: zone.id,
            },
            Name::new(format!("Socket_{:?}_Zone_{}", socket.side, zone.id.0)),
        ));
    }

    room_entity
}

fn compute_required_sides(
    zone: &Zone,
    level_graph: &LevelGraph,
) -> Vec<SocketSide> {
    let mut sides = Vec::new();
    for conn in level_graph.connections_of(zone.id) {
        let other_id = if conn.from_zone == zone.id {
            conn.to_zone
        } else {
            conn.from_zone
        };
        if let Some(other) = level_graph.get_zone(other_id) {
            let local_dir = zone.rotation.inverse() * (other.position - zone.position);
            sides.push(SocketSide::from_local_direction(local_dir));
        }
    }
    sides
}

fn room_edge_position(
    zone: &Zone,
    half_size: Vec3,
    side: SocketSide,
) -> Vec3 {
    let local_edge = match side {
        SocketSide::East => Vec3::new(half_size.x, 0.0, 0.0),
        SocketSide::West => Vec3::new(-half_size.x, 0.0, 0.0),
        SocketSide::North => Vec3::new(0.0, 0.0, half_size.z),
        SocketSide::South => Vec3::new(0.0, 0.0, -half_size.z),
    };
    zone.position + zone.rotation * local_edge
}

pub fn spawn_connector(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    zone_a: &Zone,
    zone_b: &Zone,
    prefab_a: &RoomPrefab,
    prefab_b: &RoomPrefab,
    connection: &ZoneConnection,
    index: usize,
) {
    let door_position = connection.door_position;
    let dir_ab = (zone_b.position - zone_a.position).normalize_or_zero();

    let local_dir_a = zone_a.rotation.inverse() * dir_ab;
    let local_dir_b = zone_b.rotation.inverse() * (-dir_ab);

    let side_a = SocketSide::from_local_direction(local_dir_a);
    let side_b = SocketSide::from_local_direction(local_dir_b);

    // Calculate the wall edge positions (the inner wall edges closest to the door)
    let edge_a = room_edge_position(zone_a, prefab_a.half_size, side_a);
    let edge_b = room_edge_position(zone_b, prefab_b.half_size, side_b);

    // The connector spans from edge_a to edge_b, centered at door_position
    // Along the connection axis, the connector length is half the distance between edges
    let connector_length = (edge_b - edge_a).length().max(0.5);

    // Perpendicular to the connection direction (in the horizontal plane)
    let perp = Vec3::new(-dir_ab.z, 0.0, dir_ab.x);
    let perp = if perp.length() < 0.01 { Vec3::X } else { perp.normalize() };

    // Determine connector width (match the door opening)
    let connector_width = DOOR_OPENING_WIDTH + WALL_THICKNESS * 2.0; // 6.0 + 1.0 = 7.0

    let connector_center = door_position;

    // We build the connector as: floor, ceiling, side walls, and front/back walls with door opening
    let wall_height = prefab_a.wall_height.max(prefab_b.wall_height);
    let floor_thickness = FLOOR_THICKNESS;

    let floor_size = Vec3::new(connector_length, floor_thickness, connector_width);
    let floor_pos = connector_center + Vec3::new(0.0, -floor_thickness / 2.0, 0.0);

    // Floor bridge
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::from_size(floor_size))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: prefab_a.mesh_color,
            perceptual_roughness: 0.9,
            ..default()
        })),
        Transform::from_translation(floor_pos),
        NavigationObstacle,
        ConnectorTag { connection_index: index },
        Name::new(format!("ConnectorFloor_{}", index)),
    ));

    // Ceiling bridge
    let ceiling_pos = connector_center + Vec3::new(0.0, wall_height + floor_thickness / 2.0, 0.0);
    let ceiling_size = Vec3::new(connector_length, floor_thickness, connector_width);
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::from_size(ceiling_size))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: prefab_a.mesh_color.with_alpha(0.8),
            perceptual_roughness: 0.9,
            ..default()
        })),
        Transform::from_translation(ceiling_pos),
        Name::new(format!("ConnectorCeiling_{}", index)),
    ));

    // Side walls of the connector (parallel to connection direction)
    let side_wall_thickness = 0.5;
    let side_wall_length = connector_length;
    let side_wall_height = wall_height;
    let side_wall_size = Vec3::new(side_wall_length, side_wall_height, side_wall_thickness);

    // Left side (offset by perpendicular direction)
    let mut left_pos = connector_center + perp * (connector_width / 2.0 - side_wall_thickness / 2.0);
    left_pos.y = side_wall_height / 2.0;
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::from_size(side_wall_size))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: prefab_a.mesh_color.with_alpha(0.9),
            perceptual_roughness: 0.92,
            ..default()
        })),
        Transform::from_translation(left_pos),
        NavigationObstacle,
        Name::new(format!("ConnectorSideWall_Left_{}", index)),
    ));

    let mut right_pos = connector_center - perp * (connector_width / 2.0 - side_wall_thickness / 2.0);
    right_pos.y = side_wall_height / 2.0;
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::from_size(side_wall_size))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: prefab_a.mesh_color.with_alpha(0.9),
            perceptual_roughness: 0.92,
            ..default()
        })),
        Transform::from_translation(right_pos),
        NavigationObstacle,
        Name::new(format!("ConnectorSideWall_Right_{}", index)),
    ));

    // The connector is open in the direction of the rooms - the door opening is the entire connector
    // Front/back walls with door openings are NOT needed - the rooms already have door openings
}

fn spawn_connector_physics(
    commands: &mut Commands,
    zone_a: &Zone,
    zone_b: &Zone,
    prefab_a: &RoomPrefab,
    prefab_b: &RoomPrefab,
    connection: &ZoneConnection,
    index: usize,
) {
    let door_position = connection.door_position;
    let dir_ab = (zone_b.position - zone_a.position).normalize_or_zero();
    let perp = Vec3::new(-dir_ab.z, 0.0, dir_ab.x);
    let perp = if perp.length() < 0.01 { Vec3::X } else { perp.normalize() };

    let side_a = SocketSide::from_local_direction(zone_a.rotation.inverse() * dir_ab);
    let side_b = SocketSide::from_local_direction(zone_b.rotation.inverse() * (-dir_ab));

    let edge_a = room_edge_position(zone_a, prefab_a.half_size, side_a);
    let edge_b = room_edge_position(zone_b, prefab_b.half_size, side_b);

    let connector_length = (edge_b - edge_a).length().max(0.5);
    let connector_width = DOOR_OPENING_WIDTH;
    let wall_height = prefab_a.wall_height.max(prefab_b.wall_height);

    let connector_center = door_position;

    // Physics floor bridge
    let floor_pos = connector_center + Vec3::new(0.0, -0.5, 0.0);
    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(connector_length, 0.5, connector_width),
        Position::new(floor_pos),
        Rotation::default(),
        Transform::from_translation(floor_pos),
        NavigationObstacle,
        Name::new(format!("Physics_ConnectorFloor_{}", index)),
    ));

    // Physics side walls of the connector
    let side_wall_thickness = 0.5;
    let side_wall_length = connector_length;
    let side_wall_size = Vec3::new(side_wall_length, wall_height, side_wall_thickness);

    let mut left_pos = connector_center + perp * (connector_width / 2.0 + side_wall_thickness / 2.0);
    left_pos.y = wall_height / 2.0;
    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(side_wall_size.x, side_wall_size.y, side_wall_size.z),
        Position::new(left_pos),
        Rotation::default(),
        Transform::from_translation(left_pos),
        NavigationObstacle,
        Name::new(format!("Physics_ConnectorWall_L_{}", index)),
    ));

    let mut right_pos = connector_center - perp * (connector_width / 2.0 + side_wall_thickness / 2.0);
    right_pos.y = wall_height / 2.0;
    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(side_wall_size.x, side_wall_size.y, side_wall_size.z),
        Position::new(right_pos),
        Rotation::default(),
        Transform::from_translation(right_pos),
        NavigationObstacle,
        Name::new(format!("Physics_ConnectorWall_R_{}", index)),
    ));
}

pub fn build_prefab_level(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    level_graph: &LevelGraph,
) {
    let mut rng = StdRng::seed_from_u64(level_graph.config.seed);

    commands.spawn((
        AmbientLight {
            color: Color::srgb(0.16, 0.2, 0.24),
            brightness: 24.0,
            ..default()
        },
        Name::new("ProceduralAmbientLight"),
    ));

    for zone in level_graph.zones.values() {
        let required_sides = compute_required_sides(zone, level_graph);
        let prefab = select_variant(zone.zone_type, &required_sides, &mut rng);
        spawn_room_prefab(
            &mut commands,
            &mut meshes,
            &mut materials,
            &prefab,
            zone,
        );
    }

    spawn_all_connectors(
        &mut commands,
        &mut meshes,
        &mut materials,
        level_graph,
        &mut rng,
    );
}

fn spawn_all_connectors(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    level_graph: &LevelGraph,
    rng: &mut StdRng,
) {
    let zone_prefabs: std::collections::HashMap<ZoneId, RoomPrefab> = level_graph
        .zones
        .values()
        .map(|zone| {
            let sides = compute_required_sides(zone, level_graph);
            let prefab = select_variant(zone.zone_type, &sides, rng);
            (zone.id, prefab)
        })
        .collect();

    for (index, connection) in level_graph.connections.iter().enumerate() {
        let Some(zone_a) = level_graph.get_zone(connection.from_zone) else {
            continue;
        };
        let Some(zone_b) = level_graph.get_zone(connection.to_zone) else {
            continue;
        };

        let Some(prefab_a) = zone_prefabs.get(&zone_a.id) else {
            continue;
        };
        let Some(prefab_b) = zone_prefabs.get(&zone_b.id) else {
            continue;
        };

        spawn_connector(
            commands,
            meshes,
            materials,
            zone_a,
            zone_b,
            prefab_a,
            prefab_b,
            connection,
            index,
        );
        spawn_connector_physics(
            commands,
            zone_a,
            zone_b,
            prefab_a,
            prefab_b,
            connection,
            index,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::level::generation::LevelConfig;

    #[test]
    fn hub_variants_have_enough_variants() {
        let variants = new_hub_variants();
        assert!(variants.len() >= 4, "Hub should have at least 4 variants");
    }

    #[test]
    fn corridor_variants_have_opposite_end_sockets() {
        let variants = new_corridor_variants();
        for v in &variants {
            assert!(v.has_socket(SocketSide::North) || v.has_socket(SocketSide::South));
            assert!(v.has_socket(SocketSide::South) || v.has_socket(SocketSide::North));
        }
    }

    #[test]
    fn objective_variants_have_bulkhead_socket() {
        let variants = new_objective_variants();
        for v in &variants {
            let has_bulkhead = v.sockets.iter().any(|s| {
                s.side == SocketSide::South && s.door_type == DoorType::Bulkhead
            });
            assert!(has_bulkhead, "Objective room should have a South bulkhead socket");
        }
    }

    #[test]
    fn select_variant_matches_required_sides() {
        let mut rng = StdRng::seed_from_u64(42);
        let sides = vec![SocketSide::North, SocketSide::East];
        let prefab = select_variant(ZoneType::Hub, &sides, &mut rng);
        assert!(prefab.has_socket(SocketSide::North));
        assert!(prefab.has_socket(SocketSide::East));
    }

    #[test]
    fn room_prefab_socket_positions_are_at_edges() {
        let prefab = new_hub();
        for socket in &prefab.sockets {
            let is_at_edge = socket.offset.x.abs() >= prefab.half_size.x - 0.01
                || socket.offset.z.abs() >= prefab.half_size.z - 0.01;
            assert!(is_at_edge, "Socket should be at room edge, got offset {:?}", socket.offset);
        }
    }

    #[test]
    fn compute_required_sides_for_single_connection() {
        let mut graph = LevelGraph::new(LevelConfig::default());
        let zone_a = Zone::new(ZoneId(0), ZoneType::Hub, Vec3::ZERO, Quat::IDENTITY);
        let zone_b = Zone::new(ZoneId(1), ZoneType::Corridor, Vec3::new(35.0, 0.0, 0.0), Quat::IDENTITY);
        graph.add_zone(zone_a);
        graph.add_zone(zone_b);
        graph.add_connection(ZoneId(0), ZoneId(1), Vec3::new(17.5, 0.0, 0.0), Quat::IDENTITY, DoorType::Normal);

        let zone = graph.get_zone(ZoneId(0)).unwrap();
        let sides = compute_required_sides(zone, &graph);
        assert_eq!(sides.len(), 1);
        assert_eq!(sides[0], SocketSide::East);
    }

    #[test]
    fn room_edge_position_matches_socket_offset() {
        let zone = Zone::new(ZoneId(0), ZoneType::Hub, Vec3::ZERO, Quat::IDENTITY);
        let half_size = Vec3::new(20.0, 5.0, 20.0);
        let edge = room_edge_position(&zone, half_size, SocketSide::North);
        assert!((edge.z - 20.0).abs() < 0.01);
        assert!((edge.x).abs() < 0.01);
    }
}
