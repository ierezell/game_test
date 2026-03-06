use avian3d::prelude::{LinearVelocity, Position};
use bevy::{
    ecs::query::Without,
    prelude::{Commands, Entity, Query, Res, Vec3, With, info},
};
use shared::{
    components::health::{Health, Respawnable},
    protocol::{CharacterMarker, PlayerId},
};

#[derive(bevy::prelude::Component)]
pub struct PendingNpcRespawn;

pub fn mark_dead_npcs_for_respawn(
    mut commands: Commands,
    time: Res<bevy::prelude::Time>,
    mut npc_query: Query<
        (
            Entity,
            &Health,
            &mut Respawnable,
            &mut Position,
            &mut LinearVelocity,
        ),
        (
            With<CharacterMarker>,
            Without<PlayerId>,
            Without<PendingNpcRespawn>,
        ),
    >,
) {
    let now = time.elapsed().as_secs_f32();

    for (entity, health, mut respawnable, mut position, mut linear_velocity) in &mut npc_query {
        if !health.is_dead {
            continue;
        }

        respawnable.death_time = now;
        linear_velocity.0 = Vec3::ZERO;
        position.0.y = -1000.0;

        commands.entity(entity).insert(PendingNpcRespawn);
        info!("💀 NPC {:?} killed, scheduling respawn", entity);
    }
}

pub fn respawn_dead_npcs(
    mut commands: Commands,
    time: Res<bevy::prelude::Time>,
    mut npc_query: Query<
        (
            Entity,
            &mut Health,
            &Respawnable,
            &mut Position,
            &mut LinearVelocity,
        ),
        (
            With<CharacterMarker>,
            Without<PlayerId>,
            With<PendingNpcRespawn>,
        ),
    >,
) {
    let now = time.elapsed().as_secs_f32();

    for (entity, mut health, respawnable, mut position, mut linear_velocity) in &mut npc_query {
        if !respawnable.can_respawn(now) {
            continue;
        }

        health.reset();
        if let Some(respawn_position) = respawnable.respawn_position {
            position.0 = respawn_position;
        }
        linear_velocity.0 = Vec3::ZERO;
        commands.entity(entity).remove::<PendingNpcRespawn>();

        info!("✨ NPC {:?} respawned at {:?}", entity, position.0);
    }
}

#[cfg(test)]
mod tests {
    use super::{mark_dead_npcs_for_respawn, respawn_dead_npcs};
    use avian3d::prelude::{LinearVelocity, Position};
    use bevy::prelude::{App, MinimalPlugins, Update, Vec3};
    use shared::components::health::{DamageEvent, Health, HealthPlugin, Respawnable};
    use shared::protocol::CharacterMarker;

    fn advance(app: &mut App, dt: std::time::Duration) {
        app.insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(dt));
        app.update();
    }

    #[test]
    fn npc_damage_kill_and_respawn_cycle() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(HealthPlugin);
        app.add_systems(Update, (mark_dead_npcs_for_respawn, respawn_dead_npcs));

        let spawn_position = Vec3::new(-18.0, 1.0, -8.0);
        let npc = app
            .world_mut()
            .spawn((
                CharacterMarker,
                Position::new(spawn_position),
                LinearVelocity::default(),
                Health::basic(),
                Respawnable::with_position(0.25, spawn_position),
            ))
            .id();

        app.world_mut().write_message(DamageEvent {
            target: npc,
            amount: 500.0,
            source: None,
        });

        for _ in 0..4 {
            advance(&mut app, std::time::Duration::from_millis(16));
        }

        let health_after_damage = app
            .world()
            .get::<Health>(npc)
            .expect("NPC should still exist after taking damage");
        assert!(health_after_damage.is_dead, "NPC should be marked dead");

        let hidden_position = app
            .world()
            .get::<Position>(npc)
            .expect("NPC should still have a position when dead")
            .0;
        assert!(
            hidden_position.y < -100.0,
            "Dead NPC should be hidden below world, got {:?}",
            hidden_position
        );

        for _ in 0..8 {
            advance(&mut app, std::time::Duration::from_millis(16));
        }

        let still_dead = app
            .world()
            .get::<Health>(npc)
            .expect("NPC should still exist before respawn");
        assert!(still_dead.is_dead, "NPC should still be dead before delay");

        for _ in 0..12 {
            advance(&mut app, std::time::Duration::from_millis(16));
        }

        let health_after_respawn = app
            .world()
            .get::<Health>(npc)
            .expect("NPC should still exist after respawn");
        assert!(
            !health_after_respawn.is_dead,
            "NPC should be alive after respawn delay"
        );
        assert_eq!(
            health_after_respawn.current, health_after_respawn.max,
            "NPC should respawn with full health"
        );

        let respawned_position = app
            .world()
            .get::<Position>(npc)
            .expect("NPC should have position after respawn")
            .0;
        assert_eq!(
            respawned_position, spawn_position,
            "NPC should respawn at configured respawn position"
        );
    }
}
