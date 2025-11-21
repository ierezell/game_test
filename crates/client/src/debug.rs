use avian3d::prelude::*;
use bevy::prelude::*;
use lightyear::prelude::{
    Controlled, LocalTimeline, NetworkTimeline, Predicted, PredictionManager,
};
use shared::{
    navigation::{PatrolRoute, PatrolState, SimpleNavigationAgent},
    protocol::{CharacterMarker, PlayerId},
};

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PhysicsDebugPlugin::default());
        app.add_systems(Update, debug_navigation_paths);
        app.add_systems(Update, debug_player_position);
    }
}

fn debug_navigation_paths(
    agents: Query<(
        &Position,
        &SimpleNavigationAgent,
        Option<&PatrolRoute>,
        Option<&PatrolState>,
    )>,
    mut gizmos: Gizmos,
) {
    for (position, agent, patrol_route, patrol_state) in agents.iter() {
        let color = Color::srgb(0.0, 0.0, 1.0);
        let current_pos = position.0;

        if let Some(target) = agent.current_target {
            gizmos.line(current_pos, target, color);
            gizmos.sphere(target, 0.2, Color::srgb(1.0, 0.0, 0.0));
        }

        if let Some(route) = patrol_route {
            if route.points.len() > 1 {
                for window in route.points.windows(2) {
                    gizmos.line(window[0], window[1], Color::srgb(0.5, 0.5, 1.0));
                }

                if let Some(state) = patrol_state {
                    if let Some(current_point) = route.points.get(state.current_target_index) {
                        gizmos.sphere(*current_point, 0.3, Color::srgb(0.0, 1.0, 0.0));
                    }
                }
            }
        }
    }
}

fn debug_player_position(
    player_query: Query<
        (&Name, &Position, &LinearVelocity),
        (
            With<PlayerId>,
            With<Predicted>,
            With<Controlled>,
            With<CharacterMarker>,
        ),
    >,
    timeline: Single<&LocalTimeline, With<PredictionManager>>,
) {
    for (name, position, linear_velocity) in player_query.iter() {
        debug!(
            "C:{:?} pos:{:?} vel:{:?} tick:{:?}",
            name,
            position,
            linear_velocity,
            timeline.tick()
        );
    }
}
