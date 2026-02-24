use bevy::prelude::{App, Entity, MinimalPlugins, Update};
use shared::components::health::{DamageEvent, Health, HealthPlugin};

fn spawn_target(app: &mut App) -> Entity {
    app.world_mut().spawn(Health::basic()).id()
}

#[test]
fn npc_health_accumulates_two_player_hits_to_fifty() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(HealthPlugin);
    app.add_systems(Update, || {});

    let target = spawn_target(&mut app);

    app.world_mut().write_message(DamageEvent {
        target,
        amount: 25.0,
        source: None,
    });
    app.world_mut().write_message(DamageEvent {
        target,
        amount: 25.0,
        source: None,
    });

    app.update();

    let health = app
        .world()
        .get::<Health>(target)
        .expect("Target must still have health after non-lethal damage");

    assert!(
        (health.current - 50.0).abs() < 0.001,
        "Expected health to be 50 after two 25-damage hits, got {}",
        health.current
    );
    assert!(!health.is_dead, "Target should still be alive at 50 health");
}
