use bevy::prelude::{
    App, AppExtStates, AssetApp, AssetPlugin, DefaultPlugins, Image, Mesh, PluginGroup, Shader,
    StandardMaterial, Window, WindowPlugin, default,
};
use bevy::window::PresentMode;
use client::{
    ClientGameState, Headless, LocalPlayerId, camera::ClientCameraPlugin, debug::ClientDebugPlugin,
    entities::ClientEntitiesPlugin, game::ClientGameCyclePlugin, hud::ClientHudPlugin,
    inputs::ClientInputPlugin, lobby::ClientLobbyPlugin, network::ClientNetworkPlugin,
    vfx::ClientVFXPlugin,
};
use lightyear::prelude::server::ServerPlugins;
use std::time::Duration;

use bevy::log::LogPlugin;
use bevy::render::{
    RenderPlugin as BevyRenderPlugin,
    settings::{Backends, WgpuSettings},
};

use server::{
    ServerGameState, debug::ServerDebugPlugin, entities::ServerEntitiesPlugin,
    lobby::ServerLobbyPlugin, network::ServerNetworkPlugin,
};
use shared::{NetworkMode, SharedPlugin};

use lightyear::prelude::client::ClientPlugins;

pub fn create_host_app(headless: bool, asset_path: String) -> App {
    let mut host_app = App::new();
    // In local host mode the host peer is the local server itself, which
    // lightyear represents as `PeerId::Local(0)`. Match the lobby host id so
    // the host is correctly recognised client-side (play button, camera).
    let local_player_id: u64 = 0;

    host_app.insert_resource(Headless(headless));
    host_app.insert_resource(NetworkMode::Local);

    // `bevy_ui_widgets` registers a text-input system that reads `UiScale`,
    // which is normally provided by Bevy's UI plugin. In headless builds that
    // plugin is disabled, so insert the resource explicitly (the headless test
    // apps do the same) to avoid a missing-resource panic.
    if headless {
        host_app.insert_resource(bevy::ui::UiScale::default());
    }

    if headless {
        // In headless builds `WinitPlugin` (the usual main-loop runner) is
        // disabled, so `App::run()` would exit immediately after startup and the
        // lobby would never auto-start. `ScheduleRunnerPlugin` is the minimal
        // headless runner that drives `Update`/`FixedUpdate` at real time. We add
        // only the runner here (not `MinimalPlugins`) because `DefaultPlugins`
        // already supplies `TaskPoolPlugin`/`Time`/etc.; adding `MinimalPlugins`
        // too would double-register them and panic.
        host_app.add_plugins(bevy::app::ScheduleRunnerPlugin::default());

        // AssetPlugin must be added before `init_asset` (and before the plugins
        // that call `init_asset` during their own `build`), so that the
        // `AssetServer` resource exists by the time assets are registered.
        host_app.add_plugins(AssetPlugin {
            file_path: asset_path.clone(),
            ..Default::default()
        });

        host_app.init_asset::<Mesh>();
        host_app.init_asset::<StandardMaterial>();
        host_app.init_asset::<Shader>();
        host_app.init_asset::<Image>();

        host_app.add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: None,
                    exit_condition: bevy::window::ExitCondition::DontExit,
                    ..default()
                })
                .disable::<AssetPlugin>()
                .disable::<LogPlugin>()
                .disable::<bevy::winit::WinitPlugin>()
                .disable::<bevy::render::RenderPlugin>()
                .disable::<bevy::pbr::PbrPlugin>()
                .disable::<bevy::sprite::SpritePlugin>()
                .disable::<bevy::audio::AudioPlugin>()
                .disable::<bevy::gilrs::GilrsPlugin>()
                .disable::<bevy::ui::UiPlugin>()
                .disable::<bevy::text::TextPlugin>(),
        );
    } else {
        host_app.add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Game Test - Host".to_string(),
                        resolution: (1280, 720).into(),
                        present_mode: PresentMode::AutoVsync,
                        ..default()
                    }),
                    ..default()
                })
                .set(BevyRenderPlugin {
                    render_creation: WgpuSettings {
                        backends: Some(Backends::VULKAN | Backends::DX12 | Backends::METAL),
                        ..default()
                    }
                    .into(),
                    ..default()
                })
                .set(AssetPlugin {
                    file_path: asset_path,
                    ..Default::default()
                }),
        );
    }

    host_app.add_plugins(ServerPlugins {
        tick_duration: Duration::from_secs_f64(1.0 / shared::FIXED_TIMESTEP_HZ),
    });
    host_app.add_plugins(ServerNetworkPlugin);
    host_app.add_plugins(ServerLobbyPlugin);
    host_app.add_plugins(ServerEntitiesPlugin);
    host_app.add_plugins(ServerDebugPlugin);
    host_app.init_state::<ServerGameState>();
    host_app.insert_state(ServerGameState::Lobby);

    host_app.add_plugins(ClientPlugins {
        tick_duration: Duration::from_secs_f64(1.0 / shared::FIXED_TIMESTEP_HZ),
    });
    host_app.insert_resource(lightyear::prelude::PredictionManager::default());
    host_app.add_plugins(SharedPlugin);

    host_app.insert_resource(LocalPlayerId(local_player_id));

    host_app.add_plugins(ClientNetworkPlugin);
    host_app.add_plugins(ClientInputPlugin);
    host_app.add_plugins(ClientCameraPlugin);

    host_app.add_plugins(ClientEntitiesPlugin);
    host_app.add_plugins(ClientLobbyPlugin);
    host_app.add_plugins(ClientGameCyclePlugin);
    host_app.add_plugins(ClientHudPlugin);
    host_app.add_plugins(shared::inputs::movement::MovementPlugin::new(headless));

    host_app.init_state::<ClientGameState>();
    host_app.insert_state(ClientGameState::Lobby);

    if !headless {
        host_app.add_plugins(ClientDebugPlugin);
        host_app.add_plugins(ClientVFXPlugin);
    }

    host_app
}
