#![allow(dead_code)]

use bevy::prelude::*;
use rand::Rng;

// Utility functions and systems for GTFO-like game mechanics

// Tension and atmosphere system
#[derive(Resource)]
pub struct TensionSystem {
    pub current_tension: f32,
    pub max_tension: f32,
    pub tension_decay_rate: f32,
    pub last_enemy_encounter: f32,
    pub ambient_threat_level: f32,
}

impl Default for TensionSystem {
    fn default() -> Self {
        Self {
            current_tension: 0.0,
            max_tension: 100.0,
            tension_decay_rate: 5.0,
            last_enemy_encounter: 0.0,
            ambient_threat_level: 10.0,
        }
    }
}

impl TensionSystem {
    pub fn add_tension(&mut self, amount: f32, current_time: f32) {
        self.current_tension = (self.current_tension + amount).min(self.max_tension);
        self.last_enemy_encounter = current_time;
    }

    pub fn update(&mut self, delta_time: f32, current_time: f32) {
        // Gradually reduce tension over time
        let time_since_encounter = current_time - self.last_enemy_encounter;
        let decay_factor = if time_since_encounter > 30.0 { 2.0 } else { 1.0 };
        
        self.current_tension = (self.current_tension - self.tension_decay_rate * delta_time * decay_factor)
            .max(self.ambient_threat_level);
    }

    pub fn get_tension_level(&self) -> TensionLevel {
        match self.current_tension {
            0.0..=25.0 => TensionLevel::Calm,
            25.1..=50.0 => TensionLevel::Uneasy,
            50.1..=75.0 => TensionLevel::Tense,
            _ => TensionLevel::Panic,
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum TensionLevel {
    Calm,
    Uneasy,
    Tense,
    Panic,
}

// Procedural event system for dynamic gameplay
#[derive(Resource)]
pub struct EventSystem {
    pub next_event_time: f32,
    pub event_frequency: f32,
    pub events_triggered: u32,
}

impl Default for EventSystem {
    fn default() -> Self {
        Self {
            next_event_time: 60.0, // First event in 60 seconds
            event_frequency: 120.0, // Events every 2 minutes on average
            events_triggered: 0,
        }
    }
}

#[derive(Clone, Copy)]
pub enum GameEvent {
    PowerOutage,
    SecurityAlarm,
    VentilationNoise,
    DistantExplosion,
    RadioStatic,
    EnemyReinforcements,
    EquipmentMalfunction,
    EnvironmentalHazard,
}

pub fn trigger_random_event(
    mut event_system: ResMut<EventSystem>,
    mut tension_system: ResMut<TensionSystem>,
    time: Res<Time>,
) {
    let current_time = time.elapsed_secs();
    
    if current_time >= event_system.next_event_time {
        let mut rng = rand::rng();
        let event = match rng.random_range(0..8) {
            0 => GameEvent::PowerOutage,
            1 => GameEvent::SecurityAlarm,
            2 => GameEvent::VentilationNoise,
            3 => GameEvent::DistantExplosion,
            4 => GameEvent::RadioStatic,
            5 => GameEvent::EnemyReinforcements,
            6 => GameEvent::EquipmentMalfunction,
            _ => GameEvent::EnvironmentalHazard,
        };

        execute_game_event(event, &mut tension_system, current_time);
        
        // Schedule next event
        let variation = rng.random_range(0.5..1.5);
        event_system.next_event_time = current_time + (event_system.event_frequency * variation);
        event_system.events_triggered += 1;
    }
}

pub fn execute_game_event(
    event: GameEvent,
    tension_system: &mut TensionSystem,
    current_time: f32,
) {
    match event {
        GameEvent::PowerOutage => {
            info!("Event: Power flickering detected");
            tension_system.add_tension(15.0, current_time);
        },
        GameEvent::SecurityAlarm => {
            info!("Event: Security alarm triggered");
            tension_system.add_tension(25.0, current_time);
        },
        GameEvent::VentilationNoise => {
            info!("Event: Strange noise from ventilation");
            tension_system.add_tension(10.0, current_time);
        },
        GameEvent::DistantExplosion => {
            info!("Event: Distant explosion heard");
            tension_system.add_tension(20.0, current_time);
        },
        GameEvent::RadioStatic => {
            info!("Event: Radio static interference");
            tension_system.add_tension(5.0, current_time);
        },
        GameEvent::EnemyReinforcements => {
            info!("Event: Enemy reinforcements incoming");
            tension_system.add_tension(30.0, current_time);
        },
        GameEvent::EquipmentMalfunction => {
            info!("Event: Equipment malfunction detected");
            tension_system.add_tension(12.0, current_time);
        },
        GameEvent::EnvironmentalHazard => {
            info!("Event: Environmental hazard detected");
            tension_system.add_tension(18.0, current_time);
        },
    }
}

// Utility for spawning particles/effects
pub fn spawn_muzzle_flash(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    position: Vec3,
    direction: Vec3,
) {
    let flash_material = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 1.0, 0.8),
        emissive: LinearRgba::rgb(2.0 * 5.0, 1.5 * 5.0, 0.5 * 5.0).into(),
        unlit: true,
        ..default()
    });

    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(0.2))),
        MeshMaterial3d(flash_material),
        Transform::from_translation(position + direction * 0.5),
        MuzzleFlash { lifetime: 0.1 },
        Name::new("Muzzle Flash"),
    ));
}

#[derive(Component)]
pub struct MuzzleFlash {
    pub lifetime: f32,
}

pub fn update_muzzle_flashes(
    time: Res<Time>,
    mut commands: Commands,
    mut flash_query: Query<(Entity, &mut MuzzleFlash)>,
) {
    for (entity, mut flash) in flash_query.iter_mut() {
        flash.lifetime -= time.delta_secs();
        if flash.lifetime <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

// Cooperative gameplay utilities
pub fn calculate_team_effectiveness(
    team_positions: &[Vec3],
    optimal_spacing: f32,
) -> f32 {
    if team_positions.len() < 2 {
        return 1.0;
    }

    let mut total_effectiveness = 0.0;
    let mut pair_count = 0;

    for i in 0..team_positions.len() {
        for j in (i + 1)..team_positions.len() {
            let distance = team_positions[i].distance(team_positions[j]);
            let effectiveness = if distance <= optimal_spacing {
                1.0 - (optimal_spacing - distance) / optimal_spacing * 0.5
            } else {
                1.0 - ((distance - optimal_spacing) / optimal_spacing).min(1.0) * 0.3
            };
            
            total_effectiveness += effectiveness;
            pair_count += 1;
        }
    }

    total_effectiveness / pair_count as f32
}

// Resource management utilities
#[derive(Resource)]
pub struct TeamResources {
    pub total_ammo: u32,
    pub medical_supplies: u32,
    pub tools: u32,
    pub shared_equipment: Vec<SharedEquipment>,
}

#[derive(Clone)]
pub struct SharedEquipment {
    pub equipment_type: String,
    pub uses_remaining: u32,
    pub max_uses: u32,
}
