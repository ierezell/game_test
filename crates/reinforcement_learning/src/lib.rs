pub mod ai_bot;
pub mod reinforcement_learning;

// Re-export main RL structs and functions
pub use reinforcement_learning::{
    Experience, PlayerActionSet, RLPlugin, RLTrainingState, SimpleNetwork,
};

// Re-export external agent bot interface
pub use ai_bot::{
    get_external_agent_observation, set_external_agent_jump, set_external_agent_look,
    set_external_agent_movement, set_external_agent_shoot, spawn_external_agent_bot,
};
