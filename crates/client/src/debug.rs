use avian3d::prelude::*;
use bevy::prelude::*;
use lightyear::prelude::{Controlled, LocalTimeline, Predicted};
use crate::camera::PlayerCamera;
use shared::{
    components::health::Health,
    navigation::{PatrolRoute, PatrolState, SimpleNavigationAgent},
    protocol::{CharacterMarker, PlayerId},
};

pub struct ClientDebugPlugin;

#[derive(Resource, Debug)]
struct DebugViewSettings {
    gizmos_enabled: bool,
}

impl Default for DebugViewSettings {
    fn default() -> Self {
        Self {
            gizmos_enabled: true,
        }
    }
}

impl Plugin for ClientDebugPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DebugViewSettings>();
        app.add_systems(Update, handle_debug_shortcuts);
        app.add_systems(Update, debug_navigation_paths.run_if(gizmos_enabled));
        app.add_systems(Update, debug_player_position);
        app.add_systems(Update, debug_npc_health_gizmos.run_if(gizmos_enabled));
    }
}

fn gizmos_enabled(settings: Res<DebugViewSettings>) -> bool {
    settings.gizmos_enabled
}

fn handle_debug_shortcuts(
    keys: Res<ButtonInput<KeyCode>>,
    mut settings: ResMut<DebugViewSettings>,
) {
    if keys.just_pressed(KeyCode::F3) {
        settings.gizmos_enabled = !settings.gizmos_enabled;
        info!(
            "Debug gizmos {}",
            if settings.gizmos_enabled {
                "enabled"
            } else {
                "disabled"
            }
        );
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
    timeline: Res<LocalTimeline>,
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

fn debug_npc_health_gizmos(
    npc_query: Query<(&Position, &Health), (With<CharacterMarker>, Without<PlayerId>)>,
    camera_query: Query<&GlobalTransform, With<PlayerCamera>>,
    mut gizmos: Gizmos,
) {
    let camera_transform = camera_query.single().ok();

    for (position, health) in &npc_query {
        let health_ratio = health.percentage();
        let y_offset = Vec3::Y * 2.6;
        let center = position.0 + y_offset;

        let right_axis = camera_transform
            .map(|transform| transform.right().as_vec3())
            .unwrap_or(Vec3::X)
            .normalize_or_zero();
        let up_axis = camera_transform
            .map(|transform| transform.up().as_vec3())
            .unwrap_or(Vec3::Y)
            .normalize_or_zero();

        let bar_half_width = 0.8;
        let bar_half_height = 0.08;
        let left = center - right_axis * bar_half_width;
        let right = center + right_axis * bar_half_width;

        let top_left = left + up_axis * bar_half_height;
        let bottom_left = left - up_axis * bar_half_height;
        let top_right = right + up_axis * bar_half_height;
        let bottom_right = right - up_axis * bar_half_height;

        gizmos.line(top_left, top_right, Color::BLACK);
        gizmos.line(bottom_left, bottom_right, Color::BLACK);
        gizmos.line(top_left, bottom_left, Color::BLACK);
        gizmos.line(top_right, bottom_right, Color::BLACK);

        let background_color = Color::srgb(0.35, 0.0, 0.0);
        for fill_step in -2..=2 {
            let vertical_offset = up_axis * (fill_step as f32 * 0.022);
            gizmos.line(left + vertical_offset, right + vertical_offset, background_color);
        }

        let current_right = left + right_axis * (2.0 * bar_half_width * health_ratio);
        let health_color = if health_ratio > 0.6 {
            Color::srgb(0.0, 0.95, 0.0)
        } else if health_ratio > 0.3 {
            Color::srgb(0.95, 0.75, 0.0)
        } else {
            Color::srgb(0.95, 0.0, 0.0)
        };

        for fill_step in -2..=2 {
            let vertical_offset = up_axis * (fill_step as f32 * 0.022);
            gizmos.line(
                left + vertical_offset,
                current_right + vertical_offset,
                health_color,
            );
        }

        for tick in [0.25_f32, 0.5_f32, 0.75_f32] {
            let tick_center = left + right_axis * (2.0 * bar_half_width * tick);
            let tick_top = tick_center + up_axis * (bar_half_height * 0.7);
            let tick_bottom = tick_center - up_axis * (bar_half_height * 0.7);
            gizmos.line(tick_top, tick_bottom, Color::srgb(0.15, 0.15, 0.15));
        }

        let marker_color = if health.is_dead {
            Color::srgb(0.7, 0.0, 0.0)
        } else {
            Color::srgb(0.0, 0.9, 0.0)
        };
        gizmos.sphere(center + up_axis * 0.18, 0.07, marker_color);
    }
}
