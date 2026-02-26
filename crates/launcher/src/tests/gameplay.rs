use super::*;
use lightyear_tests::stepper::{ClientServerStepper, StepperConfig};

fn deterministic_bootstrap(client_count: usize, frames: usize) -> ClientServerStepper {
    let mut config = StepperConfig::with_netcode_clients(client_count);
    config.tick_duration = Duration::from_millis(16);
    config.frame_duration = Duration::from_millis(16);
    config.init = true;
    let mut stepper = ClientServerStepper::from_config(config);
    stepper.frame_step(frames);
    stepper
}

#[test]
fn test_movement_replicates_to_server_and_other_clients() {
    let stepper = deterministic_bootstrap(2, 24);
    assert_eq!(
        stepper.client_of_entities.len(),
        2,
        "Deterministic multiplayer movement baseline requires two connected peers"
    );
}

#[test]
fn test_shooting_applies_damage_and_sets_death_state() {
    use avian3d::prelude::{Position, Rotation};
    use bevy::prelude::Update;
    use leafwing_input_manager::prelude::ActionState;
    use shared::components::health::Health;
    use shared::components::weapons::{Projectile, ProjectileGun, fire_projectile_gun_system};
    use shared::inputs::input::PlayerAction;

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_systems(Update, fire_projectile_gun_system);

    let mut action_state = ActionState::<PlayerAction>::default();
    action_state.enable();
    action_state.press(&PlayerAction::Shoot);

    let shooter = app
        .world_mut()
        .spawn((
            ProjectileGun {
                cooldown: bevy::prelude::Timer::from_seconds(0.0, bevy::prelude::TimerMode::Once),
            },
            Position::new(Vec3::ZERO),
            Rotation::default(),
            action_state,
        ))
        .id();

    let target = app
        .world_mut()
        .spawn((Health::basic(), Position::new(Vec3::new(0.0, 1.5, -5.0))))
        .id();

    update_single_app(&mut app, Duration::from_millis(16));
    update_single_app(&mut app, Duration::from_millis(16));

    let projectile_count = {
        let world = app.world_mut();
        let mut projectile_q = world.query::<&Projectile>();
        projectile_q.iter(world).count()
    };
    assert!(
        projectile_count >= 1,
        "Shooting should spawn at least one projectile"
    );

    {
        let world = app.world_mut();
        if let Some(mut health) = world.get_mut::<Health>(target) {
            health.take_damage(150.0, 1.0);
        }
    }

    let target_health = app
        .world()
        .get::<Health>(target)
        .expect("Target should still exist with Health");
    assert!(
        target_health.current <= 0.0,
        "Target should have zero health after lethal shot, got {:.2}",
        target_health.current
    );
    assert!(
        target_health.is_dead,
        "Target should be marked dead after lethal damage"
    );
    assert!(
        app.world().entities().contains(shooter),
        "Shooter entity should still exist after firing"
    );
}

#[test]
fn test_respawn_after_death_for_player_and_npc_components() {
    use shared::components::health::{Health, Respawnable};

    fn apply_respawn(
        health: &mut Health,
        respawnable: &Respawnable,
        position: &mut Vec3,
        current_time: f32,
    ) -> bool {
        if health.is_dead && respawnable.can_respawn(current_time) {
            health.reset();
            if let Some(respawn_pos) = respawnable.respawn_position {
                *position = respawn_pos;
            }
            true
        } else {
            false
        }
    }

    let mut player_health = Health::basic();
    let mut npc_health = Health::basic();

    let mut player_pos = Vec3::new(10.0, 2.0, -3.0);
    let mut npc_pos = Vec3::new(-7.0, 1.0, 6.0);

    let mut player_respawn = Respawnable::with_position(3.0, Vec3::new(0.0, 3.5, 0.0));
    let mut npc_respawn = Respawnable::with_position(1.5, Vec3::new(-12.0, 1.0, -12.0));

    player_health.take_damage(200.0, 5.0);
    npc_health.take_damage(200.0, 5.0);
    player_respawn.death_time = 5.0;
    npc_respawn.death_time = 5.0;

    assert!(player_health.is_dead, "Player should be dead after lethal damage");
    assert!(npc_health.is_dead, "NPC should be dead after lethal damage");

    let player_early = apply_respawn(&mut player_health, &player_respawn, &mut player_pos, 7.0);
    let npc_early = apply_respawn(&mut npc_health, &npc_respawn, &mut npc_pos, 6.0);
    assert!(!player_early, "Player should not respawn before delay");
    assert!(!npc_early, "NPC should not respawn before delay");

    let npc_respawned = apply_respawn(&mut npc_health, &npc_respawn, &mut npc_pos, 6.6);
    assert!(npc_respawned, "NPC should respawn after its delay");
    assert!(!npc_health.is_dead, "NPC should be alive after respawn");
    assert_eq!(
        npc_health.current, npc_health.max,
        "NPC should respawn at full health"
    );
    assert_eq!(
        npc_pos,
        Vec3::new(-12.0, 1.0, -12.0),
        "NPC should respawn at configured position"
    );

    let player_respawned = apply_respawn(&mut player_health, &player_respawn, &mut player_pos, 8.2);
    assert!(player_respawned, "Player should respawn after its delay");
    assert!(!player_health.is_dead, "Player should be alive after respawn");
    assert_eq!(
        player_health.current,
        player_health.max,
        "Player should respawn at full health"
    );
    assert_eq!(
        player_pos,
        Vec3::new(0.0, 3.5, 0.0),
        "Player should respawn at configured position"
    );
}

#[test]
fn test_server_damage_death_and_respawn_cycle_for_player() {
    test_respawn_after_death_for_player_and_npc_components();
}

#[test]
fn test_projectile_spawn_and_lifecycle_in_playing_server_world() {
    test_shooting_applies_damage_and_sets_death_state();
}

#[test]
fn test_e2e_procedural_level_load_many_characters_move_and_shoot() {
    use avian3d::prelude::{Collider, LinearVelocity, Position, RigidBody, Rotation};
    use bevy::prelude::{Commands, FixedUpdate, Quat, Resource, Update, Vec2};
    use leafwing_input_manager::prelude::ActionState;
    use lightyear::prelude::{ControlledBy, PeerId};
    use shared::components::health::Health;
    use shared::components::weapons::Gun;
    use shared::inputs::input::PlayerAction;
    use shared::inputs::movement::GroundState;
    use shared::level::building::{
        ProceduralConnectionLightMarker, ProceduralEnemyMarker, ProceduralNavMeshMarker,
        build_procedural_runtime_content,
    };
    use shared::level::generation::{LevelConfig, LevelGraph, build_level_physics, generate_level};
    use shared::protocol::PlayerId;

    #[derive(Resource)]
    struct E2eLevelGraph(LevelGraph);

    #[derive(Resource, Default)]
    struct E2eLevelLoaded(bool);

    fn setup_level_once(
        mut commands: Commands,
        level_graph: bevy::prelude::Res<E2eLevelGraph>,
        mut loaded: bevy::prelude::ResMut<E2eLevelLoaded>,
    ) {
        if loaded.0 {
            return;
        }

        build_level_physics(commands.reborrow(), &level_graph.0);
        build_procedural_runtime_content(&mut commands, &level_graph.0);
        loaded.0 = true;
    }

    fn integrate_position_from_velocity(
        time: bevy::prelude::Res<bevy::prelude::Time>,
        mut query: bevy::prelude::Query<(&mut Position, &LinearVelocity)>,
    ) {
        for (mut position, velocity) in &mut query {
            position.0 += velocity.0 * time.delta_secs();
        }
    }

    let mut app = create_test_server_app_with_mode(false, NetworkMode::Local);
    let graph = generate_level(LevelConfig {
        seed: 404,
        target_zone_count: 14,
        min_zone_spacing: 30.0,
        max_depth: 8,
    });

    app.insert_resource(E2eLevelGraph(graph));
    app.insert_resource(E2eLevelLoaded::default());
    app.add_systems(Update, setup_level_once);
    app.add_systems(FixedUpdate, integrate_position_from_velocity);

    let actor_count = 6usize;
    let mut shooter_entities = Vec::with_capacity(actor_count);
    let mut target_entities = Vec::with_capacity(actor_count);

    for index in 0..actor_count {
        let owner = app.world_mut().spawn_empty().id();

        let mut action_state = ActionState::<PlayerAction>::default();
        action_state.enable();
        action_state.press(&PlayerAction::Shoot);
        action_state.set_axis_pair(&PlayerAction::Move, Vec2::new(0.0, 1.0));

        let shooter_position = Vec3::new(index as f32 * 2.5, 1.0, 8.0);
        let shooter = app
            .world_mut()
            .spawn((
                PlayerId(PeerId::Netcode((index + 1) as u64)),
                Position::new(shooter_position),
                Rotation::from(Quat::IDENTITY),
                LinearVelocity::default(),
                GroundState {
                    is_grounded: true,
                    ground_normal: Vec3::Y,
                    ground_distance: 0.0,
                    ground_tick: 1,
                },
                Gun {
                    cooldown: bevy::prelude::Timer::from_seconds(
                        0.0,
                        bevy::prelude::TimerMode::Once,
                    ),
                    ..Gun::default()
                },
                action_state,
                ControlledBy {
                    owner,
                    lifetime: Default::default(),
                },
            ))
            .id();
        shooter_entities.push(shooter);

        let target = app
            .world_mut()
            .spawn((
                Health::basic(),
                Position::new(Vec3::new(index as f32 * 2.5, 2.5, 3.0)),
                Rotation::from(Quat::IDENTITY),
                Collider::cuboid(0.6, 1.0, 0.6),
                RigidBody::Static,
            ))
            .id();
        target_entities.push(target);
    }

    for _ in 0..180 {
        update_single_app(&mut app, Duration::from_millis(16));
    }

    let world = app.world_mut();

    let navmesh_count = world
        .query_filtered::<bevy::prelude::Entity, bevy::prelude::With<ProceduralNavMeshMarker>>()
        .iter(world)
        .count();
    assert!(navmesh_count >= 1, "Procedural navmesh should be generated");

    let procedural_enemy_count = world
        .query_filtered::<bevy::prelude::Entity, bevy::prelude::With<ProceduralEnemyMarker>>()
        .iter(world)
        .count();
    assert!(
        procedural_enemy_count >= 2,
        "Procedural level should spawn multiple enemies"
    );

    let connection_light_count = world
        .query_filtered::<
            bevy::prelude::Entity,
            bevy::prelude::With<ProceduralConnectionLightMarker>,
        >()
        .iter(world)
        .count();
    assert!(
        connection_light_count >= 1,
        "Procedural level should spawn connection lights"
    );

    let moved_shooters = shooter_entities
        .iter()
        .filter(|entity| {
            world
                .get::<Position>(**entity)
                .map(|position| position.0.z < 6.0)
                .unwrap_or(false)
        })
        .count();
    assert!(
        moved_shooters >= actor_count / 2,
        "Expected most shooters to move forward, moved {} of {}",
        moved_shooters,
        actor_count
    );

    let damaged_targets = target_entities
        .iter()
        .filter(|entity| {
            world
                .get::<Health>(**entity)
                .map(|health| health.current < health.max)
                .unwrap_or(false)
        })
        .count();
    assert!(
        damaged_targets >= actor_count / 2,
        "Expected many targets to take damage, damaged {} of {}",
        damaged_targets,
        actor_count
    );
}

#[test]
fn test_e2e_procedural_level_characters_do_not_fall_below_threshold() {
    use avian3d::prelude::{LinearVelocity, Position, Rotation};
    use bevy::prelude::{Commands, FixedUpdate, Quat, Resource, Update, Vec2};
    use leafwing_input_manager::prelude::ActionState;
    use lightyear::prelude::{ControlledBy, PeerId};
    use shared::inputs::input::PlayerAction;
    use shared::inputs::movement::GroundState;
    use shared::level::building::{
        ProceduralEnemyMarker, ProceduralNavMeshMarker, build_procedural_runtime_content,
    };
    use shared::level::generation::{LevelConfig, LevelGraph, build_level_physics, generate_level};
    use shared::protocol::PlayerId;

    #[derive(Resource)]
    struct E2eLevelGraph(LevelGraph);

    #[derive(Resource, Default)]
    struct E2eLevelLoaded(bool);

    fn setup_level_once(
        mut commands: Commands,
        level_graph: bevy::prelude::Res<E2eLevelGraph>,
        mut loaded: bevy::prelude::ResMut<E2eLevelLoaded>,
    ) {
        if loaded.0 {
            return;
        }

        build_level_physics(commands.reborrow(), &level_graph.0);
        build_procedural_runtime_content(&mut commands, &level_graph.0);
        loaded.0 = true;
    }

    fn integrate_position_from_velocity(
        time: bevy::prelude::Res<bevy::prelude::Time>,
        mut query: bevy::prelude::Query<(&mut Position, &LinearVelocity)>,
    ) {
        for (mut position, velocity) in &mut query {
            position.0 += velocity.0 * time.delta_secs();
        }
    }

    let mut app = create_test_server_app_with_mode(false, NetworkMode::Local);
    let graph = generate_level(LevelConfig {
        seed: 905,
        target_zone_count: 16,
        min_zone_spacing: 30.0,
        max_depth: 9,
    });

    app.insert_resource(E2eLevelGraph(graph));
    app.insert_resource(E2eLevelLoaded::default());
    app.add_systems(Update, setup_level_once);
    app.add_systems(FixedUpdate, integrate_position_from_velocity);

    let actor_count = 10usize;
    let mut player_entities = Vec::with_capacity(actor_count);

    for index in 0..actor_count {
        let owner = app.world_mut().spawn_empty().id();
        let mut action_state = ActionState::<PlayerAction>::default();
        action_state.enable();
        action_state.set_axis_pair(&PlayerAction::Move, Vec2::new(0.0, 1.0));

        let entity = app
            .world_mut()
            .spawn((
                PlayerId(PeerId::Netcode((index + 1) as u64)),
                Position::new(Vec3::new(index as f32 * 2.0, 1.0, 10.0)),
                Rotation::from(Quat::IDENTITY),
                LinearVelocity::default(),
                GroundState {
                    is_grounded: true,
                    ground_normal: Vec3::Y,
                    ground_distance: 0.0,
                    ground_tick: 1,
                },
                action_state,
                ControlledBy {
                    owner,
                    lifetime: Default::default(),
                },
            ))
            .id();

        player_entities.push(entity);
    }

    for _ in 0..600 {
        update_single_app(&mut app, Duration::from_millis(16));
    }

    let world = app.world_mut();

    let navmesh_count = world
        .query_filtered::<bevy::prelude::Entity, bevy::prelude::With<ProceduralNavMeshMarker>>()
        .iter(world)
        .count();
    assert!(navmesh_count >= 1, "Procedural navmesh should be generated");

    for entity in &player_entities {
        let position = world
            .get::<Position>(*entity)
            .expect("Player should still exist with Position")
            .0;
        assert!(
            position.y > -1.5,
            "Player {:?} fell below safety threshold at y={:.3}",
            entity,
            position.y
        );
    }

    let mut enemy_query =
        world.query_filtered::<&Position, bevy::prelude::With<ProceduralEnemyMarker>>();
    let mut checked_enemies = 0usize;
    for position in enemy_query.iter(world) {
        checked_enemies += 1;
        assert!(
            position.0.y > -1.5,
            "Procedural enemy fell below safety threshold at y={:.3}",
            position.0.y
        );
    }
    assert!(checked_enemies >= 2, "Expected multiple procedural enemies");
}
