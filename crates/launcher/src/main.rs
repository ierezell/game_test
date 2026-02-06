use bevy::prelude::{
    App, AppExtStates, AssetApp, AssetPlugin, DefaultPlugins, Image, Mesh, MinimalPlugins,
    PluginGroup, Shader, StandardMaterial, Window, WindowPlugin, default,
};
use bevy::window::PresentMode;
use client::{
    ClientGameState, Headless, LocalPlayerId, camera::ClientCameraPlugin, debug::ClientDebugPlugin,
    entities::ClientEntitiesPlugin, game::ClientGameCyclePlugin, inputs::ClientInputPlugin,
    lobby::ClientLobbyPlugin, network::ClientNetworkPlugin, vfx::ClientVFXPlugin,
};
use lightyear::prelude::server::ServerPlugins;
use std::time::Duration;

use bevy::log::LogPlugin;
use bevy::render::{
    RenderPlugin as BevyRenderPlugin,
    settings::{Backends, WgpuSettings},
};

use server::{
    ServerGameState, entities::ServerEntitiesPlugin, lobby::ServerLobbyPlugin,
    network::ServerNetworkPlugin, render::RenderPlugin,
};
use shared::{NetworkMode, SharedPlugin};

use lightyear::prelude::client::ClientPlugins;

fn main() {
    #[cfg(target_family = "wasm")]
    launcher::wasm::run();

    #[cfg(not(target_family = "wasm"))]
    launcher::native::run();
}

pub fn create_host_app(headless: bool, network_mode: NetworkMode, asset_path: String) -> App {
    let mut host_app = App::new();
    let client_id = 1;

    host_app.insert_resource(Headless(headless));
    host_app.insert_resource(network_mode);
    host_app.add_plugins(SharedPlugin);
    host_app.add_plugins(ServerPlugins {
        tick_duration: Duration::from_secs_f64(1.0 / shared::FIXED_TIMESTEP_HZ),
    });
    host_app.add_plugins(ServerNetworkPlugin);
    host_app.add_plugins(ServerLobbyPlugin);
    host_app.add_plugins(ServerEntitiesPlugin);
    host_app.init_state::<ServerGameState>();
    host_app.insert_state(ServerGameState::Lobby);

    if headless {
        // Add AssetPlugin first to enable asset initialization
        host_app.add_plugins(MinimalPlugins);
        // LogPlugin disabled for tests to avoid "Could not set global logger" error
        // bevy::log::LogPlugin::default(),
        host_app.add_plugins(bevy::state::app::StatesPlugin);
        host_app.add_plugins(bevy::diagnostic::DiagnosticsPlugin);
        host_app.add_plugins(bevy::asset::AssetPlugin::default());
        host_app.add_plugins(bevy::scene::ScenePlugin);
        host_app.add_plugins(bevy::mesh::MeshPlugin);
        host_app.add_plugins(bevy::animation::AnimationPlugin);
        host_app.add_plugins(AssetPlugin {
            file_path: asset_path.clone(),
            ..Default::default()
        });

        // Manually initialize assets that are usually added by RenderPlugin/PbrPlugin
        // This must happen BEFORE other plugins in DefaultPlugins (like UiPlugin) try to use them
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
                .disable::<AssetPlugin>() // Already added manually
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
        host_app
            .add_plugins(
                DefaultPlugins
                    .set(WindowPlugin {
                        primary_window: Some(Window {
                            title: format!("Game Test - Client {}", client_id),
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
                    })
                    .disable::<LogPlugin>(),
            )
            .add_plugins(RenderPlugin);
    }

    host_app.insert_resource(network_mode);
    host_app.add_plugins(shared::SharedPlugin);
    host_app.add_plugins(ClientPlugins {
        tick_duration: Duration::from_secs_f64(1.0 / shared::FIXED_TIMESTEP_HZ),
    });

    host_app.insert_resource(LocalPlayerId(client_id));
    host_app.add_plugins(ClientNetworkPlugin);
    host_app.add_plugins(ClientInputPlugin);
    host_app.add_plugins(ClientCameraPlugin);

    host_app.add_plugins(ClientEntitiesPlugin);
    host_app.add_plugins(ClientLobbyPlugin);
    host_app.add_plugins(ClientGameCyclePlugin);

    host_app.init_state::<ClientGameState>();
    host_app.insert_state(ClientGameState::LocalMenu);

    if !headless {
        host_app.add_plugins(ClientDebugPlugin);
        host_app.add_plugins(ClientVFXPlugin);
    }

    host_app
}
