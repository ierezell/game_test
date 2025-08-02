use avian3d::prelude::*;
use bevy::prelude::*;

mod audio;
mod combat;
mod debug_ui;
mod enemies;
mod environment;
mod game_state;
mod menu;
mod networking;
mod player;
mod static_level;
mod ui;
mod utils;

use game_state::GameState;

fn main() {
    App::new()
        // Bevy plugins
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "GTFO-Like Game".to_string(),
                resolution: (1920.0, 1080.0).into(),
                ..default()
            }),
            ..default()
        }))
        // Physics
        .add_plugins(PhysicsPlugins::default())
        .add_plugins(PhysicsDebugPlugin::default()) // Enable physics debug rendering
        // Game state
        .init_state::<GameState>()
        // Game systems
        .add_plugins((
            player::PlayerPlugin,
            combat::CombatPlugin,
            environment::EnvironmentPlugin,
            enemies::EnemyPlugin,
            ui::UIPlugin,
            audio::GameAudioPlugin,
            debug_ui::DebugUIPlugin,
            menu::MenuPlugin,
            networking::NetworkingPlugin,
            static_level::StaticLevelPlugin,
        ))
        // Startup systems
        .add_systems(
            Startup,
            (setup_camera, setup_basic_scene, setup_tension_system),
        )
        // Update systems
        .add_systems(
            Update,
            (
                utils::trigger_random_event,
                update_tension_system,
                utils::update_muzzle_flashes,
            ),
        )
        .run();
}

fn setup_camera(mut commands: Commands) {
    // Spawn a first-person camera for GTFO-like experience
    // The camera will be positioned by the player system
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 1.7, 5.0),
        Name::new("First Person Camera"),
        player::FirstPersonCamera,
    ));
}

fn setup_basic_scene(
    mut commands: Commands,
    _meshes: ResMut<Assets<Mesh>>,
    _materials: ResMut<Assets<StandardMaterial>>,
) {
    // Bright ambient lighting so player can see everything clearly
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.8,
        affects_lightmapped_meshes: true,
    });
}

fn setup_tension_system(mut commands: Commands) {
    commands.insert_resource(utils::TensionSystem::default());
    commands.insert_resource(utils::EventSystem::default());
    info!("Tension and event systems initialized");
}

fn update_tension_system(time: Res<Time>, mut tension_system: ResMut<utils::TensionSystem>) {
    tension_system.update(time.delta_secs(), time.elapsed_secs());
}
