use crate::entities::enemy::{Enemy, EnemyAttacker};
use crate::entities::health::Health;
use crate::navigation_pathfinding::TargetSeeker;
use crate::navigation_pathfinding::add_navigation_agent_with_speed;
use avian3d::prelude::Position;
use bevy::prelude::{Commands, Name, Vec3, info};
use lightyear::prelude::{InterpolationTarget, NetworkTarget, Replicate};

pub fn spawn_enemies(mut commands: Commands) {
    info!("Spawning enemies on server start");
    let enemy_positions = [
        Vec3::new(8.0, 1.0, 8.0),
        Vec3::new(-8.0, 1.0, 8.0),
        Vec3::new(8.0, 1.0, -8.0),
    ];

    for position in &enemy_positions {
        let entity_id = commands
            .spawn((
                Position(*position),
                Name::new("Enemy"),
                Health::no_regeneration(80.0),
                Replicate::to_clients(NetworkTarget::All),
                InterpolationTarget::to_clients(NetworkTarget::All),
            ))
            .id();

        // Add navigation agent capabilities to enemy
        add_navigation_agent_with_speed(&mut commands, entity_id, 5.0);

        // Add enemy-specific components using our new helper function
        commands.entity(entity_id).insert((
            Enemy,
            TargetSeeker::default().with_update_interval(1.0),
            EnemyAttacker::new(15.0, 2.5),
        ));
    }

    info!("Spawned {} enemies with navigation", enemy_positions.len());
}
