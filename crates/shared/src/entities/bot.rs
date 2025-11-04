use crate::entities::health::Health;
use crate::entities::player::{PlayerPhysicsBundle, color_from_id};
use crate::input::PlayerAction;
use crate::protocol::{PlayerColor, PlayerId};
use avian3d::prelude::{LinearVelocity, Position, Rotation};
use bevy::prelude::*;
use leafwing_input_manager::prelude::ActionState;
use lightyear::prelude::*;

use std::time::Duration;

pub struct BotPlugin;

impl Plugin for BotPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, update_bot_behavior)
            .add_observer(spawn_bots_on_server_start);
    }
}

#[derive(Component, Clone, Debug)]
pub struct Bot {
    pub behavior_state: BotBehaviorState,
    pub decision_timer: Timer,
    pub action_timer: Timer,
    pub target_player: Option<Entity>,
    pub last_seen_position: Option<Vec3>,
    pub reaction_time: f32,
    pub aggression_level: f32,
    pub accuracy: f32,
}

impl Default for Bot {
    fn default() -> Self {
        Self {
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

/// Different behavior states for the bot
#[derive(Clone, Debug, PartialEq)]
pub enum BotBehaviorState {
    Idle,
    Searching,
    Engaging(Entity), // Entity is the target player
    Retreating,
    Repositioning,
}

fn spawn_bots_on_server_start(
    _trigger: On<Add, lightyear::connection::server::Started>,
    mut commands: Commands,
) {
    spawn_classic_ai_bot(&mut commands, 0, Vec3::new(-10.0, 2.0, -10.0));
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
            PlayerPhysicsBundle::default(),
            // For multiplayer - replicate to all clients
            Replicate::to_clients(NetworkTarget::All),
            InterpolationTarget::to_clients(NetworkTarget::All),
        ))
        .id();

    // Add weapon holder to bot

    // Add weapon holder to bot
    crate::entities::weapons::add_weapon_holder(&mut *commands, bot_entity);

    // Add stamina system to bot
    crate::entities::stamina::add_stamina_to_player(&mut *commands, bot_entity);

    info!(
        "Classic AI Bot {} spawned with entity ID: {:?}",
        bot_id, bot_entity
    );
    bot_entity
}

fn update_bot_behavior(
    _commands: Commands,
    time: Res<Time>,
    mut bot_query: Query<(Entity, &mut Bot, &Position, &mut ActionState<PlayerAction>)>,
    player_query: Query<(Entity, &Position), With<PlayerId>>,
) {
    // Parameters for bot behavior
    let follow_distance = 2.0; // Distance to start attacking
    let move_speed = 4.0; // Units per second

    for (_, mut bot, bot_pos, mut action_state) in bot_query.iter_mut() {
        bot.decision_timer.tick(time.delta());
        bot.action_timer.tick(time.delta());

        // Find nearest player
        let mut nearest_player: Option<(Entity, Vec3, f32)> = None;
        for (player_entity, player_pos) in player_query.iter() {
            let dist = bot_pos.0.distance(player_pos.0);
            if nearest_player.is_none() || dist < nearest_player.unwrap().2 {
                nearest_player = Some((player_entity, player_pos.0, dist));
            }
        }

        if let Some((target_entity, target_pos, dist)) = nearest_player {
            // If close enough, attack
            if dist < follow_distance {
                // Simulate attack action
                action_state.press(&PlayerAction::Shoot);
                bot.behavior_state = BotBehaviorState::Engaging(target_entity);
                // Stop moving
                action_state.set_axis_pair(&PlayerAction::Move, Vec2::ZERO);
            } else {
                // Move toward player
                let direction = (target_pos - bot_pos.0).normalize_or_zero();
                let move_vec = Vec2::new(direction.x, direction.z) * move_speed * time.delta_secs();
                action_state.release(&PlayerAction::Shoot);
                action_state.set_axis_pair(&PlayerAction::Move, move_vec);
                bot.behavior_state = BotBehaviorState::Searching;
            }
            bot.target_player = Some(target_entity);
            bot.last_seen_position = Some(target_pos);
        } else {
            // No player found, idle
            action_state.release(&PlayerAction::Shoot);
            action_state.set_axis_pair(&PlayerAction::Move, Vec2::ZERO);
            bot.behavior_state = BotBehaviorState::Idle;
            bot.target_player = None;
        }
    }
}
