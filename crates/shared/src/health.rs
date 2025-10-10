use avian3d::physics_transform::Position;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

pub struct HealthPlugin;

impl Plugin for HealthPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<DamageEvent>()
            .add_message::<HealEvent>()
            .add_message::<DeathEvent>()
            .add_message::<RespawnEvent>()
            .add_systems(
                FixedUpdate,
                (
                    process_damage_events,
                    process_heal_events,
                    process_death_events,
                    health_regeneration_system,
                    check_death_conditions,
                    update_invulnerability_system,
                ),
            )
            .add_systems(Update, (update_health_ui, debug_health_system));
    }
}

/// Component representing an entity's health
#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct Health {
    pub current: f32,
    pub max: f32,
    pub regeneration_rate: f32,  // Health per second
    pub regeneration_delay: f32, // Delay after taking damage before regen starts
    pub last_damage_time: f32,
    pub is_dead: bool,
    pub can_regenerate: bool,
}

impl Health {
    pub fn new(max_health: f32) -> Self {
        Self {
            current: max_health,
            max: max_health,
            regeneration_rate: 5.0,  // 5 HP per second default
            regeneration_delay: 3.0, // 3 second delay default
            last_damage_time: 0.0,
            is_dead: false,
            can_regenerate: true,
        }
    }

    pub fn with_regeneration(max_health: f32, regen_rate: f32, regen_delay: f32) -> Self {
        Self {
            current: max_health,
            max: max_health,
            regeneration_rate: regen_rate,
            regeneration_delay: regen_delay,
            last_damage_time: 0.0,
            is_dead: false,
            can_regenerate: true,
        }
    }

    pub fn no_regeneration(max_health: f32) -> Self {
        Self {
            current: max_health,
            max: max_health,
            regeneration_rate: 0.0,
            regeneration_delay: 0.0,
            last_damage_time: 0.0,
            is_dead: false,
            can_regenerate: false,
        }
    }

    /// Get health as a percentage (0.0 to 1.0)
    pub fn percentage(&self) -> f32 {
        if self.max <= 0.0 {
            0.0
        } else {
            (self.current / self.max).clamp(0.0, 1.0)
        }
    }

    /// Check if health is critically low (below 25%)
    pub fn is_critical(&self) -> bool {
        self.percentage() < 0.25
    }

    /// Check if health is full
    pub fn is_full(&self) -> bool {
        self.current >= self.max
    }

    /// Apply damage and return actual damage dealt
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

    /// Apply healing and return actual amount healed
    pub fn heal(&mut self, amount: f32) -> f32 {
        if self.is_dead || amount <= 0.0 {
            return 0.0;
        }

        let old_health = self.current;
        self.current = (self.current + amount).min(self.max);
        old_health - self.current
    }

    /// Reset health to full (for respawning)
    pub fn reset(&mut self) {
        self.current = self.max;
        self.is_dead = false;
        self.last_damage_time = 0.0;
    }

    /// Check if regeneration should be active
    pub fn can_regenerate_now(&self, current_time: f32) -> bool {
        self.can_regenerate
            && !self.is_dead
            && !self.is_full()
            && (current_time - self.last_damage_time) >= self.regeneration_delay
    }
}

/// Component for entities that can respawn
#[derive(Component, Clone, Debug)]
pub struct Respawnable {
    pub respawn_time: f32,              // Time in seconds before respawn
    pub death_time: f32,                // When the entity died
    pub respawn_position: Option<Vec3>, // Where to respawn (None = spawn at death location)
}

impl Respawnable {
    pub fn new(respawn_time: f32) -> Self {
        Self {
            respawn_time,
            death_time: 0.0,
            respawn_position: None,
        }
    }

    pub fn with_position(respawn_time: f32, position: Vec3) -> Self {
        Self {
            respawn_time,
            death_time: 0.0,
            respawn_position: Some(position),
        }
    }

    pub fn can_respawn(&self, current_time: f32) -> bool {
        (current_time - self.death_time) >= self.respawn_time
    }
}

/// Event for dealing damage to an entity
#[derive(Message, Clone, Debug, Serialize, Deserialize)]
pub struct DamageEvent {
    pub target: Entity,
    pub amount: f32,
    pub source: Option<Entity>, // Who/what caused the damage
    pub damage_type: DamageType,
    pub ignore_invulnerability: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum DamageType {
    Physical,
    Fire,
    Poison,
    Explosion,
    Fall,
    Environment,
}

/// Event for healing an entity
#[derive(Message, Clone, Debug, Serialize, Deserialize)]
pub struct HealEvent {
    pub target: Entity,
    pub amount: f32,
    pub source: Option<Entity>,
}

/// Event triggered when an entity dies
#[derive(Message, Clone, Debug, Serialize, Deserialize)]
pub struct DeathEvent {
    pub entity: Entity,
    pub killer: Option<Entity>,
}

/// Event for respawning an entity
#[derive(Message, Clone, Debug, Serialize, Deserialize)]
pub struct RespawnEvent {
    pub entity: Entity,
    pub position: Vec3,
}

/// Component to mark entities as invulnerable (temporary immunity to damage)
#[derive(Component, Clone, Debug)]
pub struct Invulnerable {
    pub duration: f32,
    pub remaining_time: f32,
}

impl Invulnerable {
    pub fn new(duration: f32) -> Self {
        Self {
            duration,
            remaining_time: duration,
        }
    }

    pub fn is_active(&self) -> bool {
        self.remaining_time > 0.0
    }
}

/// Component for health UI indicators
#[derive(Component, Clone, Debug)]
pub struct HealthBar {
    pub target_entity: Entity,
    pub offset: Vec3,
    pub size: Vec2,
    pub show_when_full: bool,
}

/// System to process damage events
fn process_damage_events(
    mut damage_events: MessageReader<DamageEvent>,
    mut health_query: Query<&mut Health>,
    invulnerable_query: Query<&Invulnerable>,
    mut death_events: MessageWriter<DeathEvent>,
    time: Res<Time>,
) {
    let current_time = time.elapsed().as_secs_f32();

    for damage_event in damage_events.read() {
        // Check if target is invulnerable
        if !damage_event.ignore_invulnerability {
            if let Ok(invuln) = invulnerable_query.get(damage_event.target) {
                if invuln.is_active() {
                    continue; // Skip damage
                }
            }
        }

        if let Ok(mut health) = health_query.get_mut(damage_event.target) {
            let actual_damage = health.take_damage(damage_event.amount, current_time);

            if actual_damage > 0.0 {
                info!(
                    "Entity {:?} took {:.1} damage (Health: {:.1}/{:.1})",
                    damage_event.target, actual_damage, health.current, health.max
                );

                // Trigger death event if entity died
                if health.is_dead {
                    death_events.write(DeathEvent {
                        entity: damage_event.target,
                        killer: damage_event.source,
                    });
                }
            }
        }
    }
}

/// System to process heal events
fn process_heal_events(
    mut heal_events: MessageReader<HealEvent>,
    mut health_query: Query<&mut Health>,
) {
    for heal_event in heal_events.read() {
        if let Ok(mut health) = health_query.get_mut(heal_event.target) {
            let actual_healing = health.heal(heal_event.amount);

            if actual_healing > 0.0 {
                info!(
                    "Entity {:?} healed for {:.1} (Health: {:.1}/{:.1})",
                    heal_event.target, actual_healing, health.current, health.max
                );
            }
        }
    }
}

/// System to process death events
fn process_death_events(
    mut death_events: MessageReader<DeathEvent>,
    mut respawnable_query: Query<&mut Respawnable>,
    _commands: Commands,
    time: Res<Time>,
) {
    let current_time = time.elapsed().as_secs_f32();

    for death_event in death_events.read() {
        info!("Entity {:?} died", death_event.entity);

        // If entity is respawnable, start respawn timer
        if let Ok(mut respawnable) = respawnable_query.get_mut(death_event.entity) {
            respawnable.death_time = current_time;
            info!(
                "Entity {:?} will respawn in {:.1} seconds",
                death_event.entity, respawnable.respawn_time
            );
        } else {
            // Entity is not respawnable, consider removing it or marking it as dead
            // For now, we just log it
            debug!("Entity {:?} died permanently", death_event.entity);
        }
    }
}

/// System for health regeneration
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

/// System to check for entities that should respawn
fn check_death_conditions(
    respawnable_query: Query<(Entity, &Health, &Respawnable), With<Respawnable>>,
    mut respawn_events: MessageWriter<RespawnEvent>,
    position_query: Query<&Position>,
    time: Res<Time>,
) {
    let current_time = time.elapsed().as_secs_f32();

    for (entity, health, respawnable) in respawnable_query.iter() {
        if health.is_dead && respawnable.can_respawn(current_time) {
            let respawn_position = respawnable.respawn_position.unwrap_or_else(|| {
                // Default to current position or origin if no position component
                position_query
                    .get(entity)
                    .map(|pos| pos.0)
                    .unwrap_or(Vec3::ZERO)
            });

            respawn_events.write(RespawnEvent {
                entity,
                position: respawn_position,
            });
        }
    }
}

/// System to update invulnerability timers
fn update_invulnerability_system(
    mut invulnerable_query: Query<(Entity, &mut Invulnerable)>,
    mut commands: Commands,
    time: Res<Time>,
) {
    let delta_time = time.delta().as_secs_f32();

    for (entity, mut invuln) in invulnerable_query.iter_mut() {
        invuln.remaining_time -= delta_time;

        if invuln.remaining_time <= 0.0 {
            commands.entity(entity).remove::<Invulnerable>();
            debug!("Entity {:?} is no longer invulnerable", entity);
        }
    }
}

/// Placeholder system for health UI updates
fn update_health_ui(health_bar_query: Query<(&HealthBar, Entity)>, health_query: Query<&Health>) {
    // This would update health bar UI elements
    // For now, it's a placeholder for future UI implementation
    for (health_bar, _ui_entity) in health_bar_query.iter() {
        if let Ok(health) = health_query.get(health_bar.target_entity) {
            // Update UI based on health percentage
            let _health_percentage = health.percentage();
            // TODO: Update actual UI elements when UI system is implemented
        }
    }
}

/// Debug system to log health status
fn debug_health_system(health_query: Query<(Entity, &Health, Option<&Name>)>) {
    for (entity, health, name) in health_query.iter() {
        if health.is_critical() && !health.is_dead {
            let entity_name = name.map(|n| n.as_str()).unwrap_or("Unknown");
            debug!(
                "CRITICAL: {} ({:?}) health: {:.1}/{:.1} ({:.1}%)",
                entity_name,
                entity,
                health.current,
                health.max,
                health.percentage() * 100.0
            );
        }
    }
}

/// Helper function to add health to an entity
pub fn add_health(commands: &mut Commands, entity: Entity, max_health: f32) {
    commands.entity(entity).insert(Health::new(max_health));
}

/// Helper function to add health with custom regeneration
pub fn add_health_with_regen(
    commands: &mut Commands,
    entity: Entity,
    max_health: f32,
    regen_rate: f32,
    regen_delay: f32,
) {
    commands.entity(entity).insert(Health::with_regeneration(
        max_health,
        regen_rate,
        regen_delay,
    ));
}

/// Helper function to make an entity respawnable
pub fn make_respawnable(commands: &mut Commands, entity: Entity, respawn_time: f32) {
    commands
        .entity(entity)
        .insert(Respawnable::new(respawn_time));
}

/// Helper function to apply damage to an entity
pub fn apply_damage(
    damage_events: &mut MessageWriter<DamageEvent>,
    target: Entity,
    amount: f32,
    source: Option<Entity>,
    damage_type: DamageType,
) {
    damage_events.write(DamageEvent {
        target,
        amount,
        source,
        damage_type,
        ignore_invulnerability: false,
    });
}

/// Helper function to heal an entity
pub fn apply_healing(
    heal_events: &mut MessageWriter<HealEvent>,
    target: Entity,
    amount: f32,
    source: Option<Entity>,
) {
    heal_events.write(HealEvent {
        target,
        amount,
        source,
    });
}
