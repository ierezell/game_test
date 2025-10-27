use avian3d::prelude::{LinearVelocity, Position};
use bevy::prelude::*;
use leafwing_input_manager::prelude::ActionState;
use nalgebra::{DMatrix, DVector};
use rand::{Rng, rng};
use shared::input::PlayerAction;
use std::collections::VecDeque;

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
        let mut rng = rng();

        // Xavier initialization
        let w1_scale = (2.0 / (input_size + hidden_size) as f32).sqrt();
        let w2_scale = (2.0 / (hidden_size + output_size) as f32).sqrt();

        let weights1 = DMatrix::from_fn(hidden_size, input_size, |_, _| {
            rng.random_range(-w1_scale..w1_scale)
        });
        let bias1 = DVector::zeros(hidden_size);

        let weights2 = DMatrix::from_fn(output_size, hidden_size, |_, _| {
            rng.random_range(-w2_scale..w2_scale)
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
    pub epsilon: f32,
    pub training_step: usize,
    pub last_observation: Option<RLObservation>,
    pub last_action: Option<PlayerActionSet>,
    pub buffer_size: usize,
    pub batch_size: usize,
    pub episode_count: usize,
}

/// Minimal RL observation for a single bot
#[derive(Clone, Debug)]
pub struct RLObservation {
    pub position: Vec3,
    pub velocity: Vec3,
    pub health: f32,
    pub max_health: f32,
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
            epsilon: 1.0,
            training_step: 0,
            last_observation: None,
            last_action: None,
            buffer_size: 10000,
            batch_size: 32,
            episode_count: 0,
        }
    }
}

impl RLTrainingState {
    /// Initialize the RL networks
    pub fn initialize(&mut self) {
        let input_size = 8; // Position(3) + Velocity(3) + Health(2)
        let hidden_size = 64;
        let output_size = 6; // Move(2) + Look(2) + Jump(1) + Shoot(1)

        self.q_network = Some(SimpleNetwork::new(input_size, hidden_size, output_size));
        self.target_network = Some(SimpleNetwork::new(input_size, hidden_size, output_size));

        info!("RL Agent initialized with simple neural network");
    }

    /// Convert observation to state vector
    pub fn observation_to_state(&self, obs: &RLObservation) -> Vec<f32> {
        let mut state = Vec::with_capacity(8);
        // Position normalized to [-1, 1] range (assuming game world is ~20x20)
        state.extend_from_slice(&[
            obs.position.x / 10.0,
            obs.position.y / 10.0,
            obs.position.z / 10.0,
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
        current_obs: &RLObservation,
        previous_obs: Option<&RLObservation>,
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
        let mut rng = rng();
        if rng.random::<f32>() < self.epsilon {
            // Random exploration
            PlayerActionSet {
                movement: Vec2::new(rng.random_range(-1.0..1.0), rng.random_range(-1.0..1.0)),
                look: Vec2::new(rng.random_range(-1.0..1.0), rng.random_range(-1.0..1.0)),
                jump: rng.random::<f32>() < 0.1,
                shoot: rng.random::<f32>() < 0.2,
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
}

/// System to collect observations and train RL agents (minimal, single bot)
fn collect_rl_observations(
    bot_query: Query<(&Position, &LinearVelocity)>,
    mut rl_state: ResMut<RLTrainingState>,
) {
    // Initialize RL agent if not done
    if rl_state.q_network.is_none() {
        rl_state.initialize();
    }

    // Assume only one bot/player for RL
    if let Some((position, velocity)) = bot_query.iter().next() {
        // For now, hardcode health
        let observation = RLObservation {
            position: position.0,
            velocity: velocity.0,
            health: 100.0,
            max_health: 100.0,
        };

        // Calculate reward and store experience if we have previous data
        if let (Some(prev_obs), Some(prev_action)) =
            (&rl_state.last_observation, &rl_state.last_action)
        {
            let reward = rl_state.calculate_reward(&observation, Some(prev_obs), prev_action);
            let experience = Experience {
                state: rl_state.observation_to_state(prev_obs),
                action: prev_action.clone(),
                reward,
                next_state: rl_state.observation_to_state(&observation),
                done: observation.health <= 0.0,
            };
            rl_state.add_experience(experience);
            if observation.health <= 0.0 {
                rl_state.episode_count += 1;
                debug!(
                    "Episode {} ended with reward {:.2}",
                    rl_state.episode_count, reward
                );
            }
        }
        rl_state.last_observation = Some(observation);
    }
}

/// System to train the RL agent
fn train_rl_agent(mut rl_state: ResMut<RLTrainingState>) {
    rl_state.train_step();
}

/// System to apply RL agent actions to the single bot (minimal)
fn apply_rl_actions(
    mut bot_query: Query<&mut LinearVelocity>,
    mut rl_state: ResMut<RLTrainingState>,
) {
    if let Some(mut velocity) = bot_query.iter_mut().next() {
        if let Some(current_obs) = &rl_state.last_observation {
            let state = rl_state.observation_to_state(current_obs);
            let action = rl_state.get_action(&state);
            // Apply movement action directly to velocity
            let input_direction = Vec3::new(action.movement.x, 0.0, -action.movement.y);
            let desired_velocity = input_direction * 5.0;
            velocity.0 = Vec3::new(desired_velocity.x, velocity.0.y, desired_velocity.z);
            // Store action for next experience
            rl_state.last_action = Some(action);
        }
    }
}
