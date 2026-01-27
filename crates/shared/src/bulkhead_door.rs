// Bulkhead Door System - GTFO-style security doors
//
// Bulkhead doors connect zones and can be opened/closed. The state is managed
// on the server and replicated to clients via Lightyear.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

// ============================================================================
// BULKHEAD DOOR COMPONENTS
// ============================================================================

/// Bulkhead door state - replicated from server to clients
#[derive(Component, Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub enum DoorState {
    /// Door is fully closed (blocks passage)
    #[default]
    Closed,
    /// Door is opening (animation in progress)
    Opening { progress: f32 },
    /// Door is fully open (allows passage)
    Open,
    /// Door is closing (animation in progress)
    Closing { progress: f32 },
    /// Door is locked and cannot be opened
    Locked,
}

/// Marker component for bulkhead door entities
#[derive(Component, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BulkheadDoor {
    /// IDs of the zones this door connects
    pub connects_zones: (u32, u32),
    /// How long it takes to open/close (seconds)
    pub animation_duration: f32,
}

impl Default for BulkheadDoor {
    fn default() -> Self {
        Self {
            connects_zones: (0, 0),
            animation_duration: 2.0,
        }
    }
}

impl BulkheadDoor {
    pub fn new(zone_a: u32, zone_b: u32) -> Self {
        Self {
            connects_zones: (zone_a, zone_b),
            animation_duration: 2.0,
        }
    }
}

// ============================================================================
// DOOR ANIMATION SYSTEM
// ============================================================================

/// Update door opening/closing animations
///
/// This system runs on both client and server. On the server, it updates the
/// DoorState component which is then replicated to clients.
pub fn update_door_animations(
    time: Res<Time>,
    mut door_query: Query<(&mut DoorState, &BulkheadDoor)>,
) {
    let delta = time.delta_secs();

    for (mut state, door) in door_query.iter_mut() {
        match *state {
            DoorState::Opening { progress } => {
                let new_progress = (progress + delta / door.animation_duration).min(1.0);

                if new_progress >= 1.0 {
                    *state = DoorState::Open;
                } else {
                    *state = DoorState::Opening {
                        progress: new_progress,
                    };
                }
            }
            DoorState::Closing { progress } => {
                let new_progress = (progress + delta / door.animation_duration).min(1.0);

                if new_progress >= 1.0 {
                    *state = DoorState::Closed;
                } else {
                    *state = DoorState::Closing {
                        progress: new_progress,
                    };
                }
            }
            _ => {}
        }
    }
}

/// Update door visual representation based on state (CLIENT & SERVER)
///
/// This system updates the Transform of the door mesh based on the DoorState.
/// For a simple implementation, we'll slide the door up/down.
pub fn update_door_visuals(
    mut door_query: Query<(&DoorState, &mut Transform, &BulkheadDoor), Changed<DoorState>>,
) {
    for (state, mut transform, _door) in door_query.iter_mut() {
        // Calculate target Y offset based on door state
        let target_y_offset = match state {
            DoorState::Closed => 0.0,
            DoorState::Open => 6.0, // Door slides up
            DoorState::Opening { progress } => 6.0 * progress,
            DoorState::Closing { progress } => 6.0 * (1.0 - progress),
            DoorState::Locked => 0.0,
        };

        // Update door position (keeping X and Z the same)
        transform.translation.y = target_y_offset;
    }
}

// ============================================================================
// NETWORKING REGISTRATION (called from protocol plugin)
// ============================================================================

/// Register bulkhead door components for networking
///
/// NOTE: Door interaction will be handled via direct component manipulation
/// from server-side game logic (e.g., when player presses 'use' key near door)
pub fn register_bulkhead_networking(app: &mut App) {
    use lightyear::prelude::AppComponentExt;

    // Register components for replication
    app.register_component::<BulkheadDoor>();
    app.register_component::<DoorState>();

    info!("Bulkhead door networking registered");
}

// ============================================================================
// PLUGIN
// ============================================================================

pub struct BulkheadDoorPlugin;

impl Plugin for BulkheadDoorPlugin {
    fn build(&self, app: &mut App) {
        // Add door animation update system
        app.add_systems(Update, (update_door_animations, update_door_visuals));

        info!("Bulkhead Door plugin initialized");
    }
}
