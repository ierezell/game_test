use bevy::prelude::*;

// Player movement and interaction events (simplified to remove unused fields)
#[derive(Event, Debug, Clone)]
pub struct PlayerMovedEvent {
    pub entity: Entity,
}

#[derive(Event, Debug, Clone)]
pub struct PlayerJumpedEvent {
    pub entity: Entity,
}

#[derive(Event, Debug, Clone)]
pub struct FlashlightToggledEvent {
    pub entity: Entity,
    pub is_on: bool,
}

// Combat-related events (simplified to remove unused fields)
#[derive(Event, Debug, Clone)]
pub struct WeaponFireEvent {
    pub entity: Entity,
}

#[derive(Event, Debug, Clone)]
pub struct DamageDealtEvent {
    pub attacker: Entity,
    pub target: Entity,
    pub damage: f32,
}

#[derive(Event, Debug, Clone)]
pub struct EntityDeathEvent {
    pub entity: Entity,
    pub cause: DeathCause,
}

#[derive(Debug, Clone)]
pub enum DeathCause {
    Combat,
}

// Enemy AI events
#[derive(Event, Debug, Clone)]
pub struct EnemyStateChangedEvent {
    pub entity: Entity,
    pub old_state: EnemyState,
    pub new_state: EnemyState,
}

#[derive(Debug, Clone)]
pub enum EnemyState {
    Dormant,
    Patrolling,
    Investigating,
    Chasing,
    Attacking,
}

// Audio events (simplified to remove unused fields)
#[derive(Event, Debug, Clone)]
pub struct SoundTriggeredEvent {
    pub position: Vec3,
    pub sound_type: SoundType,
    pub intensity: f32,
}

#[derive(Debug, Clone)]
pub enum SoundType {
    Footstep,
    WeaponFire,
    Impact,
}

// Plugin to register events
pub struct GameEventsPlugin;

impl Plugin for GameEventsPlugin {
    fn build(&self, app: &mut App) {
        app
            // Player events
            .add_event::<PlayerMovedEvent>()
            .add_event::<PlayerJumpedEvent>()
            .add_event::<FlashlightToggledEvent>()
            // Combat events
            .add_event::<WeaponFireEvent>()
            .add_event::<DamageDealtEvent>()
            .add_event::<EntityDeathEvent>()
            // Enemy AI events
            .add_event::<EnemyStateChangedEvent>()
            // Audio events
            .add_event::<SoundTriggeredEvent>();
    }
}
