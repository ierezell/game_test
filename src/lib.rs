pub mod audio;
pub mod combat;
pub mod debug_ui;
pub mod enemies;
pub mod events;
pub mod menu;
pub mod observers;
pub mod player;
pub mod scenes;
pub mod static_level;
pub mod ui;

// Networking modules (updated for Lightyear 0.23.0)
pub mod client;
pub mod protocol;
pub mod server;
pub mod shared;

pub use combat::{Health, Weapon};
pub use enemies::{Enemy, EnemyAI, EnemyState, EnemyType};
pub use player::{Player, PlayerAction, PlayerController};
