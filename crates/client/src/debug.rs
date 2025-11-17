use avian3d::prelude::*;
use bevy::prelude::*;
use shared::navigation::{SimpleNavigationAgent, PatrolRoute, PatrolState};

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        // Add Avian3D physics debug renderer
        app.add_plugins(PhysicsDebugPlugin::default());
        
        // Add our custom navigation debug system
        app.add_systems(Update, debug_navigation_paths);
    }
}

/// Debug system to visualize simple navigation
fn debug_navigation_paths(
    agents: Query<(&Position, &SimpleNavigationAgent, Option<&PatrolRoute>, Option<&PatrolState>)>,
    mut gizmos: Gizmos,
) {
    let agent_count = agents.iter().len();
    if agent_count > 0 {
        info!("Found {} navigation agents for debug visualization", agent_count);
    }
    
    for (position, agent, patrol_route, patrol_state) in agents.iter() {
        let color = Color::srgb(0.0, 0.0, 1.0); // Blue for patrol agents
        let current_pos = position.0;
        
        // Draw current target if available
        if let Some(target) = agent.current_target {
            gizmos.line(current_pos, target, color);
            gizmos.sphere(target, 0.2, Color::srgb(1.0, 0.0, 0.0)); // Red target sphere
            info!("Drawing navigation line from {:?} to {:?}", current_pos, target);
        }

        // Draw patrol route if available
        if let Some(route) = patrol_route {
            if route.points.len() > 1 {
                info!("Drawing patrol route with {} points", route.points.len());
                for window in route.points.windows(2) {
                    gizmos.line(window[0], window[1], Color::srgb(0.5, 0.5, 1.0)); // Light blue route
                }
                
                // Highlight current target in patrol
                if let Some(state) = patrol_state {
                    if let Some(current_point) = route.points.get(state.current_target_index) {
                        gizmos.sphere(*current_point, 0.3, Color::srgb(0.0, 1.0, 0.0)); // Green current target
                    }
                }
            }
        }

        // Draw agent radius
        let radius_color = Color::srgba(0.0, 0.0, 1.0, 0.3);
        gizmos.circle_2d(current_pos.truncate(), agent.arrival_threshold, radius_color);
    }
}
