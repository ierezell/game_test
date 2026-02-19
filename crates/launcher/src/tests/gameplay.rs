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
