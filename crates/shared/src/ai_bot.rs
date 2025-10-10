use crate::entity_implementations::EnhancedPlayerPhysicsBundle;
use crate::health::Health;
use crate::input::PlayerAction;
use crate::protocol::{PlayerColor, PlayerId};
use crate::scene::color_from_id;
use avian3d::prelude::{LinearVelocity, Position, Rotation};
use bevy::prelude::*;
use leafwing_input_manager::prelude::ActionState;
use lightyear::prelude::*;
use rand::{Rng, thread_rng};
use std::time::Duration;

pub struct BotPlugin;

impl Plugin for BotPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (update_classic_bot_behavior, execute_classic_bot_actions).chain(),
        )
        .add_observer(spawn_bots_on_server_start);
    }
}

/// Component that marks an entity as an AI bot
#[derive(Component, Clone, Debug)]
pub struct AIBot {
    pub bot_id: u32,
    pub bot_type: BotType,
    pub behavior_state: BotBehaviorState,
    pub decision_timer: Timer,
    pub action_timer: Timer,
    pub target_player: Option<Entity>,
    pub last_seen_position: Option<Vec3>,
    pub reaction_time: f32,
    pub aggression_level: f32,
    pub accuracy: f32,
}

/// Different types of bots
#[derive(Clone, Debug, PartialEq)]
pub enum BotType {
    /// Classic autonomous AI bot that makes its own decisions
    Classic,
    /// External agent-controlled bot (for RL training, etc.)
    ExternalAgent,
}

impl Default for AIBot {
    fn default() -> Self {
        Self {
            bot_id: 0,
            bot_type: BotType::Classic,
            behavior_state: BotBehaviorState::Idle,
            decision_timer: Timer::new(Duration::from_millis(500), TimerMode::Repeating),
            action_timer: Timer::new(Duration::from_millis(100), TimerMode::Repeating),
            target_player: None,
            last_seen_position: None,
            reaction_time: 0.3,
            aggression_level: 0.7,
            accuracy: 0.6,
        }
    }
}

/// Different behavior states for the AI bot
#[derive(Clone, Debug, PartialEq)]
pub enum BotBehaviorState {
    Idle,
    Searching,
    Engaging(Entity), // Entity is the target player
    Retreating,
    Repositioning,
}

/// Get the current state of a bot for observation
#[derive(Debug, Clone)]
pub struct BotObservation {
    pub position: Vec3,
    pub rotation: Quat,
    pub velocity: Vec3,
    pub health: f32,
    pub max_health: f32,
    pub current_actions: Vec<PlayerAction>,
}

fn spawn_bots_on_server_start(
    _trigger: On<Add, lightyear::connection::server::Started>,
    mut commands: Commands,
) {
    info!("Spawning AI bots on server start");

    // Spawn 1 classic AI bot for autonomous gameplay
    spawn_classic_ai_bot(&mut commands, 0, Vec3::new(-10.0, 2.0, -10.0));

    // Spawn 1 external agent bot for RL training
    spawn_external_agent_bot(&mut commands, 1, Vec3::new(-5.0, 2.0, -10.0));
}

/// Spawn a classic AI bot that behaves autonomously
pub fn spawn_classic_ai_bot(commands: &mut Commands, bot_id: u32, position: Vec3) -> Entity {
    // Create a fake peer_id for the bot (using high values to avoid collision)
    let fake_peer_id = PeerId::Netcode(2000 + bot_id as u64); // Different range for classic bots
    let color = color_from_id(fake_peer_id.to_bits());

    info!(
        "Spawning Classic AI Bot {} at position {:?}",
        bot_id, position
    );

    let bot_entity = commands
        .spawn((
            // Bot identification
            AIBot {
                bot_id,
                bot_type: BotType::Classic,
                ..default()
            },
            // Same components as a regular player
            Name::new(format!("Classic_AI_Bot_{}", bot_id)),
            PlayerId(fake_peer_id),
            LinearVelocity::default(),
            Position(position),
            Rotation::default(),
            PlayerColor(color),
            // Health system
            Health::with_regeneration(100.0, 8.0, 6.0), // Slightly different regen for bots
            // Action state for bot input simulation
            ActionState::<PlayerAction>::default(),
            // Physics
            EnhancedPlayerPhysicsBundle::default(),
            // For multiplayer - replicate to all clients
            Replicate::to_clients(NetworkTarget::All),
            InterpolationTarget::to_clients(NetworkTarget::All),
        ))
        .id();

    // Add weapon holder to bot
    crate::weapons::add_weapon_holder(commands, bot_entity);

    // Add stamina system to bot
    crate::stamina::add_stamina_to_player(commands, bot_entity);

    info!(
        "Classic AI Bot {} spawned with entity ID: {:?}",
        bot_id, bot_entity
    );
    bot_entity
}

/// Spawn an external agent-controlled bot
pub fn spawn_external_agent_bot(commands: &mut Commands, bot_id: u32, position: Vec3) -> Entity {
    // Create a fake peer_id for the bot (using high values to avoid collision)
    let fake_peer_id = PeerId::Netcode(3000 + bot_id as u64); // Different range for external agent bots
    let color = color_from_id(fake_peer_id.to_bits());

    info!(
        "Spawning External Agent Bot {} at position {:?}",
        bot_id, position
    );

    let bot_entity = commands
        .spawn((
            // Bot identification
            AIBot {
                bot_id,
                bot_type: BotType::ExternalAgent,
                ..default()
            },
            // Same components as a regular player
            Name::new(format!("Agent_Bot_{}", bot_id)),
            PlayerId(fake_peer_id),
            LinearVelocity::default(),
            Position(position),
            Rotation::default(),
            PlayerColor(color),
            // Health system
            Health::with_regeneration(100.0, 8.0, 6.0),
            // Action state for external control - SAME ActionState<PlayerAction> as human players!
            // This allows external agents (RL, Python scripts, etc.) to control this bot
            // using the exact same action space as human players
            ActionState::<PlayerAction>::default(),
            // Physics
            EnhancedPlayerPhysicsBundle::default(),
            // For multiplayer - replicate to all clients
            Replicate::to_clients(NetworkTarget::All),
            InterpolationTarget::to_clients(NetworkTarget::All),
        ))
        .id();

    // Add weapon holder to bot
    crate::weapons::add_weapon_holder(commands, bot_entity);

    // Add stamina system to bot
    crate::stamina::add_stamina_to_player(commands, bot_entity);

    info!(
        "External Agent Bot {} spawned with entity ID: {:?}",
        bot_id, bot_entity
    );
    bot_entity
}

/// System to update classic bot AI behavior and decision making
fn update_classic_bot_behavior(
    mut bot_query: Query<(Entity, &mut AIBot, &Position, &Health), (With<AIBot>, With<PlayerId>)>,
    player_query: Query<(Entity, &Position, &Health), (With<PlayerId>, Without<AIBot>)>,
    time: Res<Time>,
) {
    for (_bot_entity, mut bot, bot_position, bot_health) in bot_query.iter_mut() {
        // Only process classic bots
        if bot.bot_type != BotType::Classic {
            continue;
        }

        bot.decision_timer.tick(time.delta());

        // Only make decisions periodically to simulate realistic reaction time
        if !bot.decision_timer.just_finished() {
            continue;
        }

        // Find nearest player
        let nearest_player = player_query
            .iter()
            .map(|(entity, pos, health)| (entity, pos, bot_position.0.distance(pos.0), health))
            .filter(|(_, _, distance, health)| *distance < 25.0 && health.current > 0.0) // Alive players within range
            .min_by(|a, b| a.2.partial_cmp(&b.2).unwrap());

        match nearest_player {
            Some((player_entity, player_pos, distance, player_health)) => {
                bot.target_player = Some(player_entity);
                bot.last_seen_position = Some(player_pos.0);

                // Decide behavior based on distance, health, and bot characteristics
                bot.behavior_state = match bot.behavior_state {
                    BotBehaviorState::Idle | BotBehaviorState::Searching => {
                        if distance < 15.0 {
                            BotBehaviorState::Engaging(player_entity)
                        } else {
                            BotBehaviorState::Searching
                        }
                    }
                    BotBehaviorState::Engaging(current_target) => {
                        // Check if we should retreat (low health)
                        if bot_health.current < 30.0 && player_health.current > 50.0 {
                            BotBehaviorState::Retreating
                        } else if distance > 20.0 {
                            BotBehaviorState::Searching
                        } else if current_target == player_entity {
                            BotBehaviorState::Engaging(player_entity)
                        } else {
                            // Switch to closer target
                            BotBehaviorState::Engaging(player_entity)
                        }
                    }
                    BotBehaviorState::Retreating => {
                        if bot_health.current > 70.0 && distance < 15.0 {
                            BotBehaviorState::Engaging(player_entity)
                        } else if distance > 20.0 {
                            BotBehaviorState::Searching
                        } else {
                            BotBehaviorState::Retreating
                        }
                    }
                    BotBehaviorState::Repositioning => {
                        if distance < 12.0 {
                            BotBehaviorState::Engaging(player_entity)
                        } else {
                            BotBehaviorState::Searching
                        }
                    }
                };

                debug!(
                    "Classic Bot {} behavior: {:?} (distance: {:.1})",
                    bot.bot_id, bot.behavior_state, distance
                );
            }
            None => {
                // No players in range
                bot.target_player = None;
                bot.behavior_state = BotBehaviorState::Idle;
            }
        }
    }
}

/// System to execute classic bot actions based on their behavior state
fn execute_classic_bot_actions(
    mut bot_query: Query<
        (
            Entity,
            &mut AIBot,
            &mut ActionState<PlayerAction>,
            &Position,
            &Rotation,
        ),
        (With<AIBot>, With<PlayerId>),
    >,
    player_query: Query<&Position, (With<PlayerId>, Without<AIBot>)>,
    time: Res<Time>,
) {
    for (_bot_entity, mut bot, mut action_state, bot_position, bot_rotation) in bot_query.iter_mut()
    {
        // Only process classic bots
        if bot.bot_type != BotType::Classic {
            continue;
        }

        bot.action_timer.tick(time.delta());

        // Clear previous actions - reset dual axis to zero and release buttons
        action_state.set_axis_pair(&PlayerAction::Move, Vec2::ZERO);
        action_state.set_axis_pair(&PlayerAction::Look, Vec2::ZERO);

        for action in [PlayerAction::Jump, PlayerAction::Shoot] {
            if action_state.pressed(&action) {
                action_state.release(&action);
            }
        }

        // Execute actions based on behavior state
        match &bot.behavior_state {
            BotBehaviorState::Idle => {
                // Random idle movement occasionally
                if thread_rng().r#gen::<f32>() < 0.1 {
                    let random_movement = Vec2::new(
                        thread_rng().gen_range(-0.3..0.3),
                        thread_rng().gen_range(-0.3..0.3),
                    );
                    action_state.set_axis_pair(&PlayerAction::Move, random_movement);
                }
            }

            BotBehaviorState::Searching => {
                if let Some(last_pos) = bot.last_seen_position {
                    // Move towards last seen position
                    let direction = (last_pos - bot_position.0).normalize_or_zero();
                    let movement = Vec2::new(direction.x, direction.z);
                    action_state.set_axis_pair(&PlayerAction::Move, movement.clamp_length_max(1.0));
                }
            }

            BotBehaviorState::Engaging(target_entity) => {
                if let Ok(target_pos) = player_query.get(*target_entity) {
                    let to_target = target_pos.0 - bot_position.0;
                    let distance = to_target.length();
                    let direction = to_target.normalize_or_zero();

                    // Movement logic
                    let movement = if distance > 10.0 {
                        // Move closer
                        Vec2::new(direction.x, direction.z)
                    } else if distance < 5.0 {
                        // Too close, back away while maintaining engagement
                        Vec2::new(-direction.x * 0.5, -direction.z * 0.5)
                    } else {
                        // Optimal range, strafe
                        let strafe = Vec2::new(-direction.z, direction.x); // Perpendicular vector
                        strafe
                            * if thread_rng().r#gen::<bool>() {
                                1.0
                            } else {
                                -1.0
                            }
                    };

                    action_state.set_axis_pair(&PlayerAction::Move, movement.clamp_length_max(1.0));

                    // Look towards target
                    let look_direction =
                        calculate_look_input(bot_position.0, target_pos.0, bot_rotation);
                    if let Some(look) = look_direction {
                        action_state.set_axis_pair(&PlayerAction::Look, look);
                    }

                    // Shoot with some accuracy and reaction time
                    if distance < 15.0 && bot.action_timer.just_finished() {
                        if thread_rng().r#gen::<f32>() < bot.accuracy {
                            action_state.press(&PlayerAction::Shoot);
                        }
                    }
                } else {
                    // Target lost
                    bot.behavior_state = BotBehaviorState::Searching;
                }
            }

            BotBehaviorState::Retreating => {
                if let Some(last_pos) = bot.last_seen_position {
                    // Move away from last seen position
                    let direction = (bot_position.0 - last_pos).normalize_or_zero();
                    let movement = Vec2::new(direction.x, direction.z);
                    action_state.set_axis_pair(&PlayerAction::Move, movement.clamp_length_max(1.0));
                }
            }

            BotBehaviorState::Repositioning => {
                // Move to a tactical position (simplified - just move perpendicular)
                if let Some(last_pos) = bot.last_seen_position {
                    let to_target = (last_pos - bot_position.0).normalize_or_zero();
                    let perpendicular = Vec2::new(-to_target.z, to_target.x);
                    action_state
                        .set_axis_pair(&PlayerAction::Move, perpendicular.clamp_length_max(1.0));
                }
            }
        }
    }
}

/// Calculate look input to aim at target
fn calculate_look_input(bot_pos: Vec3, target_pos: Vec3, bot_rotation: &Rotation) -> Option<Vec2> {
    let to_target = target_pos - bot_pos;
    let target_yaw = to_target.z.atan2(to_target.x);

    // Get current yaw from rotation
    let current_yaw = bot_rotation.0.to_euler(EulerRot::YXZ).0;

    // Calculate yaw difference
    let mut yaw_diff = target_yaw - current_yaw;

    // Normalize angle difference to [-PI, PI]
    while yaw_diff > std::f32::consts::PI {
        yaw_diff -= 2.0 * std::f32::consts::PI;
    }
    while yaw_diff < -std::f32::consts::PI {
        yaw_diff += 2.0 * std::f32::consts::PI;
    }

    // Convert to mouse delta (with some sensitivity scaling)
    if yaw_diff.abs() > 0.1 {
        Some(Vec2::new(yaw_diff * 2.0, 0.0)) // Only horizontal look for now
    } else {
        None
    }
}

/// Helper function to add a bot to an existing entity
pub fn add_ai_bot_component(commands: &mut Commands, entity: Entity, bot_id: u32) {
    commands.entity(entity).insert((
        AIBot {
            bot_id,
            ..default()
        },
        ActionState::<PlayerAction>::default(),
    ));
}

/// External Agent Control API - these functions allow external systems to control agent bots

/// Set movement input for an external agent bot
pub fn set_external_agent_movement(
    mut bot_query: Query<(&AIBot, &mut ActionState<PlayerAction>), With<PlayerId>>,
    bot_id: u32,
    movement: Vec2,
) {
    for (bot, mut action_state) in bot_query.iter_mut() {
        if bot.bot_type == BotType::ExternalAgent && bot.bot_id == bot_id {
            action_state.set_axis_pair(&PlayerAction::Move, movement.clamp_length_max(1.0));
            break;
        }
    }
}

/// Set look input for an external agent bot
pub fn set_external_agent_look(
    mut bot_query: Query<(&AIBot, &mut ActionState<PlayerAction>), With<PlayerId>>,
    bot_id: u32,
    look_delta: Vec2,
) {
    for (bot, mut action_state) in bot_query.iter_mut() {
        if bot.bot_type == BotType::ExternalAgent && bot.bot_id == bot_id {
            action_state.set_axis_pair(&PlayerAction::Look, look_delta);
            break;
        }
    }
}

/// Set shoot action for an external agent bot
pub fn set_external_agent_shoot(
    mut bot_query: Query<(&AIBot, &mut ActionState<PlayerAction>), With<PlayerId>>,
    bot_id: u32,
    should_shoot: bool,
) {
    for (bot, mut action_state) in bot_query.iter_mut() {
        if bot.bot_type == BotType::ExternalAgent && bot.bot_id == bot_id {
            if should_shoot {
                action_state.press(&PlayerAction::Shoot);
            } else {
                action_state.release(&PlayerAction::Shoot);
            }
            break;
        }
    }
}

/// Set jump action for an external agent bot
pub fn set_external_agent_jump(
    mut bot_query: Query<(&AIBot, &mut ActionState<PlayerAction>), With<PlayerId>>,
    bot_id: u32,
    should_jump: bool,
) {
    for (bot, mut action_state) in bot_query.iter_mut() {
        if bot.bot_type == BotType::ExternalAgent && bot.bot_id == bot_id {
            if should_jump {
                action_state.press(&PlayerAction::Jump);
            } else {
                action_state.release(&PlayerAction::Jump);
            }
            break;
        }
    }
}

/// Set all actions for an external agent bot at once (convenience function)
pub fn set_external_agent_actions(
    mut bot_query: Query<(&AIBot, &mut ActionState<PlayerAction>), With<PlayerId>>,
    bot_id: u32,
    movement: Vec2,
    look_delta: Vec2,
    should_shoot: bool,
    should_jump: bool,
) {
    for (bot, mut action_state) in bot_query.iter_mut() {
        if bot.bot_type == BotType::ExternalAgent && bot.bot_id == bot_id {
            // Set movement and look
            action_state.set_axis_pair(&PlayerAction::Move, movement.clamp_length_max(1.0));
            action_state.set_axis_pair(&PlayerAction::Look, look_delta);

            // Set discrete actions
            if should_shoot {
                action_state.press(&PlayerAction::Shoot);
            } else {
                action_state.release(&PlayerAction::Shoot);
            }

            if should_jump {
                action_state.press(&PlayerAction::Jump);
            } else {
                action_state.release(&PlayerAction::Jump);
            }
            break;
        }
    }
}

/// Get observation data for a bot
pub fn get_bot_observation(
    bot_query: Query<
        (
            &AIBot,
            &Position,
            &Rotation,
            &LinearVelocity,
            &Health,
            &ActionState<PlayerAction>,
        ),
        With<PlayerId>,
    >,
    bot_id: u32,
) -> Option<BotObservation> {
    for (bot, position, rotation, velocity, health, action_state) in bot_query.iter() {
        if bot.bot_id == bot_id {
            return Some(BotObservation {
                position: position.0,
                rotation: rotation.0,
                velocity: velocity.0,
                health: health.current,
                max_health: health.max,
                current_actions: action_state.get_pressed().into_iter().collect(),
            });
        }
    }
    None
}

/// Get all external agent bot IDs
pub fn get_external_agent_bot_ids(bot_query: Query<&AIBot, With<PlayerId>>) -> Vec<u32> {
    bot_query
        .iter()
        .filter(|bot| bot.bot_type == BotType::ExternalAgent)
        .map(|bot| bot.bot_id)
        .collect()
}
