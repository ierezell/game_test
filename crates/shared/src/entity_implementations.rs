use super::entity_traits::*;
use crate::enemy::{Enemy, EnemyBundle};
use crate::input::{PLAYER_CAPSULE_HEIGHT, PLAYER_CAPSULE_RADIUS};
use crate::scene::*;

use avian3d::prelude::{
    AngularDamping, Collider, Friction, LinearDamping, LockedAxes, Mass, Restitution, RigidBody,
};
use bevy::{
    color::palettes::css::{BLUE, GREEN, RED, WHITE},
    prelude::{Bundle, Capsule3d, Color, Cuboid, Mesh, StandardMaterial, default},
};

/// Floor entity implementation
pub struct FloorEntity {
    pub size: f32,
    pub thickness: f32,
}

impl Default for FloorEntity {
    fn default() -> Self {
        Self {
            size: ROOM_SIZE * 2.0,
            thickness: FLOOR_THICKNESS,
        }
    }
}

impl VisualProvider for FloorEntity {
    fn get_mesh(&self) -> Mesh {
        Cuboid::new(self.size, self.thickness, self.size).into()
    }

    fn get_material(&self) -> StandardMaterial {
        StandardMaterial {
            base_color: self.get_color(),
            ..default()
        }
    }

    fn get_color(&self) -> Color {
        GREEN.into()
    }
}

impl PhysicsProvider for FloorEntity {
    type PhysicsBundle = FloorPhysicsBundle;

    fn get_physics_bundle(&self) -> Self::PhysicsBundle {
        FloorPhysicsBundle {
            collider: self.get_collider(),
            rigid_body: self.get_rigid_body(),
            restitution: Restitution::ZERO,
        }
    }

    fn get_collider(&self) -> Collider {
        Collider::cuboid(ROOM_SIZE, self.thickness, ROOM_SIZE)
    }

    fn get_rigid_body(&self) -> RigidBody {
        RigidBody::Static
    }
}

impl GameEntity for FloorEntity {
    fn entity_type(&self) -> &'static str {
        "Floor"
    }
}

impl Spawnable for FloorEntity {}

/// Wall entity implementation
pub struct WallEntity {
    pub wall_type: WallType,
}

#[derive(Clone, Debug)]
pub enum WallType {
    North,
    South,
    East,
    West,
}

impl WallType {
    pub fn get_dimensions(&self) -> (f32, f32, f32) {
        match self {
            WallType::North | WallType::South => (ROOM_SIZE, WALL_HEIGHT, WALL_THICKNESS),
            WallType::East | WallType::West => (WALL_THICKNESS, WALL_HEIGHT, ROOM_SIZE),
        }
    }

    pub fn get_name(&self) -> &'static str {
        match self {
            WallType::North => "Wall North",
            WallType::South => "Wall South",
            WallType::East => "Wall East",
            WallType::West => "Wall West",
        }
    }
}

impl Default for WallEntity {
    fn default() -> Self {
        Self {
            wall_type: WallType::North,
        }
    }
}

impl WallEntity {
    pub fn new(wall_type: WallType) -> Self {
        Self { wall_type }
    }
}

impl VisualProvider for WallEntity {
    fn get_mesh(&self) -> Mesh {
        let (width, height, depth) = self.wall_type.get_dimensions();
        Cuboid::new(width, height, depth).into()
    }

    fn get_material(&self) -> StandardMaterial {
        StandardMaterial {
            base_color: self.get_color(),
            ..default()
        }
    }

    fn get_color(&self) -> Color {
        WHITE.into()
    }
}

impl PhysicsProvider for WallEntity {
    type PhysicsBundle = WallPhysicsBundle;

    fn get_physics_bundle(&self) -> Self::PhysicsBundle {
        WallPhysicsBundle {
            collider: self.get_collider(),
            rigid_body: self.get_rigid_body(),
        }
    }

    fn get_collider(&self) -> Collider {
        let (width, height, depth) = self.wall_type.get_dimensions();
        Collider::cuboid(width, height, depth)
    }

    fn get_rigid_body(&self) -> RigidBody {
        RigidBody::Static
    }
}

impl GameEntity for WallEntity {
    fn entity_type(&self) -> &'static str {
        "Wall"
    }
}

impl Spawnable for WallEntity {}

/// Player entity implementation
pub struct PlayerEntity {
    pub color: Color,
    pub radius: f32,
    pub height: f32,
    pub mass: f32,
}

impl Default for PlayerEntity {
    fn default() -> Self {
        Self {
            color: BLUE.into(),
            radius: PLAYER_CAPSULE_RADIUS,
            height: PLAYER_CAPSULE_HEIGHT,
            mass: 80.0,
        }
    }
}

impl PlayerEntity {
    pub fn with_color(color: Color) -> Self {
        Self {
            color,
            ..Default::default()
        }
    }
}

/// Enhanced physics bundle for improved player movement
#[derive(Bundle)]
pub struct EnhancedPlayerPhysicsBundle {
    // Legacy physics components
    pub rigid_body: RigidBody,
    pub collider: Collider,
    pub mass: Mass,
    pub restitution: Restitution,
    pub friction: Friction,
    pub linear_damping: LinearDamping,
    pub angular_damping: AngularDamping,
    pub locked_axes: LockedAxes,
}

impl Default for EnhancedPlayerPhysicsBundle {
    fn default() -> Self {
        Self {
            rigid_body: RigidBody::Dynamic,
            collider: Collider::capsule(PLAYER_CAPSULE_HEIGHT, PLAYER_CAPSULE_RADIUS),
            mass: Mass(80.0),
            restitution: Restitution::ZERO,
            friction: Friction::ZERO,
            linear_damping: LinearDamping(1.0),
            angular_damping: AngularDamping(8.0),
            locked_axes: LockedAxes::ROTATION_LOCKED.unlock_rotation_y(),
        }
    }
}

impl VisualProvider for PlayerEntity {
    fn get_mesh(&self) -> Mesh {
        Capsule3d::new(self.radius, self.height).into()
    }

    fn get_material(&self) -> StandardMaterial {
        StandardMaterial {
            base_color: self.get_color(),
            ..default()
        }
    }

    fn get_color(&self) -> Color {
        self.color
    }
}

impl PhysicsProvider for PlayerEntity {
    type PhysicsBundle = EnhancedPlayerPhysicsBundle;

    fn get_physics_bundle(&self) -> Self::PhysicsBundle {
        EnhancedPlayerPhysicsBundle {
            rigid_body: self.get_rigid_body(),
            collider: self.get_collider(),
            mass: Mass(self.mass),
            restitution: Restitution::ZERO,
            friction: Friction::ZERO,
            linear_damping: LinearDamping(1.0),
            angular_damping: AngularDamping(8.0),
            locked_axes: LockedAxes::ROTATION_LOCKED.unlock_rotation_y(),
        }
    }

    fn get_collider(&self) -> Collider {
        Collider::capsule(self.height, self.radius)
    }

    fn get_rigid_body(&self) -> RigidBody {
        RigidBody::Dynamic
    }
}

impl GameEntity for PlayerEntity {
    fn entity_type(&self) -> &'static str {
        "Player"
    }
}

impl Spawnable for PlayerEntity {
    fn get_spawn_offset(&self) -> Option<bevy::prelude::Vec3> {
        Some(bevy::prelude::Vec3::new(0.0, self.height + 0.6, 0.0))
    }
}

/// Enemy entity implementation
pub struct EnemyEntity {
    pub color: Color,
    pub radius: f32,
    pub height: f32,
    pub mass: f32,
    pub enemy_type: EnemyType,
}

#[derive(Clone, Debug)]
pub enum EnemyType {
    Basic,
    Fast,
    Heavy,
}

impl Default for EnemyEntity {
    fn default() -> Self {
        Self {
            color: RED.into(),
            radius: PLAYER_CAPSULE_RADIUS * 0.8, // Slightly smaller than players
            height: PLAYER_CAPSULE_HEIGHT * 0.9,
            mass: 60.0,
            enemy_type: EnemyType::Basic,
        }
    }
}

impl EnemyEntity {
    pub fn new(enemy_type: EnemyType) -> Self {
        let (color, radius, height, mass) = match enemy_type {
            EnemyType::Basic => (
                RED.into(),
                PLAYER_CAPSULE_RADIUS * 0.8,
                PLAYER_CAPSULE_HEIGHT * 0.9,
                60.0,
            ),
            EnemyType::Fast => (
                Color::srgb(1.0, 0.5, 0.0),
                PLAYER_CAPSULE_RADIUS * 0.6,
                PLAYER_CAPSULE_HEIGHT * 0.8,
                40.0,
            ), // Orange
            EnemyType::Heavy => (
                Color::srgb(0.5, 0.0, 0.0),
                PLAYER_CAPSULE_RADIUS * 1.2,
                PLAYER_CAPSULE_HEIGHT * 1.1,
                100.0,
            ), // Dark red
        };

        Self {
            color,
            radius,
            height,
            mass,
            enemy_type,
        }
    }
}

impl VisualProvider for EnemyEntity {
    fn get_mesh(&self) -> Mesh {
        Capsule3d::new(self.radius, self.height).into()
    }

    fn get_material(&self) -> StandardMaterial {
        StandardMaterial {
            base_color: self.get_color(),
            ..default()
        }
    }

    fn get_color(&self) -> Color {
        self.color
    }
}

/// Enemy physics bundle
#[derive(Bundle)]
pub struct EnemyPhysicsBundle {
    pub rigid_body: RigidBody,
    pub collider: Collider,
    pub mass: Mass,
    pub restitution: Restitution,
    pub friction: Friction,
    pub linear_damping: LinearDamping,
    pub angular_damping: AngularDamping,
    pub locked_axes: LockedAxes,
    pub enemy_bundle: EnemyBundle,
}

impl PhysicsProvider for EnemyEntity {
    type PhysicsBundle = EnemyPhysicsBundle;

    fn get_physics_bundle(&self) -> Self::PhysicsBundle {
        let enemy = match self.enemy_type {
            EnemyType::Basic => Enemy {
                detection_range: 8.0,
                attack_range: 2.0,
                move_speed: 3.0,
                health: 80.0,
                max_health: 80.0,
            },
            EnemyType::Fast => Enemy {
                detection_range: 12.0,
                attack_range: 1.5,
                move_speed: 5.0,
                health: 50.0,
                max_health: 50.0,
            },
            EnemyType::Heavy => Enemy {
                detection_range: 6.0,
                attack_range: 2.5,
                move_speed: 2.0,
                health: 150.0,
                max_health: 150.0,
            },
        };

        EnemyPhysicsBundle {
            rigid_body: self.get_rigid_body(),
            collider: self.get_collider(),
            mass: Mass(self.mass),
            restitution: Restitution::ZERO,
            friction: Friction::new(0.7), // More friction than players for different feel
            linear_damping: LinearDamping(1.5), // Higher damping for more controlled movement
            angular_damping: AngularDamping(10.0),
            locked_axes: LockedAxes::ROTATION_LOCKED.unlock_rotation_y(),
            enemy_bundle: EnemyBundle { enemy, ..default() },
        }
    }

    fn get_collider(&self) -> Collider {
        Collider::capsule(self.height, self.radius)
    }

    fn get_rigid_body(&self) -> RigidBody {
        RigidBody::Dynamic
    }
}

impl GameEntity for EnemyEntity {
    fn entity_type(&self) -> &'static str {
        match self.enemy_type {
            EnemyType::Basic => "Enemy_Basic",
            EnemyType::Fast => "Enemy_Fast",
            EnemyType::Heavy => "Enemy_Heavy",
        }
    }
}

impl Spawnable for EnemyEntity {
    fn get_spawn_offset(&self) -> Option<bevy::prelude::Vec3> {
        Some(bevy::prelude::Vec3::new(0.0, self.height + 0.1, 0.0))
    }
}
