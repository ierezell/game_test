use crate::camera::PlayerCamera;

use avian3d::prelude::*;
use bevy::dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin, FrameTimeGraphConfig};
use bevy::prelude::*;

use shared::{
    components::health::Health,
    navigation::{PatrolRoute, PatrolState, SimpleNavigationAgent},
    protocol::{CharacterMarker, PlayerId},
};
use std::time::Duration;

pub struct ClientDebugPlugin;

#[derive(Resource, Debug)]
struct DebugViewState {
    enabled: bool,
}

impl Default for DebugViewState {
    fn default() -> Self {
        Self { enabled: true }
    }
}

impl Plugin for ClientDebugPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DebugViewState>();
        app.add_plugins(FpsOverlayPlugin {
            config: FpsOverlayConfig {
                text_config: TextFont {
                    font_size: 18.0,
                    ..default()
                },
                text_color: Color::srgb(0.2, 1.0, 0.2),
                refresh_interval: Duration::from_millis(200),
                enabled: true,
                frame_time_graph_config: FrameTimeGraphConfig {
                    enabled: false,
                    ..default()
                },
            },
        });
        app.add_systems(Update, toggle_debug_view);
        app.add_systems(
            Update,
            (debug_navigation_paths, debug_npc_health_gizmos).run_if(debug_view_enabled),
        );
    }
}

fn debug_view_enabled(debug_view_state: Res<DebugViewState>) -> bool {
    debug_view_state.enabled
}

fn toggle_debug_view(
    keys: Res<ButtonInput<KeyCode>>,
    mut debug_view_state: ResMut<DebugViewState>,
    mut fps_overlay_config: ResMut<FpsOverlayConfig>,
) {
    if keys.just_pressed(KeyCode::KeyH) || keys.just_pressed(KeyCode::F3) {
        debug_view_state.enabled = !debug_view_state.enabled;
        fps_overlay_config.enabled = debug_view_state.enabled;
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

        if let Some(route) = patrol_route
            && route.points.len() > 1
        {
            for window in route.points.windows(2) {
                gizmos.line(window[0], window[1], Color::srgb(0.5, 0.5, 1.0));
            }

            if let Some(state) = patrol_state
                && let Some(current_point) = route.points.get(state.current_target_index)
            {
                gizmos.sphere(*current_point, 0.3, Color::srgb(0.0, 1.0, 0.0));
            }
        }
    }
}

fn debug_npc_health_gizmos(
    npc_query: Query<(&Position, &Health), (With<CharacterMarker>, Without<PlayerId>)>,
    camera_query: Query<&GlobalTransform, With<PlayerCamera>>,
    mut gizmos: Gizmos,
) {
    let camera_transform = camera_query.single().ok();

    for (position, health) in &npc_query {
        let health_ratio = health.percentage();
        let center = position.0 + Vec3::Y * 2.5;

        let right_axis = camera_transform
            .map(|transform| transform.right().as_vec3())
            .unwrap_or(Vec3::X)
            .normalize_or_zero();
        let up_axis = camera_transform
            .map(|transform| transform.up().as_vec3())
            .unwrap_or(Vec3::Y)
            .normalize_or_zero();

        let bar_width = 1.6;
        let left = center - right_axis * (bar_width * 0.5);
        let right = center + right_axis * (bar_width * 0.5);
        let current_right = left + right_axis * (bar_width * health_ratio);

        let background_color = Color::srgb(0.3, 0.0, 0.0);
        let health_color = if health_ratio > 0.6 {
            Color::srgb(0.0, 0.95, 0.0)
        } else if health_ratio > 0.3 {
            Color::srgb(0.95, 0.75, 0.0)
        } else {
            Color::srgb(0.95, 0.0, 0.0)
        };

        for offset in [-0.03_f32, 0.0, 0.03] {
            let vertical_offset = up_axis * offset;
            gizmos.line(
                left + vertical_offset,
                right + vertical_offset,
                background_color,
            );
            gizmos.line(
                left + vertical_offset,
                current_right + vertical_offset,
                health_color,
            );
        }
    }
}
