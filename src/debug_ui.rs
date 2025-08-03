#![allow(dead_code)]

use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPlugin, egui};
use bevy_inspector_egui::quick::WorldInspectorPlugin;

/// Debug UI plugin for physics debugging with EGUI
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

/// Resource to track debug menu state
#[derive(Resource, Default)]
pub struct DebugMenuState {
    pub show_menu: bool,
    pub show_inspector: bool, // Add entity inspector toggle
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

impl DebugMenuState {
    pub fn new() -> Self {
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
            aabb_color: [1.0, 1.0, 0.0],           // Yellow
            collider_color: [0.0, 1.0, 0.0],       // Green
            contact_point_color: [1.0, 0.0, 0.0],  // Red
            contact_normal_color: [0.0, 0.0, 1.0], // Blue
        }
    }
}

/// System to handle debug menu input (M key)
pub fn debug_input_system(
    mut debug_state: ResMut<DebugMenuState>,
    input: Res<ButtonInput<KeyCode>>,
    _commands: Commands,
    _meshes: ResMut<Assets<Mesh>>,
    _materials: ResMut<Assets<StandardMaterial>>,
) {
    if input.just_pressed(KeyCode::KeyM) {
        debug_state.show_menu = !debug_state.show_menu;
        info!("Debug menu toggled: {}", debug_state.show_menu);
    }
}

/// System to render the debug UI
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

                // Entity Inspector Toggle
                ui.horizontal(|ui| {
                    ui.checkbox(&mut debug_state.show_inspector, "Show Entity Inspector");
                    ui.label("Browse all entities in the world");
                });

                ui.separator();
                ui.heading("Physics Debug Visualization");
                ui.separator();

                // AABB Debug
                ui.horizontal(|ui| {
                    if ui
                        .checkbox(&mut debug_state.show_aabb, "Show AABB")
                        .clicked()
                    {
                        update_physics_gizmos(&mut gizmo_config_store, &debug_state);
                    }
                    ui.color_edit_button_rgb(&mut debug_state.aabb_color);
                });

                // Collider Debug
                ui.horizontal(|ui| {
                    if ui
                        .checkbox(&mut debug_state.show_colliders, "Show Colliders")
                        .clicked()
                    {
                        update_physics_gizmos(&mut gizmo_config_store, &debug_state);
                    }
                    ui.color_edit_button_rgb(&mut debug_state.collider_color);
                });

                // Contact Points Debug
                ui.horizontal(|ui| {
                    if ui
                        .checkbox(&mut debug_state.show_contact_points, "Show Contact Points")
                        .clicked()
                    {
                        update_physics_gizmos(&mut gizmo_config_store, &debug_state);
                    }
                    ui.color_edit_button_rgb(&mut debug_state.contact_point_color);
                });

                // Contact Normals Debug
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

                // Joints Debug
                if ui
                    .checkbox(&mut debug_state.show_joints, "Show Joints")
                    .clicked()
                {
                    update_physics_gizmos(&mut gizmo_config_store, &debug_state);
                }

                // Raycasts Debug
                if ui
                    .checkbox(&mut debug_state.show_raycasts, "Show Raycasts")
                    .clicked()
                {
                    update_physics_gizmos(&mut gizmo_config_store, &debug_state);
                }

                // Shapecasts Debug
                if ui
                    .checkbox(&mut debug_state.show_shapecasts, "Show Shapecasts")
                    .clicked()
                {
                    update_physics_gizmos(&mut gizmo_config_store, &debug_state);
                }

                // Axes Debug
                if ui
                    .checkbox(&mut debug_state.show_axes, "Show Axes")
                    .clicked()
                {
                    update_physics_gizmos(&mut gizmo_config_store, &debug_state);
                }

                ui.separator();

                // Hide Meshes Option
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

                // Quick presets
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

/// Helper function to update physics gizmos configuration
fn update_physics_gizmos(
    _gizmo_config_store: &mut GizmoConfigStore,
    _debug_state: &DebugMenuState,
) {
    // TODO: Update physics gizmos configuration for newer Avian3D version
    // The API has changed - need to investigate new PhysicsGizmos configuration
    info!("Physics gizmos configuration update placeholder");
}
