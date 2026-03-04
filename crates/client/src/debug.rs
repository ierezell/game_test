use crate::ClientGameState;
use crate::camera::PlayerCamera;

use avian3d::prelude::*;
use bevy::dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin, FrameTimeGraphConfig};
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};
use leafwing_input_manager::prelude::ActionState;

use lightyear::prelude::{Controlled, Predicted};
use shared::{
    components::health::Health,
    inputs::input::PlayerAction,
    navigation::{PatrolRoute, PatrolState, SimpleNavigationAgent},
    protocol::{CharacterMarker, PlayerId},
};
use std::time::Duration;

pub struct ClientDebugPlugin;

#[derive(Resource, Debug, Default)]
struct DebugViewState {
    enabled: bool,
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
                enabled: false,
                frame_time_graph_config: FrameTimeGraphConfig {
                    enabled: false,
                    ..default()
                },
            },
        });
        app.add_systems(OnEnter(ClientGameState::Playing), spawn_debug_options_ui);
        app.add_systems(OnExit(ClientGameState::Playing), despawn_debug_options_ui);
        app.add_systems(Update, toggle_debug_view);
        app.add_systems(
            Update,
            update_debug_options_visibility.run_if(in_state(ClientGameState::Playing)),
        );
        app.add_systems(
            Update,
            (
                debug_navigation_paths,
                debug_npc_health_gizmos,
                update_debug_options_text,
            )
                .run_if(in_state(ClientGameState::Playing))
                .run_if(debug_view_enabled),
        );
    }
}

#[derive(Component)]
struct DebugOptionsRoot;

#[derive(Component)]
struct DebugCursorStatusText;

#[derive(Component)]
struct DebugInputStatusText;

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

fn spawn_debug_options_ui(mut commands: Commands) {
    commands
        .spawn((
            Name::new("DebugOptionsOverlay"),
            DebugOptionsRoot,
            Visibility::Hidden,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(16.0),
                top: Val::Px(16.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(4.0),
                padding: UiRect::all(Val::Px(10.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.05, 0.05, 0.08, 0.85)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Debug Options"),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
            ));

            parent.spawn((
                DebugCursorStatusText,
                Text::new("Cursor: --"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
            ));

            parent.spawn((
                DebugInputStatusText,
                Text::new("Input: --"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
            ));

            parent.spawn((
                Text::new("LMB: Lock cursor | Esc: Unlock cursor"),
                TextFont {
                    font_size: 13.0,
                    ..default()
                },
            ));
        });
}

fn despawn_debug_options_ui(
    mut commands: Commands,
    debug_ui_query: Query<Entity, With<DebugOptionsRoot>>,
) {
    for entity in &debug_ui_query {
        commands.entity(entity).despawn();
    }
}

fn update_debug_options_visibility(
    debug_view_state: Res<DebugViewState>,
    mut ui_query: Query<&mut Visibility, With<DebugOptionsRoot>>,
) {
    if let Ok(mut visibility) = ui_query.single_mut() {
        *visibility = if debug_view_state.enabled {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

fn update_debug_options_text(
    mut cursor_text_query: Query<
        &mut Text,
        (With<DebugCursorStatusText>, Without<DebugInputStatusText>),
    >,
    mut input_text_query: Query<
        &mut Text,
        (With<DebugInputStatusText>, Without<DebugCursorStatusText>),
    >,
    cursor_options_query: Query<&CursorOptions, With<PrimaryWindow>>,
    player_actions: Query<
        &ActionState<PlayerAction>,
        (With<PlayerId>, With<Predicted>, With<Controlled>),
    >,
) {
    if let Ok(mut text) = cursor_text_query.single_mut() {
        let is_locked = cursor_options_query
            .single()
            .is_ok_and(|cursor_options| cursor_options.grab_mode == CursorGrabMode::Locked);
        **text = if is_locked {
            "Cursor: Locked".to_string()
        } else {
            "Cursor: Unlocked".to_string()
        };
    }

    if let Ok(mut text) = input_text_query.single_mut() {
        let input_enabled = player_actions
            .single()
            .is_ok_and(|action_state| !action_state.disabled());
        **text = if input_enabled {
            "Input: Enabled".to_string()
        } else {
            "Input: Blocked".to_string()
        };
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
