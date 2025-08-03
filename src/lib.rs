// Library interface for gtfo-like-game
// This allows the modules to be imported in tests

pub mod audio;
pub mod combat;
pub mod debug_ui;
pub mod enemies;
pub mod environment;
pub mod game_state;
pub mod menu;
pub mod player;
pub mod static_level;
pub mod ui;
pub mod utils;

// Re-export commonly used types for easier testing
pub use combat::{Health, Projectile, Weapon};
pub use enemies::{Enemy, EnemyAI, EnemyState, EnemyType};
pub use game_state::GameState;
pub use player::{Player, PlayerAction, PlayerController};
