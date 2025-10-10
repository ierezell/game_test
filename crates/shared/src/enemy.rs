use avian3d::prelude::{LinearVelocity, Position, Rotation};
use bevy::prelude::{
    App, Bundle, Component, Entity, FixedUpdate, MessageWriter, Plugin, Query, Res, Timer,
    TimerMode, Vec3, With, Without, info,
};
use std::time::Duration;

use crate::health::{DamageEvent, DamageType};
use crate::protocol::PlayerId;

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (
                enemy_patrol_system,
                enemy_chase_system,
                enemy_attack_system,
                enemy_state_management,
                enemy_attack_damage_system,
            ),
        );
    }
}

/// Enemy AI states
#[derive(Component, Debug, Clone, PartialEq)]
pub enum EnemyState {
    Patrol,
    Chase { target: Entity },
    Attack { target: Entity },
    Dead,
}

impl Default for EnemyState {
    fn default() -> Self {
        EnemyState::Patrol
    }
}

/// Enemy entity marker and data
#[derive(Component)]
pub struct Enemy {
    pub detection_range: f32,
    pub attack_range: f32,
    pub move_speed: f32,
    pub health: f32,
    pub max_health: f32,
}

impl Default for Enemy {
    fn default() -> Self {
        Self {
            detection_range: 10.0,
            attack_range: 2.0,
            move_speed: 3.0,
            health: 100.0,
            max_health: 100.0,
        }
    }
}

/// Patrol behavior data
#[derive(Component)]
pub struct PatrolBehavior {
    pub patrol_points: Vec<Vec3>,
    pub current_target_index: usize,
    pub wait_timer: Timer,
    pub patrol_speed: f32,
}

impl Default for PatrolBehavior {
    fn default() -> Self {
        Self {
            patrol_points: vec![
                Vec3::new(0.0, 0.0, 0.0),
                Vec3::new(5.0, 0.0, 0.0),
                Vec3::new(5.0, 0.0, 5.0),
                Vec3::new(0.0, 0.0, 5.0),
            ],
            current_target_index: 0,
            wait_timer: Timer::new(Duration::from_secs(2), TimerMode::Once),
            patrol_speed: 2.0,
        }
    }
}

/// Chase behavior data
#[derive(Component)]
pub struct ChaseBehavior {
    pub chase_speed: f32,
    pub lose_target_distance: f32,
    pub lose_target_timer: Timer,
}

impl Default for ChaseBehavior {
    fn default() -> Self {
        Self {
            chase_speed: 4.0,
            lose_target_distance: 15.0,
            lose_target_timer: Timer::new(Duration::from_secs(3), TimerMode::Once),
        }
    }
}

/// Attack behavior data
#[derive(Component)]
pub struct AttackBehavior {
    pub attack_cooldown: Timer,
    pub attack_damage: f32,
}

impl Default for AttackBehavior {
    fn default() -> Self {
        Self {
            attack_cooldown: Timer::new(Duration::from_secs(1), TimerMode::Repeating),
            attack_damage: 25.0,
        }
    }
}

/// Complete enemy bundle
#[derive(Bundle)]
pub struct EnemyBundle {
    pub enemy: Enemy,
    pub state: EnemyState,
    pub patrol: PatrolBehavior,
    pub chase: ChaseBehavior,
    pub attack: AttackBehavior,
    pub position: Position,
    pub rotation: Rotation,
    pub velocity: LinearVelocity,
}

impl Default for EnemyBundle {
    fn default() -> Self {
        Self {
            enemy: Enemy::default(),
            state: EnemyState::default(),
            patrol: PatrolBehavior::default(),
            chase: ChaseBehavior::default(),
            attack: AttackBehavior::default(),
            position: Position::default(),
            rotation: Rotation::default(),
            velocity: LinearVelocity::default(),
        }
    }
}

/// Patrol AI System
fn enemy_patrol_system(
    mut enemy_query: Query<
        (
            &mut Position,
            &mut LinearVelocity,
            &mut PatrolBehavior,
            &EnemyState,
        ),
        (With<Enemy>, Without<PlayerId>),
    >,
    time: bevy::prelude::Res<bevy::prelude::Time>,
) {
    for (position, mut velocity, mut patrol, state) in enemy_query.iter_mut() {
        if !matches!(state, EnemyState::Patrol) {
            continue;
        }

        if patrol.patrol_points.is_empty() {
            continue;
        }

        let target_point = patrol.patrol_points[patrol.current_target_index];
        let distance_to_target = position.0.distance(target_point);

        if distance_to_target < 1.0 {
            // Reached patrol point, wait and then move to next
            patrol.wait_timer.tick(time.delta());
            velocity.0 = Vec3::ZERO;

            if patrol.wait_timer.is_finished() {
                patrol.current_target_index =
                    (patrol.current_target_index + 1) % patrol.patrol_points.len();
                patrol.wait_timer.reset();
            }
        } else {
            // Move towards patrol point
            let direction = (target_point - position.0).normalize_or_zero();
            velocity.0 = direction * patrol.patrol_speed;
        }
    }
}

/// Chase AI System
fn enemy_chase_system(
    mut enemy_query: Query<
        (
            Entity,
            &mut Position,
            &mut LinearVelocity,
            &mut ChaseBehavior,
            &Enemy,
            &EnemyState,
        ),
        Without<PlayerId>,
    >,
    player_query: Query<(Entity, &Position), With<PlayerId>>,
    time: bevy::prelude::Res<bevy::prelude::Time>,
) {
    for (_enemy_entity, enemy_pos, mut velocity, mut chase, _enemy_data, state) in
        enemy_query.iter_mut()
    {
        if let EnemyState::Chase { target } = state {
            // Find the target player
            if let Ok((_, player_pos)) = player_query.get(*target) {
                let distance_to_player = enemy_pos.0.distance(player_pos.0);

                if distance_to_player <= chase.lose_target_distance {
                    // Chase the player
                    let direction = (player_pos.0 - enemy_pos.0).normalize_or_zero();
                    velocity.0 = direction * chase.chase_speed;
                    chase.lose_target_timer.reset();
                } else {
                    // Player too far, start lose target timer
                    chase.lose_target_timer.tick(time.delta());
                    velocity.0 = Vec3::ZERO;
                }
            } else {
                // Target no longer exists
                chase.lose_target_timer.tick(time.delta());
                velocity.0 = Vec3::ZERO;
            }
        }
    }
}

/// Attack AI System
fn enemy_attack_system(
    mut enemy_query: Query<
        (Entity, &Position, &mut AttackBehavior, &Enemy, &EnemyState),
        Without<PlayerId>,
    >,
    player_query: Query<(Entity, &Position), With<PlayerId>>,
    time: bevy::prelude::Res<bevy::prelude::Time>,
) {
    for (enemy_entity, enemy_pos, mut attack, enemy_data, state) in enemy_query.iter_mut() {
        attack.attack_cooldown.tick(time.delta());

        if let EnemyState::Attack { target } = state {
            if let Ok((_, player_pos)) = player_query.get(*target) {
                let distance_to_player = enemy_pos.0.distance(player_pos.0);

                if distance_to_player <= enemy_data.attack_range
                    && attack.attack_cooldown.is_finished()
                {
                    // Perform attack
                    info!(
                        "Enemy {:?} attacks player {:?} for {} damage!",
                        enemy_entity, target, attack.attack_damage
                    );
                    attack.attack_cooldown.reset();
                }
            }
        }
    }
}

/// State Management System - determines when to transition between states
fn enemy_state_management(
    mut enemy_query: Query<
        (
            Entity,
            &Position,
            &mut EnemyState,
            &mut ChaseBehavior,
            &Enemy,
        ),
        Without<PlayerId>,
    >,
    player_query: Query<(Entity, &Position), With<PlayerId>>,
) {
    for (enemy_entity, enemy_pos, mut state, chase, enemy_data) in enemy_query.iter_mut() {
        let closest_player = player_query
            .iter()
            .map(|(entity, pos)| (entity, enemy_pos.0.distance(pos.0)))
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

        // Clone the current state to avoid borrow checker issues
        let current_state = state.clone();

        match current_state {
            EnemyState::Patrol => {
                // Check if any player is within detection range
                if let Some((player_entity, distance)) = closest_player {
                    if distance <= enemy_data.detection_range {
                        *state = EnemyState::Chase {
                            target: player_entity,
                        };
                        info!(
                            "Enemy {:?} detected player {:?} and started chasing!",
                            enemy_entity, player_entity
                        );
                    }
                }
            }
            EnemyState::Chase { target } => {
                if let Some((player_entity, distance)) = closest_player {
                    if player_entity == target {
                        if distance <= enemy_data.attack_range {
                            *state = EnemyState::Attack { target };
                            info!(
                                "Enemy {:?} is now attacking player {:?}!",
                                enemy_entity, target
                            );
                        } else if distance > chase.lose_target_distance
                            && chase.lose_target_timer.is_finished()
                        {
                            *state = EnemyState::Patrol;
                            info!(
                                "Enemy {:?} lost player {:?} and returning to patrol!",
                                enemy_entity, target
                            );
                        }
                    }
                }
            }
            EnemyState::Attack { target } => {
                if let Some((player_entity, distance)) = closest_player {
                    if player_entity == target {
                        if distance > enemy_data.attack_range {
                            *state = EnemyState::Chase { target };
                            info!(
                                "Enemy {:?} target moved away, switching to chase!",
                                enemy_entity
                            );
                        }
                    }
                } else {
                    // Target no longer exists
                    *state = EnemyState::Patrol;
                    info!(
                        "Enemy {:?} target disappeared, returning to patrol!",
                        enemy_entity
                    );
                }
            }
            EnemyState::Dead => {
                // Dead enemies don't change state
            }
        }
    }
}

/// System to handle enemy attacks and deal damage to players
fn enemy_attack_damage_system(
    mut enemy_query: Query<(Entity, &Position, &mut AttackBehavior, &EnemyState), With<Enemy>>,
    player_query: Query<(Entity, &Position), (With<PlayerId>, Without<Enemy>)>,
    mut damage_events: MessageWriter<DamageEvent>,
    time: Res<bevy::prelude::Time>,
) {
    for (enemy_entity, enemy_pos, mut attack, state) in enemy_query.iter_mut() {
        // Only attack if in attack state
        if let EnemyState::Attack { target } = state {
            attack.attack_cooldown.tick(time.delta());

            // Check if we can attack (cooldown finished)
            if attack.attack_cooldown.is_finished() {
                // Find the target player
                if let Ok((player_entity, player_pos)) = player_query.get(*target) {
                    let distance = enemy_pos.0.distance(player_pos.0);

                    // Check if player is within attack range
                    if distance <= 2.5 {
                        // Attack range
                        // Deal damage to the player
                        damage_events.write(DamageEvent {
                            target: player_entity,
                            amount: attack.attack_damage,
                            source: Some(enemy_entity),
                            damage_type: DamageType::Physical,
                            ignore_invulnerability: false,
                        });

                        // Reset attack cooldown
                        attack.attack_cooldown.reset();

                        info!(
                            "Enemy {:?} attacked player {:?} for {:.1} damage at distance {:.2}",
                            enemy_entity, player_entity, attack.attack_damage, distance
                        );
                    }
                }
            }
        }
    }
}
