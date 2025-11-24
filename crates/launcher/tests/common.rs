#![allow(dead_code)] // Allow unused functions in test utilities

use avian3d::collision::CollisionDiagnostics;
use avian3d::dynamics::solver::SolverDiagnostics;
use avian3d::prelude::Position;
use avian3d::spatial_query::SpatialQueryDiagnostics;
use bevy::prelude::*;
use client::lobby::AutoStart;
use launcher::{AutoHost, AutoJoin};
use leafwing_input_manager::prelude::ActionState;
use lightyear::prelude::{MessageSender, MetadataChannel};
use server::{ServerGameState, create_server_app};
use shared::{
    input::PlayerAction,
    protocol::{CharacterMarker, HostStartGameEvent, PlayerId},
};
use std::{thread, time::Duration};

// Headless version of ClientInputPlugin that doesn't depend on window events
struct HeadlessClientInputPlugin;

impl Plugin for HeadlessClientInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, headless_client_player_movement);
        // Skip window-dependent systems like handle_focus_change and toggle_cursor_grab
        app.add_observer(headless_grab_cursor);
    }
}

// Headless version of ClientEntitiesPlugin that doesn't depend on rendering assets
struct HeadlessClientEntitiesPlugin;

impl Plugin for HeadlessClientEntitiesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, headless_handle_local_player_setup);
        // Skip visual setup systems that depend on Mesh/Materials
    }
}

// Headless version that only sets up gameplay components, not visual ones
fn headless_handle_local_player_setup(
    mut commands: Commands,
    player_query: Query<
        (Entity, &shared::protocol::PlayerId),
        (
            With<lightyear::prelude::Predicted>,
            With<lightyear::prelude::Controlled>,
            With<shared::protocol::PlayerId>,
            Without<shared::input::FpsController>, // Use FpsController as a marker that it's already set up
        ),
    >,
    local_player_id: Res<client::LocalPlayerId>,
) {
    for (entity, player_id) in player_query.iter() {
        if player_id.0.to_bits() == local_player_id.0 {
            let input_map = client::input::get_player_input_map();
            let mut action_state = ActionState::<shared::input::PlayerAction>::default();
            action_state.enable();
            commands.entity(entity).insert((
                input_map,
                action_state,
                shared::input::FpsController::default(),
                shared::entities::PlayerPhysicsBundle::default(),
            ));
        }
    }
}

// Copy of client_player_movement but available in headless context
fn headless_client_player_movement(
    time: Res<Time>,
    spatial_query: Res<avian3d::prelude::SpatialQueryPipeline>,
    mut player_query: Query<
        (
            Entity,
            &ActionState<shared::input::PlayerAction>,
            &mut shared::input::FpsController,
            &mut Transform,
            &mut avian3d::prelude::LinearVelocity,
            &avian3d::prelude::Collider,
        ),
        (
            With<shared::protocol::PlayerId>,
            With<lightyear::prelude::Predicted>,
            With<lightyear::prelude::Controlled>,
        ),
    >,
) {
    if let Ok((entity, action_state, mut controller, mut transform, mut velocity, collider)) =
        player_query.single_mut()
    {
        shared::input::shared_player_movement(
            time.clone(),
            spatial_query.clone(),
            entity,
            action_state,
            &mut controller,
            &mut transform,
            &mut velocity,
            collider,
        );
    }
}

// Headless version that doesn't deal with cursor/window
fn headless_grab_cursor(
    trigger: On<Add, lightyear::prelude::Controlled>,
    mut commands: Commands,
    mut action_query: Query<
        &mut ActionState<shared::input::PlayerAction>,
        (
            With<shared::protocol::PlayerId>,
            With<lightyear::prelude::Predicted>,
            With<lightyear::prelude::Controlled>,
        ),
    >,
) {
    let controlled_entity = trigger.entity;

    match action_query.get_mut(controlled_entity) {
        Ok(mut action_state) => {
            action_state.enable();
        }
        Err(_) => {
            let input_map = client::input::get_player_input_map();
            let mut action_state = ActionState::<shared::input::PlayerAction>::default();
            action_state.enable();
            commands
                .entity(controlled_entity)
                .insert((input_map, action_state));
        }
    }
}

pub fn create_test_server() -> App {
    let mut server_app = create_server_app(true);

    // Add missing Avian3D diagnostics resources for headless testing
    server_app.insert_resource(CollisionDiagnostics::default());
    server_app.insert_resource(SolverDiagnostics::default());
    server_app.insert_resource(SpatialQueryDiagnostics::default());
    server_app.insert_resource(SolverDiagnostics::default());

    server_app
}

pub fn create_test_client(
    client_id: u64,
    auto_start: bool,
    auto_host: bool,
    auto_join: bool,
) -> App {
    let mut client_app = create_headless_client_app(client_id);
    if auto_start {
        client_app.insert_resource(AutoStart(true));
    }
    if auto_host {
        client_app.insert_resource(AutoHost(true));
    }
    if auto_join {
        client_app.insert_resource(AutoJoin(true));
    }

    client_app
}

pub fn create_headless_client_app(client_id: u64) -> App {
    // TODO :

    // Modify for something cleaner and easier that re-use client app like :
    //     pub fn create_headless_client_app(client_id: u64) -> App {
    //     use bevy::window::WindowPlugin;
    //     use client::create_client_app;

    //     // Mirror create_client_app() but with WindowPlugin disabled for headless testing
    //     let mut client_app = create_client_app(client_id, "../../assets".to_string());
    //     client_app.add_plugins(
    //         DefaultPlugins
    //             .build()
    //             .disable::<WindowPlugin>() // Remove window creation for headless testing
    //             .disable::<bevy::log::LogPlugin>(), // Disable LogPlugin to avoid conflict with lightyear logging
    //     );
    //     client_app.insert_resource(avian3d::collision::broad_phase::CollisionDiagnostics::default());

    //     client_app
    // }
    // Implement the user's conceptual approach: create_client_app() + disable WindowPlugin
    // Since plugins can't be modified after adding, we recreate the same setup but headless

    let mut client_app = App::new();
    let client_id = if client_id == 0 { 1 } else { client_id };

    // Use MinimalPlugins for headless testing (following Lightyear examples pattern)
    // This avoids all rendering-related systems that cause WindowResized message errors
    client_app.add_plugins((
        bevy::MinimalPlugins,
        bevy::log::LogPlugin::default(),
        bevy::state::app::StatesPlugin,
        bevy::diagnostic::DiagnosticsPlugin,
        bevy::asset::AssetPlugin {
            file_path: "../../assets".to_string(),
            ..Default::default()
        },
        bevy::scene::ScenePlugin,
        bevy::mesh::MeshPlugin,
        bevy::animation::AnimationPlugin,
    ));

    // Add the same plugins as create_client_app() in the same order
    client_app.add_plugins(shared::SharedPlugin);
    client_app.add_plugins(lightyear::prelude::client::ClientPlugins {
        tick_duration: std::time::Duration::from_secs_f64(1.0 / shared::FIXED_TIMESTEP_HZ),
    });

    client_app.insert_resource(client::LocalPlayerId(client_id));
    client_app.add_plugins(client::network::ClientNetworkPlugin);

    // Add headless-compatible input plugin
    client_app.add_plugins(HeadlessClientInputPlugin);

    // Skip RenderPlugin and DebugPlugin for headless mode - they require rendering systems
    // client_app.add_plugins(client::debug::DebugPlugin);  // Skip - uses Gizmos

    client_app.add_plugins(HeadlessClientEntitiesPlugin);
    client_app.add_plugins(client::lobby::ClientLobbyPlugin);
    client_app.add_plugins(client::game::GameClientPlugin);

    client_app.init_state::<client::ClientGameState>();
    client_app.insert_state(client::ClientGameState::LocalMenu);

    // Add Avian3D diagnostics resources for testing
    client_app.insert_resource(CollisionDiagnostics::default());
    client_app.insert_resource(SolverDiagnostics::default());
    client_app.insert_resource(SpatialQueryDiagnostics::default());

    client_app
}

pub fn run_app_updates(app: &mut App, cycles: usize) {
    for _ in 0..cycles {
        app.update();
    }
}

pub fn run_apps_updates(apps: &mut [&mut App], cycles: usize) {
    for _ in 0..cycles {
        for app in apps.iter_mut() {
            app.update();
        }
    }
}

pub fn run_app_updates_with_delay(app: &mut App, cycles: usize, delay_ms: u64) {
    for _ in 0..cycles {
        app.update();
        if delay_ms > 0 {
            thread::sleep(Duration::from_millis(delay_ms));
        }
    }
}

pub fn has_component<T: Component>(world: &World, entity: Entity) -> bool {
    world.get::<T>(entity).is_some()
}

pub fn count_entities_with<T: Component>(world: &mut World) -> usize {
    let mut query = world.query::<&T>();
    query.iter(world).count()
}

pub fn get_entities_with<T: Component>(world: &mut World) -> Vec<Entity> {
    let mut query = world.query::<(Entity, &T)>();
    query.iter(world).map(|(entity, _)| entity).collect()
}

pub fn simulate_player_movement(world: &mut World, player_entity: Entity, movement: Vec2) {
    if let Some(mut action_state) = world.get_mut::<ActionState<PlayerAction>>(player_entity) {
        action_state.set_axis_pair(&PlayerAction::Move, movement);
    }
}

pub fn simulate_player_look(world: &mut World, player_entity: Entity, look_delta: Vec2) {
    if let Some(mut action_state) = world.get_mut::<ActionState<PlayerAction>>(player_entity) {
        action_state.set_axis_pair(&PlayerAction::Look, look_delta);
    }
}

pub fn simulate_player_shoot(world: &mut World, player_entity: Entity, shooting: bool) {
    if let Some(mut action_state) = world.get_mut::<ActionState<PlayerAction>>(player_entity) {
        if shooting {
            action_state.press(&PlayerAction::Shoot);
        } else {
            action_state.release(&PlayerAction::Shoot);
        }
    }
}

pub fn get_entity_position(world: &World, entity: Entity) -> Option<Vec3> {
    world.get::<Position>(entity).map(|pos| pos.0)
}

pub fn assert_entity_moved(
    world: &World,
    entity: Entity,
    initial_position: Vec3,
    min_distance: f32,
) {
    let current_position =
        get_entity_position(world, entity).expect("Entity should have position component");

    let distance_moved = (current_position - initial_position).length();
    assert!(
        distance_moved >= min_distance,
        "Entity should have moved at least {} units, but only moved {} units. Initial: {:?}, Current: {:?}",
        min_distance,
        distance_moved,
        initial_position,
        current_position
    );
}

pub fn assert_entity_stable(
    world: &World,
    entity: Entity,
    initial_position: Vec3,
    max_distance: f32,
) {
    let current_position =
        get_entity_position(world, entity).expect("Entity should have position component");

    let distance_moved = (current_position - initial_position).length();
    assert!(
        distance_moved <= max_distance,
        "Entity should not have moved more than {} units, but moved {} units",
        max_distance,
        distance_moved
    );
}

pub fn wait_for_condition<F>(mut condition: F, max_attempts: usize, delay_ms: u64) -> bool
where
    F: FnMut() -> bool,
{
    for _ in 0..max_attempts {
        if condition() {
            return true;
        }
        if delay_ms > 0 {
            thread::sleep(Duration::from_millis(delay_ms));
        }
    }
    false
}

pub fn get_spawned_npcs(world: &mut World) -> Vec<Entity> {
    let mut query = world.query_filtered::<Entity, (With<CharacterMarker>, Without<PlayerId>)>();
    query.iter(world).collect()
}

pub fn get_spawned_players(world: &mut World) -> Vec<Entity> {
    let mut query = world.query::<(Entity, &PlayerId)>();
    query.iter(world).map(|(entity, _)| entity).collect()
}

pub fn is_game_session_active(server_world: &mut World) -> bool {
    let playing = matches!(
        *server_world.resource::<State<ServerGameState>>().get(),
        ServerGameState::Playing
    );

    let has_players = !get_spawned_players(server_world).is_empty();

    playing && has_players
}

pub fn setup_two_player_game() -> (App, App, App) {
    let mut server_app = create_test_server();
    for _ in 0..20 {
        server_app.update();
    }

    let mut client1 = create_test_client(1, false, false, true);
    let mut client2 = create_test_client(2, false, false, true);

    for _ in 0..100 {
        server_app.update();
        client1.update();
        client2.update();
    }

    let mut sender_query = client1
        .world_mut()
        .query::<&mut MessageSender<HostStartGameEvent>>();
    if let Ok(mut sender) = sender_query.single_mut(client1.world_mut()) {
        let _ = sender.send::<MetadataChannel>(HostStartGameEvent);
    }

    for _ in 0..500 {
        server_app.update();
        client1.update();
        client2.update();
    }

    (server_app, client1, client2)
}
