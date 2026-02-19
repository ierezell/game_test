#![allow(dead_code)]

use bevy::MinimalPlugins;
use bevy::log::LogPlugin;
use bevy::prelude::{
    App, AssetApp, AssetPlugin, DefaultPlugins, Image, Mesh, PluginGroup, Shader, StandardMaterial,
    Vec3, default,
};
use bevy::state::app::AppExtStates;
use bevy::window::WindowPlugin;
use client::camera::ClientCameraPlugin;
use client::entities::ClientEntitiesPlugin;
use client::game::ClientGameCyclePlugin;
use client::inputs::ClientInputPlugin;
use client::lobby::ClientLobbyPlugin;
use client::network::ClientNetworkPlugin;
use client::{ClientGameState, Headless, LocalPlayerId};
use lightyear::prelude::client::ClientPlugins;
use lightyear::prelude::server::ServerPlugins;
use server::ServerGameState;
use server::entities::ServerEntitiesPlugin;
use server::lobby::ServerLobbyPlugin;
use server::network::ServerNetworkPlugin;
use shared::{NetworkMode, SharedPlugin};
use std::time::Duration;

#[cfg(feature = "legacy_udp_tests")]
mod app_flow;
#[cfg(feature = "legacy_udp_tests")]
mod ccc;
#[cfg(feature = "legacy_udp_tests")]
mod gameplay;
#[cfg(feature = "legacy_udp_tests")]
mod world;
mod deterministic_stepper;

fn update_all(server_app: &mut App, client_app1: &mut App, client_app2: &mut App) {
    let dt = Duration::from_millis(16);
    server_app.insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(dt));
    client_app1.insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(dt));
    client_app2.insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(dt));
    server_app.update();
    client_app1.update();
    client_app2.update();
}

fn update_all_with_third_client(
    server_app: &mut App,
    client_app1: &mut App,
    client_app2: &mut App,
    client_app3: &mut App,
) {
    let dt = Duration::from_millis(16);
    server_app.insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(dt));
    client_app1.insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(dt));
    client_app2.insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(dt));
    client_app3.insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(dt));
    server_app.update();
    client_app1.update();
    client_app2.update();
    client_app3.update();
}

fn update_pair(server_app: &mut App, client_app: &mut App) {
    let dt = Duration::from_millis(16);
    server_app.insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(dt));
    client_app.insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(dt));
    server_app.update();
    client_app.update();
}

fn update_single_app(app: &mut App, dt: Duration) {
    app.insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(dt));
    app.update();
}

fn server_lobby_player_count(server_app: &mut App) -> usize {
    let world = server_app.world_mut();
    let mut q = world.query::<&shared::protocol::LobbyState>();
    q.iter(world).next().map_or(0, |lobby| lobby.players.len())
}

fn try_send_host_start(client_app: &mut App) -> bool {
    use lightyear::prelude::{Client, MessageSender};
    use shared::protocol::{HostStartGameEvent, LobbyControlChannel};

    let world = client_app.world_mut();
    let mut q = world.query_filtered::<
        &mut MessageSender<HostStartGameEvent>,
        bevy::prelude::With<Client>,
    >();
    if let Some(mut sender) = q.iter_mut(world).next() {
        sender.send::<LobbyControlChannel>(HostStartGameEvent { requested: true });
        true
    } else {
        false
    }
}

fn wait_until_all_playing(server_app: &mut App, client_app1: &mut App, client_app2: &mut App) {
    let mut sent_start = false;
    let mut sent_attempts = 0;

    for _ in 0..600 {
        update_all(server_app, client_app1, client_app2);

        let server_state_now = server_app
            .world()
            .resource::<bevy::prelude::State<ServerGameState>>()
            .get()
            .clone();

        if server_lobby_player_count(server_app) >= 2
            && server_state_now == ServerGameState::Lobby
            && try_send_host_start(client_app1)
        {
            sent_start = true;
            sent_attempts += 1;
        }

        let server_playing = server_state_now == ServerGameState::Playing;
        let client1_playing = client_app1
            .world()
            .resource::<bevy::prelude::State<ClientGameState>>()
            .get()
            == &ClientGameState::Playing;
        let client2_playing = client_app2
            .world()
            .resource::<bevy::prelude::State<ClientGameState>>()
            .get()
            == &ClientGameState::Playing;

        if server_playing && client1_playing && client2_playing {
            return;
        }
    }

    let server_state = server_app
        .world()
        .resource::<bevy::prelude::State<ServerGameState>>()
        .get()
        .clone();
    let client1_state = client_app1
        .world()
        .resource::<bevy::prelude::State<ClientGameState>>()
        .get()
        .clone();
    let client2_state = client_app2
        .world()
        .resource::<bevy::prelude::State<ClientGameState>>()
        .get()
        .clone();

    let server_receiver_stats = {
        use lightyear::prelude::MessageReceiver;
        use shared::protocol::HostStartGameEvent;

        let world = server_app.world_mut();
        let mut q = world.query::<&mut MessageReceiver<HostStartGameEvent>>();
        let mut receiver_count = 0usize;
        let mut buffered_messages = 0usize;
        for receiver in q.iter_mut(world) {
            receiver_count += 1;
            buffered_messages += receiver.num_messages();
        }
        (receiver_count, buffered_messages)
    };

    let client_sender_count = {
        use lightyear::prelude::MessageSender;
        use shared::protocol::HostStartGameEvent;

        let world = client_app1.world_mut();
        let mut q = world.query::<&MessageSender<HostStartGameEvent>>();
        q.iter(world).count()
    };

    let client_transport_lobby_channel = {
        use lightyear::prelude::{Client, Connected, Transport};
        use shared::protocol::LobbyControlChannel;

        let world = client_app1.world_mut();
        let mut q = world.query_filtered::<&Transport, (bevy::prelude::With<Client>, bevy::prelude::With<Connected>)>();
        q.iter(world)
            .next()
            .map(|transport| {
                (
                    transport.has_sender::<LobbyControlChannel>(),
                    transport.has_receiver::<LobbyControlChannel>(),
                )
            })
    };

    let server_transport_lobby_channel = {
        use lightyear::connection::client_of::ClientOf;
        use lightyear::prelude::{Connected, Transport};
        use shared::protocol::LobbyControlChannel;

        let world = server_app.world_mut();
        let mut q = world.query_filtered::<&Transport, (bevy::prelude::With<ClientOf>, bevy::prelude::With<Connected>)>();
        q.iter(world)
            .next()
            .map(|transport| {
                (
                    transport.has_sender::<LobbyControlChannel>(),
                    transport.has_receiver::<LobbyControlChannel>(),
                )
            })
    };

    let client_replication_visibility = {
        use lightyear::prelude::Confirmed;
        use shared::protocol::{LevelSeed, LobbyState};

        let world = client_app1.world_mut();
        let mut lobby_q = world.query::<&LobbyState>();
        let mut confirmed_lobby_q = world.query::<&Confirmed<LobbyState>>();
        let mut seed_q = world.query::<&LevelSeed>();
        let mut confirmed_seed_q = world.query::<&Confirmed<LevelSeed>>();

        (
            lobby_q.iter(world).count(),
            confirmed_lobby_q.iter(world).count(),
            seed_q.iter(world).count(),
            confirmed_seed_q.iter(world).count(),
        )
    };

    let client_link_replication_components = {
        use lightyear::prelude::{Client, Connected, ReplicationReceiver, ReplicationSender};

        let world = client_app1.world_mut();
        let mut q = world.query_filtered::<
            (
                Option<&ReplicationSender>,
                Option<&ReplicationReceiver>,
            ),
            (bevy::prelude::With<Client>, bevy::prelude::With<Connected>),
        >();

        q.iter(world)
            .next()
            .map(|(sender, receiver)| (sender.is_some(), receiver.is_some()))
    };

    panic!(
        "Timed out waiting for all peers to enter Playing. sent_start={}, attempts={}, server={:?}, client1={:?}, client2={:?}, client1_host_start_senders={}, server_host_start_receivers={}, server_host_start_buffered={}, client_lobby_channel={:?}, server_lobby_channel={:?}, client1_replication=(lobby:{}, confirmed_lobby:{}, seed:{}, confirmed_seed:{}), client1_link_replication={:?}",
        sent_start,
        sent_attempts,
        server_state,
        client1_state,
        client2_state,
        client_sender_count,
        server_receiver_stats.0,
        server_receiver_stats.1,
        client_transport_lobby_channel,
        server_transport_lobby_channel,
        client_replication_visibility.0,
        client_replication_visibility.1,
        client_replication_visibility.2,
        client_replication_visibility.3,
        client_link_replication_components,
    );
}

fn server_player_entity(server_app: &mut App, id: u64) -> Option<bevy::prelude::Entity> {
    use shared::protocol::PlayerId;

    let world = server_app.world_mut();
    let mut q = world.query::<(bevy::prelude::Entity, &PlayerId)>();
    q.iter(world)
        .find_map(|(entity, player_id)| match player_id.0 {
            lightyear::prelude::PeerId::Netcode(pid) if pid == id => Some(entity),
            _ => None,
        })
}

fn server_player_position_by_entity(
    server_app: &mut App,
    entity: bevy::prelude::Entity,
) -> Option<Vec3> {
    use avian3d::prelude::Position;

    let world = server_app.world_mut();
    world.get::<Position>(entity).map(|position| position.0)
}

fn client_interpolated_player_position(client_app: &mut App, id: u64) -> Option<Vec3> {
    use avian3d::prelude::Position;
    use lightyear::prelude::Confirmed;
    use lightyear::prelude::Interpolated;
    use shared::protocol::PlayerId;

    let world = client_app.world_mut();
    let mut q = world.query_filtered::<(&PlayerId, &Position), bevy::prelude::With<Interpolated>>();
    if let Some(position) = q
        .iter(world)
        .find_map(|(player_id, position)| match player_id.0 {
            lightyear::prelude::PeerId::Netcode(pid) if pid == id => Some(position.0),
            _ => None,
        })
    {
        return Some(position);
    }

    let mut confirmed_q = world.query_filtered::<
        (&Confirmed<PlayerId>, &Confirmed<Position>),
        bevy::prelude::With<Interpolated>,
    >();
    confirmed_q
        .iter(world)
        .find_map(|(player_id, position)| match player_id.0.0 {
            lightyear::prelude::PeerId::Netcode(pid) if pid == id => Some(position.0.0),
            _ => None,
        })
}

fn client_has_local_predicted_player(client_app: &mut App, id: u64) -> bool {
    use lightyear::prelude::{Confirmed, Controlled, Predicted};
    use shared::protocol::PlayerId;

    let world = client_app.world_mut();

    let mut direct_q = world.query_filtered::<
        &PlayerId,
        (
            bevy::prelude::With<Predicted>,
            bevy::prelude::With<Controlled>,
        ),
    >();
    if direct_q
        .iter(world)
        .any(|pid| matches!(pid.0, lightyear::prelude::PeerId::Netcode(pid) if pid == id))
    {
        return true;
    }

    let mut confirmed_q = world.query_filtered::<
        &Confirmed<PlayerId>,
        (
            bevy::prelude::With<Predicted>,
            bevy::prelude::With<Controlled>,
        ),
    >();
    confirmed_q
        .iter(world)
        .any(|pid| matches!(pid.0.0, lightyear::prelude::PeerId::Netcode(pid) if pid == id))
}

fn client_has_remote_interpolated_player(client_app: &mut App, id: u64) -> bool {
    use lightyear::prelude::{Confirmed, Interpolated};
    use shared::protocol::PlayerId;

    let world = client_app.world_mut();

    let mut direct_q = world.query_filtered::<&PlayerId, bevy::prelude::With<Interpolated>>();
    if direct_q
        .iter(world)
        .any(|pid| matches!(pid.0, lightyear::prelude::PeerId::Netcode(pid) if pid == id))
    {
        return true;
    }

    let mut confirmed_q =
        world.query_filtered::<&Confirmed<PlayerId>, bevy::prelude::With<Interpolated>>();
    confirmed_q
        .iter(world)
        .any(|pid| matches!(pid.0.0, lightyear::prelude::PeerId::Netcode(pid) if pid == id))
}

fn wait_for_client_player_views(
    server_app: &mut App,
    client_app1: &mut App,
    client_app2: &mut App,
    max_ticks: usize,
) -> bool {
    for _ in 0..max_ticks {
        update_all(server_app, client_app1, client_app2);

        let c1_local = client_has_local_predicted_player(client_app1, 1);
        let c1_remote = client_has_remote_interpolated_player(client_app1, 2);
        let c2_local = client_has_local_predicted_player(client_app2, 2);
        let c2_remote = client_has_remote_interpolated_player(client_app2, 1);

        if c1_local && c1_remote && c2_local && c2_remote {
            return true;
        }
    }

    false
}

fn translate_server_player_entity(
    server_app: &mut App,
    entity: bevy::prelude::Entity,
    delta: Vec3,
) -> bool {
    use avian3d::prelude::Position;

    let world = server_app.world_mut();
    if let Some(mut position) = world.get_mut::<Position>(entity) {
        position.0 += delta;
        true
    } else {
        false
    }
}

fn snap_vec3(value: Vec3) -> Vec3 {
    let scale = 1_000_000.0;
    Vec3::new(
        (value.x * scale).round() / scale,
        (value.y * scale).round() / scale,
        (value.z * scale).round() / scale,
    )
}

fn first_level_seed(app: &mut App) -> Option<u64> {
    use lightyear::prelude::Confirmed;
    use shared::protocol::LevelSeed;

    let world = app.world_mut();
    let mut q = world.query::<&LevelSeed>();
    if let Some(seed) = q.iter(world).next() {
        return Some(seed.seed);
    }

    let mut confirmed_q = world.query::<&Confirmed<LevelSeed>>();
    confirmed_q.iter(world).next().map(|seed| seed.0.seed)
}

fn assert_level_exists(app: &mut App, peer_name: &str, min_characters: usize) {
    use avian3d::prelude::Collider;
    use shared::gym::LevelDoneMarker;
    use shared::protocol::CharacterMarker;

    let world = app.world_mut();

    let mut level_q = world.query::<&LevelDoneMarker>();
    let mut collider_q = world.query::<&Collider>();
    let mut character_q = world.query::<&CharacterMarker>();

    let level_count = level_q.iter(world).count();
    let collider_count = collider_q.iter(world).count();
    let character_count = character_q.iter(world).count();

    assert!(
        level_count >= 1,
        "{} should have gym level marker, found {}",
        peer_name,
        level_count
    );
    assert!(
        collider_count >= 6,
        "{} should have level colliders, found {}",
        peer_name,
        collider_count
    );
    if min_characters > 0 {
        assert!(
            character_count >= min_characters,
            "{} should have at least {} character entities, found {}",
            peer_name,
            min_characters,
            character_count
        );
    }
}

fn create_test_client_app_with_mode(client_id: u64, gym_mode: bool, network_mode: NetworkMode) -> App {
    create_test_client_app_with_mode_and_endpoint(client_id, gym_mode, network_mode, None)
}

fn create_test_client_app_with_mode_and_endpoint(
    client_id: u64,
    gym_mode: bool,
    network_mode: NetworkMode,
    crossbeam_endpoint: Option<client::network::CrossbeamClientEndpoint>,
) -> App {
    let mut client_app = App::new();
    let client_id = if client_id == 0 { 1 } else { client_id };
    client_app.insert_resource(Headless(true));
    client_app.add_plugins(AssetPlugin {
        file_path: "../../../../assets".to_string(),
        ..Default::default()
    });

    client_app.init_asset::<Mesh>();
    client_app.init_asset::<StandardMaterial>();
    client_app.init_asset::<Shader>();
    client_app.init_asset::<Image>();

    client_app.add_plugins(
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

    client_app.insert_resource(network_mode);
    if let Some(endpoint) = crossbeam_endpoint {
        client_app.insert_resource(endpoint);
    }
    client_app.insert_resource(shared::GymMode(gym_mode));
    client_app.add_plugins(SharedPlugin);
    client_app.add_plugins(ClientPlugins {
        tick_duration: Duration::from_secs_f64(1.0 / shared::FIXED_TIMESTEP_HZ),
    });

    client_app.insert_resource(LocalPlayerId(client_id));
    client_app.add_plugins(ClientNetworkPlugin);
    client_app.add_plugins(ClientInputPlugin);
    client_app.add_plugins(ClientCameraPlugin);

    client_app.add_plugins(ClientEntitiesPlugin);
    client_app.add_plugins(ClientLobbyPlugin);
    client_app.add_plugins(ClientGameCyclePlugin);

    client_app.init_state::<ClientGameState>();
    client_app.insert_state(ClientGameState::Lobby);

    client_app
}

fn create_crossbeam_pair() -> (
    client::network::CrossbeamClientEndpoint,
    lightyear::crossbeam::CrossbeamIo,
) {
    let (client_io, server_io) = lightyear::crossbeam::CrossbeamIo::new_pair();
    (
        client::network::CrossbeamClientEndpoint(client_io),
        server_io,
    )
}

fn add_server_clientof(
    server_app: &mut App,
    client_id: u64,
    server_io: lightyear::crossbeam::CrossbeamIo,
) {
    use lightyear::prelude::server::{ClientOf, Server};
    use lightyear::prelude::{
        Connected, Link, LinkOf, Linked, LocalId, PeerId, PingConfig, PingManager, RemoteId,
        ReplicationReceiver, ReplicationSender, Transport,
    };

    let server_world = server_app.world_mut();
    let server_entity = server_world
        .query_filtered::<bevy::prelude::Entity, bevy::prelude::With<Server>>()
        .single(server_world)
        .expect("Server entity should exist before adding crossbeam ClientOf links");

    server_world.spawn((
        ClientOf,
        Connected,
        LinkOf {
            server: server_entity,
        },
        Link::new(None),
        Linked,
        server_io,
        Transport::default(),
        RemoteId(PeerId::Netcode(client_id)),
        LocalId(PeerId::Server),
        PingManager::new(PingConfig {
            ping_interval: Duration::default(),
        }),
        ReplicationSender::default(),
        ReplicationReceiver::default(),
        bevy::prelude::Name::from(format!("ClientOf {}", client_id)),
    ));
}

fn setup_two_client_server(gym_mode: bool) -> (App, App, App) {
    let mut server_app = create_test_server_app_with_mode(gym_mode, NetworkMode::Crossbeam);

    let (client1_endpoint, server1_io) = create_crossbeam_pair();
    let (client2_endpoint, server2_io) = create_crossbeam_pair();

    let mut client_app1 = create_test_client_app_with_mode_and_endpoint(
        1,
        gym_mode,
        NetworkMode::Crossbeam,
        Some(client1_endpoint),
    );
    let mut client_app2 = create_test_client_app_with_mode_and_endpoint(
        2,
        gym_mode,
        NetworkMode::Crossbeam,
        Some(client2_endpoint),
    );

    for _ in 0..4 {
        server_app.update();
        client_app1.update();
        client_app2.update();
    }

    add_server_clientof(&mut server_app, 1, server1_io);
    add_server_clientof(&mut server_app, 2, server2_io);

    for _ in 0..4 {
        server_app.update();
        client_app1.update();
        client_app2.update();
    }

    (server_app, client_app1, client_app2)
}

fn setup_one_client_server(gym_mode: bool) -> (App, App) {
    let mut server_app = create_test_server_app_with_mode(gym_mode, NetworkMode::Crossbeam);
    let (client_endpoint, server_io) = create_crossbeam_pair();
    let mut client_app = create_test_client_app_with_mode_and_endpoint(
        1,
        gym_mode,
        NetworkMode::Crossbeam,
        Some(client_endpoint),
    );

    for _ in 0..4 {
        server_app.update();
        client_app.update();
    }

    add_server_clientof(&mut server_app, 1, server_io);

    for _ in 0..4 {
        server_app.update();
        client_app.update();
    }

    (server_app, client_app)
}

fn attach_crossbeam_client(server_app: &mut App, client_id: u64, gym_mode: bool) -> App {
    let (client_endpoint, server_io) = create_crossbeam_pair();
    let mut client_app = create_test_client_app_with_mode_and_endpoint(
        client_id,
        gym_mode,
        NetworkMode::Crossbeam,
        Some(client_endpoint),
    );

    for _ in 0..3 {
        server_app.update();
        client_app.update();
    }

    add_server_clientof(server_app, client_id, server_io);

    for _ in 0..3 {
        server_app.update();
        client_app.update();
    }

    client_app
}

fn create_test_client_app_with_gym_mode(client_id: u64, gym_mode: bool) -> App {
    create_test_client_app_with_mode(client_id, gym_mode, NetworkMode::Local)
}

fn create_test_client_app(client_id: u64) -> App {
    create_test_client_app_with_gym_mode(client_id, true)
}

fn create_test_server_app_with_mode(gym_mode: bool, network_mode: NetworkMode) -> App {
    let mut app = App::new();

    app.add_plugins((
        MinimalPlugins,
        bevy::state::app::StatesPlugin,
        bevy::diagnostic::DiagnosticsPlugin,
        bevy::asset::AssetPlugin::default(),
        bevy::scene::ScenePlugin,
        bevy::mesh::MeshPlugin,
        bevy::animation::AnimationPlugin,
    ));

    app.insert_resource(network_mode);
    app.insert_resource(shared::GymMode(gym_mode));
    app.add_plugins(SharedPlugin);
    app.add_plugins(ServerPlugins {
        tick_duration: Duration::from_secs_f64(1.0 / shared::FIXED_TIMESTEP_HZ),
    });
    app.add_plugins(ServerNetworkPlugin);
    app.add_plugins(ServerLobbyPlugin);
    app.add_plugins(ServerEntitiesPlugin);
    app.insert_resource(server::lobby::AutoStartOnLobbyReady(false));
    app.init_state::<ServerGameState>();
    app.insert_state(ServerGameState::Lobby);

    app
}

fn create_test_server_app_with_gym_mode(gym_mode: bool) -> App {
    create_test_server_app_with_mode(gym_mode, NetworkMode::Local)
}

fn create_test_server_app() -> App {
    create_test_server_app_with_gym_mode(true)
}
