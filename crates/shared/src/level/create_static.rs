use avian3d::prelude::{Position, Rotation};
use bevy::prelude::{Commands, Component, Name, Vec3, info};
use lightyear::prelude::{NetworkTarget, Replicate};
use rand::rngs::StdRng;
use rand::SeedableRng;
use serde::{Deserialize, Serialize};

pub const FLOOR_THICKNESS: f32 = 1.0;
pub const WALL_THICKNESS: f32 = 1.0;
pub const WALL_HEIGHT: f32 = 10.0;
pub const ROOM_SIZE: f32 = 50.0;

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct LevelDoneMarker;

pub fn setup_static_level(mut commands: Commands, seed: Option<u64>) {
    let seed = seed.unwrap_or(42); // Default seed if none provided
    info!("Setting up static level with seed: {}", seed);

    // Use seed for procedural generation
    let mut _rng = StdRng::seed_from_u64(seed);

    // Floor
    commands.spawn((
        Name::new("Floor"),
        Position(Vec3::new(0.0, -FLOOR_THICKNESS / 2.0, 0.0)),
        Rotation::default(),
        Replicate::to_clients(NetworkTarget::All),
    ));

    // Walls - could be procedurally varied based on seed
    let wall_positions = [
        (
            Vec3::new(
                ROOM_SIZE / 2.0 + WALL_THICKNESS / 2.0,
                WALL_HEIGHT / 2.0,
                0.0,
            ),
            "Wall East",
        ),
        (
            Vec3::new(
                -ROOM_SIZE / 2.0 - WALL_THICKNESS / 2.0,
                WALL_HEIGHT / 2.0,
                0.0,
            ),
            "Wall West",
        ),
        (
            Vec3::new(
                0.0,
                WALL_HEIGHT / 2.0,
                ROOM_SIZE / 2.0 + WALL_THICKNESS / 2.0,
            ),
            "Wall North",
        ),
        (
            Vec3::new(
                0.0,
                WALL_HEIGHT / 2.0,
                -ROOM_SIZE / 2.0 - WALL_THICKNESS / 2.0,
            ),
            "Wall South",
        ),
    ];

    for (position, name) in wall_positions {
        commands.spawn((
            Name::new(name),
            Position(position),
            Rotation::default(),
            Replicate::to_clients(NetworkTarget::All),
        ));
    }

    info!("Scene setup complete with seed: {}", seed);
    commands.spawn((
        LevelDoneMarker, 
        Name::new("Level"),
        Replicate::to_clients(NetworkTarget::All),
    ));
}

// Convenience function that uses default parameters for existing code
pub fn setup_static_level_default(commands: Commands) {
    setup_static_level(commands, None);
}
