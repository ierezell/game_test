use crate::GymMode;
use crate::gym::GymRandomWanderer;
use crate::navigation::{NavigationPathState, SimpleNavigationAgent};
use crate::protocol::{CharacterMarker, PlayerId};
use avian3d::prelude::Position;
use bevy::prelude::{
    Component, Entity, Name, Query, Res, Time, Vec2, Vec3, With, Without, info, warn,
};
use std::fmt::Arguments;

#[derive(Clone, Copy, Debug)]
pub struct DebugControls {
    pub client_debug_gizmos: bool,
    pub gym_diagnostics_logs: bool,
    pub console_debug_prints: bool,
}

pub const DEBUG_CONTROLS: DebugControls = DebugControls {
    client_debug_gizmos: true,
    gym_diagnostics_logs: true,
    console_debug_prints: true,
};

#[inline]
pub const fn client_debug_gizmos_enabled() -> bool {
    DEBUG_CONTROLS.client_debug_gizmos
}

#[inline]
pub const fn gym_diagnostics_logs_enabled() -> bool {
    DEBUG_CONTROLS.gym_diagnostics_logs
}

#[inline]
pub const fn console_debug_prints_enabled() -> bool {
    DEBUG_CONTROLS.console_debug_prints
}

pub fn debug_println(args: Arguments<'_>) {
    if console_debug_prints_enabled() {
        println!("{args}");
    }
}

pub fn gym_debug_info(args: Arguments<'_>) {
    if gym_diagnostics_logs_enabled() {
        info!("{args}");
    }
}

pub fn gym_debug_warn(args: Arguments<'_>) {
    if gym_diagnostics_logs_enabled() {
        warn!("{args}");
    }
}

#[derive(Component, Clone, Debug)]
pub struct GymWanderDiagnostics {
    pub last_position: Vec3,
    pub stationary_secs: f32,
    pub report_cooldown_secs: f32,
    pub snapshot_cooldown_secs: f32,
    pub no_target_secs: f32,
    pub pathless_secs: f32,
    pub last_distance_to_target: Option<f32>,
    pub no_progress_secs: f32,
}

impl GymWanderDiagnostics {
    pub fn new(spawn_position: Vec3) -> Self {
        Self {
            last_position: spawn_position,
            stationary_secs: 0.0,
            report_cooldown_secs: 0.0,
            snapshot_cooldown_secs: 0.0,
            no_target_secs: 0.0,
            pathless_secs: 0.0,
            last_distance_to_target: None,
            no_progress_secs: 0.0,
        }
    }
}

pub fn log_gym_wandering_diagnostics(
    gym_mode: Option<Res<GymMode>>,
    time: Res<Time>,
    mut npc_query: Query<
        (
            Entity,
            Option<&Name>,
            &Position,
            &SimpleNavigationAgent,
            Option<&NavigationPathState>,
            &mut GymWanderDiagnostics,
        ),
        (
            With<GymRandomWanderer>,
            With<CharacterMarker>,
            Without<PlayerId>,
        ),
    >,
) {
    if !gym_diagnostics_logs_enabled() {
        return;
    }

    let is_gym_mode = gym_mode.map(|gm| gm.0).unwrap_or(false);
    if !is_gym_mode {
        return;
    }

    let dt = time.delta_secs();

    for (entity, name, position, nav_agent, path_state, mut diagnostics) in &mut npc_query {
        diagnostics.report_cooldown_secs = (diagnostics.report_cooldown_secs - dt).max(0.0);
        diagnostics.snapshot_cooldown_secs = (diagnostics.snapshot_cooldown_secs - dt).max(0.0);

        let moved_planar = Vec2::new(position.0.x, position.0.z).distance(Vec2::new(
            diagnostics.last_position.x,
            diagnostics.last_position.z,
        ));
        diagnostics.last_position = position.0;

        let label = name.map(|n| n.as_str()).unwrap_or("GymNpc");

        let Some(target) = nav_agent.current_target else {
            diagnostics.stationary_secs = 0.0;
            diagnostics.pathless_secs = 0.0;
            diagnostics.no_progress_secs = 0.0;
            diagnostics.last_distance_to_target = None;
            diagnostics.no_target_secs += dt;

            if diagnostics.no_target_secs >= 1.0 && diagnostics.report_cooldown_secs <= 0.0 {
                warn!(
                    "Gym NPC has no target: entity={:?} name={} pos={:?} speed={:.2}",
                    entity, label, position.0, nav_agent.speed,
                );
                diagnostics.report_cooldown_secs = 1.0;
            }
            continue;
        };

        diagnostics.no_target_secs = 0.0;

        let distance_to_target =
            Vec2::new(position.0.x, position.0.z).distance(Vec2::new(target.x, target.z));

        let (waypoint, remaining_waypoints) = path_state
            .map(|path| (path.current_waypoint, path.remaining_waypoints.len()))
            .unwrap_or((None, 0));
        let has_path_guidance = waypoint.is_some() || remaining_waypoints > 0;

        if !has_path_guidance && distance_to_target > nav_agent.arrival_threshold.max(0.75) * 1.15 {
            diagnostics.pathless_secs += dt;
        } else {
            diagnostics.pathless_secs = 0.0;
        }

        if let Some(previous_distance) = diagnostics.last_distance_to_target {
            if distance_to_target + 0.03 >= previous_distance {
                diagnostics.no_progress_secs += dt;
            } else {
                diagnostics.no_progress_secs = 0.0;
            }
        }
        diagnostics.last_distance_to_target = Some(distance_to_target);

        if diagnostics.pathless_secs >= 1.0 && diagnostics.report_cooldown_secs <= 0.0 {
            warn!(
                "Gym NPC has target but no path guidance: entity={:?} name={} pos={:?} target={:?} dist={:.2} moved={:.4} remaining_waypoints={}",
                entity,
                label,
                position.0,
                target,
                distance_to_target,
                moved_planar,
                remaining_waypoints,
            );
            diagnostics.report_cooldown_secs = 1.0;
        }

        if diagnostics.no_progress_secs >= 2.0
            && distance_to_target > nav_agent.arrival_threshold.max(0.75) * 1.5
            && diagnostics.report_cooldown_secs <= 0.0
        {
            warn!(
                "Gym NPC making no progress toward target: entity={:?} name={} pos={:?} target={:?} dist={:.2} moved={:.4} waypoint={:?} remaining_waypoints={}",
                entity,
                label,
                position.0,
                target,
                distance_to_target,
                moved_planar,
                waypoint,
                remaining_waypoints,
            );
            diagnostics.report_cooldown_secs = 1.0;
        }

        if diagnostics.snapshot_cooldown_secs <= 0.0 {
            info!(
                "Gym NPC telemetry: entity={:?} name={} pos={:?} target={:?} dist={:.2} moved={:.4} waypoint={:?} remaining_waypoints={} speed={:.2}",
                entity,
                label,
                position.0,
                target,
                distance_to_target,
                moved_planar,
                waypoint,
                remaining_waypoints,
                nav_agent.speed,
            );
            diagnostics.snapshot_cooldown_secs = 2.5;
        }

        let considered_stuck = moved_planar < 0.015
            && distance_to_target > nav_agent.arrival_threshold.max(0.75) * 1.15;

        if considered_stuck {
            diagnostics.stationary_secs += dt;
        } else {
            diagnostics.stationary_secs = 0.0;
        }

        if diagnostics.stationary_secs >= 1.2 && diagnostics.report_cooldown_secs <= 0.0 {
            warn!(
                "Gym NPC appears stuck/vibrating: entity={:?} name={} pos={:?} target={:?} dist={:.2} moved={:.4} waypoint={:?} remaining_waypoints={} speed={:.2} arrival_threshold={:.2}",
                entity,
                label,
                position.0,
                target,
                distance_to_target,
                moved_planar,
                waypoint,
                remaining_waypoints,
                nav_agent.speed,
                nav_agent.arrival_threshold,
            );
            diagnostics.report_cooldown_secs = 1.0;
        }
    }
}
