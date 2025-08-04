use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPlugin, egui};
use bevy_inspector_egui::quick::WorldInspectorPlugin;

pub struct DebugUIPlugin;

impl Plugin for DebugUIPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin::default())
            .add_plugins(WorldInspectorPlugin::new())
            .init_resource::<DebugMenuState>()
            .add_systems(Update, debug_input_system)
            .add_systems(Update, debug_ui_system);
    }
}

#[derive(Resource)]
pub struct DebugMenuState {
    pub show_menu: bool,
    pub show_inspector: bool,
    pub show_aabb: bool,
    pub show_colliders: bool,
    pub show_contact_points: bool,
    pub show_contact_normals: bool,
    pub show_joints: bool,
    pub show_raycasts: bool,
    pub show_shapecasts: bool,
    pub show_axes: bool,
    pub hide_meshes: bool,
    pub aabb_color: [f32; 3],
    pub collider_color: [f32; 3],
    pub contact_point_color: [f32; 3],
    pub contact_normal_color: [f32; 3],
}

impl Default for DebugMenuState {
    fn default() -> Self {
        Self {
            show_menu: false,
            show_inspector: false,
            show_aabb: false,
            show_colliders: false,
            show_contact_points: false,
            show_contact_normals: false,
            show_joints: false,
            show_raycasts: false,
            show_shapecasts: false,
            show_axes: false,
            hide_meshes: false,
            aabb_color: [0.0, 1.0, 0.0],
            collider_color: [1.0, 1.0, 0.0],
            contact_point_color: [1.0, 0.0, 0.0],
            contact_normal_color: [0.0, 0.0, 1.0],
        }
    }
}

pub fn debug_input_system(
    mut debug_state: ResMut<DebugMenuState>,
    input: Res<ButtonInput<KeyCode>>,
) {
    if input.just_pressed(KeyCode::KeyM) {
        debug_state.show_menu = !debug_state.show_menu;
        debug!("Debug menu toggled: {}", debug_state.show_menu);
    }
}

pub fn debug_ui_system(
    mut contexts: EguiContexts,
    mut debug_state: ResMut<DebugMenuState>,
    mut gizmo_config_store: ResMut<GizmoConfigStore>,
) {
    if !debug_state.show_menu {
        return;
    }

    egui::Window::new("Debug Menu")
        .default_width(350.0)
        .default_height(700.0)
        .show(
            contexts.ctx_mut().expect("Failed to get EGUI context"),
            |ui| {
                ui.heading("Debug Tools");
                ui.separator();

                ui.horizontal(|ui| {
                    ui.checkbox(&mut debug_state.show_inspector, "Show Entity Inspector");
                    ui.label("Browse all entities in the world");
                });

                ui.separator();
                ui.heading("Physics Debug Visualization");
                ui.separator();

                ui.horizontal(|ui| {
                    if ui
                        .checkbox(&mut debug_state.show_aabb, "Show AABB")
                        .clicked()
                    {
                        update_physics_gizmos(&mut gizmo_config_store, &debug_state);
                    }
                    ui.color_edit_button_rgb(&mut debug_state.aabb_color);
                });

                ui.horizontal(|ui| {
                    if ui
                        .checkbox(&mut debug_state.show_colliders, "Show Colliders")
                        .clicked()
                    {
                        update_physics_gizmos(&mut gizmo_config_store, &debug_state);
                    }
                    ui.color_edit_button_rgb(&mut debug_state.collider_color);
                });

                ui.horizontal(|ui| {
                    if ui
                        .checkbox(&mut debug_state.show_contact_points, "Show Contact Points")
                        .clicked()
                    {
                        update_physics_gizmos(&mut gizmo_config_store, &debug_state);
                    }
                    ui.color_edit_button_rgb(&mut debug_state.contact_point_color);
                });

                ui.horizontal(|ui| {
                    if ui
                        .checkbox(
                            &mut debug_state.show_contact_normals,
                            "Show Contact Normals",
                        )
                        .clicked()
                    {
                        update_physics_gizmos(&mut gizmo_config_store, &debug_state);
                    }
                    ui.color_edit_button_rgb(&mut debug_state.contact_normal_color);
                });

                if ui
                    .checkbox(&mut debug_state.show_joints, "Show Joints")
                    .clicked()
                {
                    update_physics_gizmos(&mut gizmo_config_store, &debug_state);
                }

                if ui
                    .checkbox(&mut debug_state.show_raycasts, "Show Raycasts")
                    .clicked()
                {
                    update_physics_gizmos(&mut gizmo_config_store, &debug_state);
                }

                if ui
                    .checkbox(&mut debug_state.show_shapecasts, "Show Shapecasts")
                    .clicked()
                {
                    update_physics_gizmos(&mut gizmo_config_store, &debug_state);
                }

                if ui
                    .checkbox(&mut debug_state.show_axes, "Show Axes")
                    .clicked()
                {
                    update_physics_gizmos(&mut gizmo_config_store, &debug_state);
                }

                ui.separator();

                if ui
                    .checkbox(
                        &mut debug_state.hide_meshes,
                        "Hide Meshes (Show Only Debug)",
                    )
                    .clicked()
                {
                    update_physics_gizmos(&mut gizmo_config_store, &debug_state);
                }

                ui.separator();

                ui.heading("Quick Presets");
                ui.horizontal(|ui| {
                    if ui.button("All On").clicked() {
                        debug_state.show_aabb = true;
                        debug_state.show_colliders = true;
                        debug_state.show_contact_points = true;
                        debug_state.show_contact_normals = true;
                        debug_state.show_joints = true;
                        debug_state.show_raycasts = true;
                        debug_state.show_shapecasts = true;
                        debug_state.show_axes = true;
                        update_physics_gizmos(&mut gizmo_config_store, &debug_state);
                    }

                    if ui.button("All Off").clicked() {
                        debug_state.show_aabb = false;
                        debug_state.show_colliders = false;
                        debug_state.show_contact_points = false;
                        debug_state.show_contact_normals = false;
                        debug_state.show_joints = false;
                        debug_state.show_raycasts = false;
                        debug_state.show_shapecasts = false;
                        debug_state.show_axes = false;
                        debug_state.hide_meshes = false;
                        update_physics_gizmos(&mut gizmo_config_store, &debug_state);
                    }
                });

                ui.horizontal(|ui| {
                    if ui.button("Colliders Only").clicked() {
                        debug_state.show_aabb = false;
                        debug_state.show_colliders = true;
                        debug_state.show_contact_points = false;
                        debug_state.show_contact_normals = false;
                        debug_state.show_joints = false;
                        debug_state.show_raycasts = false;
                        debug_state.show_shapecasts = false;
                        debug_state.show_axes = false;
                        debug_state.hide_meshes = false;
                        update_physics_gizmos(&mut gizmo_config_store, &debug_state);
                    }

                    if ui.button("Physics Only").clicked() {
                        debug_state.show_aabb = true;
                        debug_state.show_colliders = true;
                        debug_state.show_contact_points = true;
                        debug_state.show_contact_normals = true;
                        debug_state.show_joints = false;
                        debug_state.show_raycasts = false;
                        debug_state.show_shapecasts = false;
                        debug_state.show_axes = false;
                        debug_state.hide_meshes = true;
                        update_physics_gizmos(&mut gizmo_config_store, &debug_state);
                    }
                });

                ui.separator();
                ui.label("Press 'M' to toggle this menu");

                ui.separator();
                ui.heading("Instructions");
                ui.label("• AABB: Shows bounding boxes around physics objects");
                ui.label("• Colliders: Shows wireframes of collision shapes");
                ui.label("• Contact Points: Shows red dots where objects touch");
                ui.label("• Contact Normals: Shows blue lines for collision normals");
                ui.label("• Joints: Shows joint connections and constraints");
                ui.label("• Raycasts: Shows debug rays from raycasting");
                ui.label("• Shapecasts: Shows debug shapes from shapecasting");
                ui.label("• Axes: Shows coordinate axes at object centers");
                ui.separator();
                ui.label("Run 'cargo test' to execute integration tests");
            },
        );
}

fn update_physics_gizmos(gizmo_config_store: &mut GizmoConfigStore, debug_state: &DebugMenuState) {
    let any_debug_enabled = debug_state.show_aabb
        || debug_state.show_colliders
        || debug_state.show_contact_points
        || debug_state.show_contact_normals
        || debug_state.show_joints
        || debug_state.show_raycasts
        || debug_state.show_shapecasts
        || debug_state.show_axes;

    let (config, _) = gizmo_config_store.config_mut::<DefaultGizmoConfigGroup>();
    config.enabled = any_debug_enabled;

    config.line.width = if any_debug_enabled { 2.0 } else { 1.0 };

    config.depth_bias = if debug_state.hide_meshes { 0.0 } else { -0.1 };

    if any_debug_enabled {
        let enabled_features: Vec<&str> = [
            ("AABB", debug_state.show_aabb),
            ("Colliders", debug_state.show_colliders),
            ("Contact Points", debug_state.show_contact_points),
            ("Contact Normals", debug_state.show_contact_normals),
            ("Joints", debug_state.show_joints),
            ("Raycasts", debug_state.show_raycasts),
            ("Shapecasts", debug_state.show_shapecasts),
            ("Axes", debug_state.show_axes),
        ]
        .iter()
        .filter_map(|(name, enabled)| if *enabled { Some(*name) } else { None })
        .collect();

        if !enabled_features.is_empty() {
            debug!(
                "Physics debug visualization enabled: {}",
                enabled_features.join(", ")
            );
            if debug_state.hide_meshes {
                debug!("Mesh rendering disabled for debug-only view");
            }
        }
    } else {
        debug!("Physics debug visualization disabled");
    }
}
