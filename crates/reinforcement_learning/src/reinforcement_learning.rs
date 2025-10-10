use avian3d::prelude::{LinearVelocity, Position, Rotation};
use bevy::prelude::*;
use leafwing_input_manager::prelude::ActionState;
use nalgebra::{DMatrix, DVector};
use rand::{Rng, thread_rng};
use shared::ai_bot::{AIBot, BotObservation, BotType};
use shared::health::Health;
use shared::input::PlayerAction;
use shared::protocol::PlayerId;
use std::collections::{HashMap, VecDeque};

pub struct RLPlugin;

impl Plugin for RLPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(RLTrainingState::default()).add_systems(
            FixedUpdate,
            (collect_rl_observations, train_rl_agent, apply_rl_actions).chain(),
        );
    }
}

/// Simple neural network using nalgebra
#[derive(Clone, Debug)]
pub struct SimpleNetwork {
    weights1: DMatrix<f32>,
    bias1: DVector<f32>,
    weights2: DMatrix<f32>,
    bias2: DVector<f32>,
}

impl SimpleNetwork {
    /// Create a new simple network
    pub fn new(input_size: usize, hidden_size: usize, output_size: usize) -> Self {
        let mut rng = thread_rng();

        // Xavier initialization
        let w1_scale = (2.0 / (input_size + hidden_size) as f32).sqrt();
        let w2_scale = (2.0 / (hidden_size + output_size) as f32).sqrt();

        let weights1 = DMatrix::from_fn(hidden_size, input_size, |_, _| {
            rng.gen_range(-w1_scale..w1_scale)
        });
        let bias1 = DVector::zeros(hidden_size);

        let weights2 = DMatrix::from_fn(output_size, hidden_size, |_, _| {
            rng.gen_range(-w2_scale..w2_scale)
        });
        let bias2 = DVector::zeros(output_size);

        Self {
            weights1,
            bias1,
            weights2,
            bias2,
        }
    }

    /// Forward pass through the network
    pub fn forward(&self, input: &DVector<f32>) -> DVector<f32> {
        // First layer with ReLU activation
        let hidden = &self.weights1 * input + &self.bias1;
        let activated = hidden.map(|x| x.max(0.0)); // ReLU

        // Output layer
        &self.weights2 * &activated + &self.bias2
    }

    /// Get Q-values for state
    pub fn q_values(&self, state: &DVector<f32>) -> DVector<f32> {
        self.forward(state)
    }
}

/// Experience replay buffer entry
#[derive(Clone, Debug)]
pub struct Experience {
    pub state: Vec<f32>,
    pub action: PlayerActionSet,
    pub reward: f32,
    pub next_state: Vec<f32>,
    pub done: bool,
}

/// Simple RL training state using nalgebra
#[derive(Resource)]
pub struct RLTrainingState {
    pub q_network: Option<SimpleNetwork>,
    pub target_network: Option<SimpleNetwork>,
    pub experience_buffer: VecDeque<Experience>,
    pub training_enabled: bool,
    pub epsilon: f32,
    pub epsilon_decay: f32,
    pub epsilon_min: f32,
    pub learning_rate: f32,
    pub discount_factor: f32,
    pub batch_size: usize,
    pub buffer_size: usize,
    pub target_update_frequency: usize,
    pub training_step: usize,
    pub episode_count: usize,
    pub last_observations: HashMap<u32, BotObservation>,
    pub last_actions: HashMap<u32, PlayerActionSet>,
}

/// Player action representation for RL
#[derive(Clone, Debug)]
pub struct PlayerActionSet {
    pub movement: Vec2,
    pub look: Vec2,
    pub jump: bool,
    pub shoot: bool,
}

impl PlayerActionSet {
    /// Convert to vector for neural network
    pub fn to_vector(&self) -> Vec<f32> {
        vec![
            self.movement.x,
            self.movement.y,
            self.look.x,
            self.look.y,
            if self.jump { 1.0 } else { 0.0 },
            if self.shoot { 1.0 } else { 0.0 },
        ]
    }

    /// Create from vector
    pub fn from_vector(data: &[f32]) -> Self {
        Self {
            movement: Vec2::new(data[0].clamp(-1.0, 1.0), data[1].clamp(-1.0, 1.0)),
            look: Vec2::new(data[2].clamp(-1.0, 1.0), data[3].clamp(-1.0, 1.0)),
            jump: data[4] > 0.5,
            shoot: data[5] > 0.5,
        }
    }

    /// Apply to ActionState - this is the key interface with player actions
    pub fn apply_to_action_state(&self, action_state: &mut ActionState<PlayerAction>) {
        // Clear previous actions first
        action_state.set_axis_pair(&PlayerAction::Move, Vec2::ZERO);
        action_state.set_axis_pair(&PlayerAction::Look, Vec2::ZERO);
        action_state.release(&PlayerAction::Jump);
        action_state.release(&PlayerAction::Shoot);

        // Apply new actions
        action_state.set_axis_pair(&PlayerAction::Move, self.movement);
        action_state.set_axis_pair(&PlayerAction::Look, self.look);

        if self.jump {
            action_state.press(&PlayerAction::Jump);
        }

        if self.shoot {
            action_state.press(&PlayerAction::Shoot);
        }
    }

    /// Create from ActionState for observation
    pub fn from_action_state(action_state: &ActionState<PlayerAction>) -> Self {
        Self {
            movement: action_state.axis_pair(&PlayerAction::Move),
            look: action_state.axis_pair(&PlayerAction::Look),
            jump: action_state.pressed(&PlayerAction::Jump),
            shoot: action_state.pressed(&PlayerAction::Shoot),
        }
    }
}

impl Default for RLTrainingState {
    fn default() -> Self {
        Self {
            q_network: None,
            target_network: None,
            experience_buffer: VecDeque::new(),
            training_enabled: true,
            epsilon: 1.0,
            epsilon_decay: 0.995,
            epsilon_min: 0.01,
            learning_rate: 0.001,
            discount_factor: 0.99,
            batch_size: 32,
            buffer_size: 10000,
            target_update_frequency: 100,
            training_step: 0,
            episode_count: 0,
            last_observations: HashMap::new(),
            last_actions: HashMap::new(),
        }
    }
}

impl RLTrainingState {
    /// Initialize the RL networks
    pub fn initialize(&mut self) {
        let input_size = 12; // Position(3) + Rotation(4) + Velocity(3) + Health(2)
        let hidden_size = 64;
        let output_size = 6; // Move(2) + Look(2) + Jump(1) + Shoot(1)

        self.q_network = Some(SimpleNetwork::new(input_size, hidden_size, output_size));
        self.target_network = Some(SimpleNetwork::new(input_size, hidden_size, output_size));

        info!("RL Agent initialized with simple neural network");
    }

    /// Convert bot observation to state vector
    pub fn observation_to_state(&self, obs: &BotObservation) -> Vec<f32> {
        let mut state = Vec::with_capacity(12);

        // Position normalized to [-1, 1] range (assuming game world is ~20x20)
        state.extend_from_slice(&[
            obs.position.x / 10.0,
            obs.position.y / 10.0,
            obs.position.z / 10.0,
        ]);

        // Rotation as quaternion (already normalized)
        state.extend_from_slice(&[
            obs.rotation.x,
            obs.rotation.y,
            obs.rotation.z,
            obs.rotation.w,
        ]);

        // Velocity normalized
        state.extend_from_slice(&[
            obs.velocity.x / 10.0,
            obs.velocity.y / 10.0,
            obs.velocity.z / 10.0,
        ]);

        // Health normalized
        state.push(obs.health / obs.max_health);
        state.push(obs.max_health / 100.0);

        state
    }

    /// Convert neural network output to player actions
    pub fn vector_to_action(&self, output: &DVector<f32>) -> PlayerActionSet {
        let data = output.as_slice();
        PlayerActionSet::from_vector(data)
    }

    /// Calculate reward based on game state and actions
    pub fn calculate_reward(
        &self,
        current_obs: &BotObservation,
        previous_obs: Option<&BotObservation>,
        actions: &PlayerActionSet,
    ) -> f32 {
        let mut reward = 0.0;

        if let Some(prev_obs) = previous_obs {
            // Health-based rewards
            let health_diff = current_obs.health - prev_obs.health;
            reward += health_diff * 0.5;

            // Death penalty
            if current_obs.health <= 0.0 {
                reward -= 50.0;
                return reward;
            }

            // Survival bonus
            reward += 0.1;

            // Movement rewards
            let movement_magnitude = current_obs.velocity.length();
            if movement_magnitude > 0.1 {
                reward += 0.05;
            }

            // Boundary penalty
            let distance_from_center = current_obs.position.length();
            if distance_from_center > 15.0 {
                reward -= 0.1 * (distance_from_center - 15.0);
            }

            // Action efficiency
            let action_intensity = actions.movement.length()
                + actions.look.length()
                + if actions.jump { 1.0 } else { 0.0 }
                + if actions.shoot { 1.0 } else { 0.0 };
            if action_intensity > 4.0 {
                reward -= 0.01 * (action_intensity - 4.0);
            }
        }

        reward
    }

    /// Add experience to replay buffer
    pub fn add_experience(&mut self, experience: Experience) {
        self.experience_buffer.push_back(experience);

        // Remove old experiences if buffer is full
        while self.experience_buffer.len() > self.buffer_size {
            self.experience_buffer.pop_front();
        }
    }

    /// Get action from Q-network with epsilon-greedy exploration
    pub fn get_action(&self, state: &[f32]) -> PlayerActionSet {
        if thread_rng().r#gen::<f32>() < self.epsilon {
            // Random exploration
            let mut rng = thread_rng();
            PlayerActionSet {
                movement: Vec2::new(rng.gen_range(-1.0..1.0), rng.gen_range(-1.0..1.0)),
                look: Vec2::new(rng.gen_range(-1.0..1.0), rng.gen_range(-1.0..1.0)),
                jump: rng.r#gen::<f32>() < 0.1,
                shoot: rng.r#gen::<f32>() < 0.2,
            }
        } else {
            // Use Q-network to predict action
            if let Some(network) = &self.q_network {
                let input = DVector::from_vec(state.to_vec());
                let output = network.forward(&input);
                self.vector_to_action(&output)
            } else {
                // Fallback to no action
                PlayerActionSet {
                    movement: Vec2::ZERO,
                    look: Vec2::ZERO,
                    jump: false,
                    shoot: false,
                }
            }
        }
    }

    /// Simple training step (placeholder for proper training implementation)
    pub fn train_step(&mut self) {
        if self.experience_buffer.len() < self.batch_size {
            return;
        }

        // In a full implementation, this would implement proper DQN training
        // with loss calculation and gradient descent using nalgebra
        // For now, just update the step counter
        self.training_step += 1;
    }

    /// Update target network (copy from main network)
    pub fn update_target_network(&mut self) {
        if let Some(q_network) = &self.q_network {
            self.target_network = Some(q_network.clone());
        }
    }

    /// Update exploration rate
    pub fn update_epsilon(&mut self) {
        self.epsilon = (self.epsilon * self.epsilon_decay).max(self.epsilon_min);
    }

    /// Set training enabled/disabled
    pub fn set_training_enabled(&mut self, enabled: bool) {
        self.training_enabled = enabled;
        info!(
            "RL Training {}",
            if enabled { "enabled" } else { "disabled" }
        );
    }

    /// Reset training state
    pub fn reset_training(&mut self) {
        self.experience_buffer.clear();
        self.last_observations.clear();
        self.last_actions.clear();
        self.training_step = 0;
        self.episode_count = 0;
        self.epsilon = 1.0;
        self.initialize();
        info!("RL Training state reset");
    }
}

/// System to collect observations and train RL agents
fn collect_rl_observations(
    bot_query: Query<(&AIBot, &Position, &Rotation, &LinearVelocity, &Health), With<PlayerId>>,
    mut rl_state: ResMut<RLTrainingState>,
) {
    // Initialize RL agent if not done
    if rl_state.q_network.is_none() {
        rl_state.initialize();
    }

    for (bot, position, rotation, velocity, health) in bot_query.iter() {
        // Only process external agent bots for RL training
        if bot.bot_type != BotType::ExternalAgent {
            continue;
        }

        let observation = BotObservation {
            position: position.0,
            rotation: rotation.0,
            velocity: velocity.0,
            health: health.current,
            max_health: health.max,
            current_actions: vec![],
        };

        // Calculate reward and store experience if we have previous data
        if let (Some(prev_obs), Some(prev_action)) = (
            rl_state.last_observations.get(&bot.bot_id),
            rl_state.last_actions.get(&bot.bot_id),
        ) {
            let reward = rl_state.calculate_reward(&observation, Some(prev_obs), prev_action);

            let experience = Experience {
                state: rl_state.observation_to_state(prev_obs),
                action: prev_action.clone(),
                reward,
                next_state: rl_state.observation_to_state(&observation),
                done: health.current <= 0.0,
            };

            rl_state.add_experience(experience);

            if health.current <= 0.0 {
                rl_state.episode_count += 1;
                debug!(
                    "Episode {} ended for bot {} with reward {:.2}",
                    rl_state.episode_count, bot.bot_id, reward
                );
            }
        }

        rl_state.last_observations.insert(bot.bot_id, observation);
    }
}

/// System to train the RL agent
fn train_rl_agent(mut rl_state: ResMut<RLTrainingState>) {
    if !rl_state.training_enabled {
        return;
    }

    // Perform training step
    rl_state.train_step();

    // Update target network periodically
    if rl_state.training_step % rl_state.target_update_frequency == 0 {
        rl_state.update_target_network();
    }

    // Update exploration rate
    rl_state.update_epsilon();

    // Log training progress
    if rl_state.training_step % 100 == 0 && rl_state.training_step > 0 {
        info!(
            "RL Training Step {}: Epsilon: {:.3}, Buffer: {}, Episodes: {}",
            rl_state.training_step,
            rl_state.epsilon,
            rl_state.experience_buffer.len(),
            rl_state.episode_count
        );
    }
}

/// System to apply RL agent actions to external agent bots
fn apply_rl_actions(
    mut bot_query: Query<(&AIBot, &mut ActionState<PlayerAction>), With<PlayerId>>,
    mut rl_state: ResMut<RLTrainingState>,
) {
    for (bot, mut action_state) in bot_query.iter_mut() {
        // Only control external agent bots
        if bot.bot_type != BotType::ExternalAgent {
            continue;
        }

        // Get current state and predict action
        if let Some(current_obs) = rl_state.last_observations.get(&bot.bot_id) {
            let state = rl_state.observation_to_state(current_obs);
            let action = rl_state.get_action(&state);

            // Apply actions directly to the ActionState - this is the key interface!
            action.apply_to_action_state(&mut action_state);

            // Store action for next experience
            rl_state.last_actions.insert(bot.bot_id, action);
        }
    }
}
