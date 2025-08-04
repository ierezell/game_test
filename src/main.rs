use avian3d::prelude::*;
use bevy::prelude::*;

mod audio;
mod combat;
mod debug_ui;
mod enemies;

mod menu;
mod player;
mod static_level;
mod ui;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "GTFO-Like Game".to_string(),
                resolution: (800.0, 600.0).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(PhysicsPlugins::default())
        .add_plugins(PhysicsDebugPlugin::default())
        .init_state::<menu::GameState>()
        .add_plugins((
            player::PlayerPlugin,
            combat::CombatPlugin,
            enemies::EnemyPlugin,
            ui::UIPlugin,
            audio::GameAudioPlugin,
            debug_ui::DebugUIPlugin,
            menu::MenuPlugin,
            static_level::StaticLevelPlugin,
        ))
        .add_systems(Startup, (setup_camera, setup_basic_light))
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 1.7, 5.0).looking_at(Vec3::ONE, Vec3::Y),
        Name::new("First Person Camera"),
        player::FirstPersonCamera,
    ));
}

fn setup_basic_light(mut commands: Commands) {
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.8,
        affects_lightmapped_meshes: true,
    });
}
