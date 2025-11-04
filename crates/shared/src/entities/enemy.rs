use crate::navigation_pathfinding::{NavigationAgent, TargetSeeker};
use crate::protocol::PlayerId;
use avian3d::prelude::{LinearVelocity, Position};
use bevy::prelude::*;

/// Minimal enemy component
#[derive(Component, Clone, Debug)]
pub struct Enemy;

/// Component for enemies that should attack players when close enough
#[derive(Component, Debug, Clone)]
pub struct EnemyAttacker {
    /// Attack range in units
    pub attack_range: f32,
    /// Attack damage
    pub damage: f32,
    /// Time between attacks
    pub attack_cooldown: f32,
    /// Time since last attack
    pub last_attack_time: f32,
}

impl Default for EnemyAttacker {
    fn default() -> Self {
        Self {
            attack_range: 2.0,
            damage: 10.0,
            attack_cooldown: 1.0,
            last_attack_time: 0.0,
        }
    }
}

impl EnemyAttacker {
    pub fn new(damage: f32, attack_range: f32) -> Self {
        Self {
            damage,
            attack_range,
            ..Default::default()
        }
    }

    pub fn can_attack(&self) -> bool {
        self.last_attack_time >= self.attack_cooldown
    }

    pub fn attack(&mut self) {
        self.last_attack_time = 0.0;
    }

    pub fn update_cooldown(&mut self, delta_time: f32) {
        self.last_attack_time += delta_time;
    }
}

/// System for enemy behavior using navigation
pub fn enemy_navigation_behavior(
    mut enemy_query: Query<
        (Entity, &Position, &mut TargetSeeker),
        (With<Enemy>, Without<PlayerId>),
    >,
    player_query: Query<(Entity, &Position), With<PlayerId>>,
) {
    for (enemy_entity, enemy_pos, mut seeker) in enemy_query.iter_mut() {
        // Find the closest player
        let mut closest_player: Option<Entity> = None;
        let mut closest_distance = f32::INFINITY;

        for (player_entity, player_pos) in player_query.iter() {
            let distance = enemy_pos.0.distance(player_pos.0);
            if distance < closest_distance {
                closest_distance = distance;
                closest_player = Some(player_entity);
            }
        }

        // Update target seeker with closest player
        if let Some(target) = closest_player {
            if seeker.target != Some(target) {
                seeker.target = Some(target);
                debug!("Enemy {:?} targeting player {:?}", enemy_entity, target);
            }
        } else {
            seeker.target = None;
        }
    }
}

/// System for enemy attack behavior
pub fn enemy_attack_behavior(
    mut enemy_query: Query<
        (&Position, &mut EnemyAttacker, &NavigationAgent),
        (With<Enemy>, Without<PlayerId>),
    >,
    player_query: Query<&Position, With<PlayerId>>,
    time: Res<Time>,
) {
    for (enemy_pos, mut attacker, agent) in enemy_query.iter_mut() {
        attacker.update_cooldown(time.delta_secs());

        // Check if enemy has reached destination (close to player)
        if agent.has_reached_destination() && attacker.can_attack() {
            // Find any player within attack range
            for player_pos in player_query.iter() {
                let distance = enemy_pos.0.distance(player_pos.0);
                if distance <= attacker.attack_range {
                    attacker.attack();
                    info!("Enemy attacking player! Damage: {}", attacker.damage);
                    // Here you could send damage events or apply damage directly
                    break;
                }
            }
        }
    }
}

/// System for basic enemy patrol and attack (legacy - kept for compatibility)
pub fn enemy_behavior(
    mut query: Query<(&Position, &mut LinearVelocity), With<Enemy>>,
    player_query: Query<&Position, With<PlayerId>>,
) {
    for (pos, mut vel) in query.iter_mut() {
        // Find the first player (for simplicity)
        if let Some(player_pos) = player_query.iter().next() {
            let to_player = player_pos.0 - pos.0;
            let dist = to_player.length();
            if dist > 2.0 {
                // Move toward player
                vel.0 = to_player.normalize() * 2.0;
            } else {
                // Attack: stop and (in a real system) deal damage
                vel.0 = Vec3::ZERO;
                // Damage logic would go here
            }
        } else {
            // No player: stand still
            vel.0 = Vec3::ZERO;
        }
    }
}

/// Helper function to create an enemy with navigation capabilities
pub fn create_navigation_enemy(commands: &mut Commands, position: Vec3, _speed: f32) -> Entity {
    commands
        .spawn((
            Enemy,
            Position(position),
            TargetSeeker::default().with_update_interval(1.0),
            EnemyAttacker::new(15.0, 2.5),
            Name::new("NavigationEnemy"),
        ))
        .id()
}
