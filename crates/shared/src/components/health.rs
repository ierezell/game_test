use bevy::prelude::{
    App, Component, Entity, Message, MessageReader, Plugin, Query, Res, Time, Update, Vec3, info,
};
use serde::{Deserialize, Serialize};

pub struct HealthPlugin;

impl Plugin for HealthPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<DamageEvent>()
            .add_systems(Update, (process_damage_events, health_regeneration_system));
    }
}

#[derive(Component, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Health {
    pub current: f32,
    pub max: f32,
    pub regeneration_rate: f32,
    pub regeneration_delay: f32,
    pub last_damage_time: f32,
    pub is_dead: bool,
    pub can_regenerate: bool,
}

impl Health {
    pub fn basic() -> Self {
        Self {
            current: 100.0,
            max: 100.0,
            regeneration_rate: 5.0,
            regeneration_delay: 3.0,
            last_damage_time: 0.0,
            is_dead: false,
            can_regenerate: true,
        }
    }
}

impl Health {
    pub fn percentage(&self) -> f32 {
        if self.max <= 0.0 {
            0.0
        } else {
            (self.current / self.max).clamp(0.0, 1.0)
        }
    }

    pub fn take_damage(&mut self, amount: f32, current_time: f32) -> f32 {
        if self.is_dead || amount <= 0.0 {
            return 0.0;
        }

        let old_health = self.current;
        self.current = (self.current - amount).max(0.0);
        self.last_damage_time = current_time;

        if self.current <= 0.0 {
            self.is_dead = true;
        }

        old_health - self.current
    }

    pub fn heal(&mut self, amount: f32) -> f32 {
        if self.is_dead || amount <= 0.0 {
            return 0.0;
        }

        let old_health = self.current;
        self.current = (self.current + amount).min(self.max);
        self.current - old_health
    }

    pub fn reset(&mut self) {
        self.current = self.max;
        self.is_dead = false;
        self.last_damage_time = 0.0;
    }

    pub fn can_regenerate_now(&self, current_time: f32) -> bool {
        self.can_regenerate
            && !self.is_dead
            && (self.current < self.max)
            && (current_time - self.last_damage_time) >= self.regeneration_delay
    }
}

#[derive(Component, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Respawnable {
    pub respawn_delay: f32,
    pub death_time: f32,
    pub respawn_position: Option<Vec3>, // Where to respawn (None = spawn at death location)
}

impl Respawnable {
    pub fn new(respawn_delay: f32) -> Self {
        Self {
            respawn_delay,
            death_time: 0.0,
            respawn_position: None,
        }
    }

    pub fn with_position(respawn_delay: f32, position: Vec3) -> Self {
        Self {
            respawn_delay,
            death_time: 0.0,
            respawn_position: Some(position),
        }
    }

    pub fn can_respawn(&self, current_time: f32) -> bool {
        (current_time - self.death_time) >= self.respawn_delay
    }
}

#[derive(Message, Clone, Debug, Serialize, Deserialize)]
pub struct DamageEvent {
    pub target: Entity,
    pub amount: f32,
    pub source: Option<Entity>, // Who/what caused the damage
}

fn process_damage_events(
    mut damage_events: MessageReader<DamageEvent>,
    mut health_query: Query<&mut Health>,

    time: Res<Time>,
) {
    let current_time = time.elapsed().as_secs_f32();

    for damage_event in damage_events.read() {
        if let Ok(mut health) = health_query.get_mut(damage_event.target) {
            let actual_damage = health.take_damage(damage_event.amount, current_time);

            if actual_damage > 0.0 {
                info!(
                    "Entity {:?} took {:.1} damage (Health: {:.1}/{:.1})",
                    damage_event.target, actual_damage, health.current, health.max
                );
            }
        }
    }
}

fn health_regeneration_system(mut health_query: Query<&mut Health>, time: Res<Time>) {
    let current_time = time.elapsed().as_secs_f32();
    let delta_time = time.delta().as_secs_f32();

    for mut health in health_query.iter_mut() {
        if health.can_regenerate_now(current_time) {
            let regen_amount = health.regeneration_rate * delta_time;
            health.heal(regen_amount);
        }
    }
}
