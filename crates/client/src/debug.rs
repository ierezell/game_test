use bevy::prelude::*;
use shared::navigation::{NavigationAgent, NavigationBehavior, NavigationTarget};
pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, debug_navigation_paths);
    }
}

/// Debug system to visualize navigation paths
fn debug_navigation_paths(
    agents: Query<(&Transform, &NavigationTarget, &NavigationAgent)>,
    mut gizmos: Gizmos,
) {
    for (transform, target, agent) in agents.iter() {
        let color = match agent.behavior {
            NavigationBehavior::Direct => Color::srgb(0.0, 1.0, 0.0), // Green
            NavigationBehavior::Patrol => Color::srgb(0.0, 0.0, 1.0), // Blue
            NavigationBehavior::Follow { .. } => Color::srgb(1.0, 1.0, 0.0), // Yellow
            NavigationBehavior::Flee { .. } => Color::srgb(1.0, 0.0, 0.0), // Red
            NavigationBehavior::Formation { .. } => Color::srgb(1.0, 0.0, 1.0), // Magenta
        };

        // Draw path
        if !target.path.is_empty() {
            let mut points = vec![transform.translation];
            points.extend(target.path.iter().copied());

            for window in points.windows(2) {
                gizmos.line(window[0], window[1], color);
            }

            // Draw destination
            gizmos.sphere(target.destination, 0.2, color);
        }

        // Draw agent radius (using a simple approach for now)
        let radius_color = Color::srgba(
            color.to_srgba().red,
            color.to_srgba().green,
            color.to_srgba().blue,
            0.3,
        );
        gizmos.circle_2d(transform.translation.truncate(), agent.radius, radius_color);
    }
}
