use crate::input::PlayerAction;
use avian3d::prelude::LinearVelocity;
use bevy::prelude::*;
use leafwing_input_manager::prelude::ActionState;
use serde::{Deserialize, Serialize};

pub struct StaminaPlugin;

impl Plugin for StaminaPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (
                update_stamina_consumption,
                update_stamina_regeneration,
                apply_stamina_effects,
            )
                .chain(),
        );
    }
}

/// Stamina component that tracks energy for actions
#[derive(Component, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Stamina {
    pub current: f32,
    pub max: f32,
    pub regeneration_rate: f32,     // Stamina per second when resting
    pub regeneration_delay: f32,    // Seconds before regen starts after use
    pub regeneration_timer: Timer,  // Timer for regeneration delay
    pub sprint_drain_rate: f32,     // Stamina per second when sprinting
    pub jump_cost: f32,             // Stamina cost per jump
    pub shoot_cost: f32,            // Stamina cost per shot
    pub low_stamina_threshold: f32, // Below this, player gets penalties
}

impl Default for Stamina {
    fn default() -> Self {
        Self {
            current: 100.0,
            max: 100.0,
            regeneration_rate: 20.0, // 20 stamina/sec when resting
            regeneration_delay: 2.0, // 2 seconds delay after use
            regeneration_timer: Timer::from_seconds(2.0, TimerMode::Once),
            sprint_drain_rate: 25.0,     // 25 stamina/sec when sprinting
            jump_cost: 15.0,             // 15 stamina per jump
            shoot_cost: 5.0,             // 5 stamina per shot
            low_stamina_threshold: 25.0, // Below 25%, player gets penalties
        }
    }
}

impl Stamina {
    /// Create stamina with custom values
    pub fn with_config(
        max: f32,
        regen_rate: f32,
        regen_delay: f32,
        sprint_drain: f32,
        jump_cost: f32,
        shoot_cost: f32,
    ) -> Self {
        Self {
            current: max,
            max,
            regeneration_rate: regen_rate,
            regeneration_delay: regen_delay,
            regeneration_timer: Timer::from_seconds(regen_delay, TimerMode::Once),
            sprint_drain_rate: sprint_drain,
            jump_cost,
            shoot_cost,
            low_stamina_threshold: max * 0.25,
        }
    }

    /// Check if player has enough stamina for an action
    pub fn can_afford(&self, cost: f32) -> bool {
        self.current >= cost
    }

    /// Consume stamina for an action
    pub fn consume(&mut self, amount: f32) {
        self.current = (self.current - amount).max(0.0);
        // Reset regeneration timer when stamina is used
        self.regeneration_timer.reset();
    }

    /// Get stamina as percentage
    pub fn percentage(&self) -> f32 {
        if self.max > 0.0 {
            self.current / self.max
        } else {
            0.0
        }
    }

    /// Check if stamina is low (affects performance)
    pub fn is_low(&self) -> bool {
        self.current < self.low_stamina_threshold
    }

    /// Check if stamina is critically low (severe penalties)
    pub fn is_critical(&self) -> bool {
        self.current < self.low_stamina_threshold * 0.5
    }

    /// Get movement speed multiplier based on stamina
    pub fn get_movement_multiplier(&self) -> f32 {
        if self.is_critical() {
            0.5 // 50% speed when critically low
        } else if self.is_low() {
            0.75 // 75% speed when low
        } else {
            1.0 // Normal speed
        }
    }

    /// Get sprint speed multiplier (can sprint if stamina allows)
    pub fn get_sprint_multiplier(&self) -> f32 {
        if self.current > 10.0 {
            1.5 // 50% faster when sprinting (if stamina allows)
        } else {
            1.0 // Can't sprint when low stamina
        }
    }

    /// Check if player can sprint
    pub fn can_sprint(&self) -> bool {
        self.current > 10.0
    }

    /// Check if player can jump
    pub fn can_jump(&self) -> bool {
        self.can_afford(self.jump_cost)
    }

    /// Check if player can shoot effectively
    pub fn can_shoot(&self) -> bool {
        self.can_afford(self.shoot_cost)
    }

    /// Get accuracy penalty based on stamina (when low stamina, accuracy suffers)
    pub fn get_accuracy_multiplier(&self) -> f32 {
        if self.is_critical() {
            0.6 // 40% accuracy penalty when critical
        } else if self.is_low() {
            0.8 // 20% accuracy penalty when low
        } else {
            1.0 // Normal accuracy
        }
    }
}

/// Component to track stamina-related effects and modifiers
#[derive(Component, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct StaminaEffects {
    pub is_sprinting: bool,
    pub movement_multiplier: f32,
    pub accuracy_multiplier: f32,
    pub last_jump_time: f32,
    pub last_shot_time: f32,
    pub exhaustion_level: ExhaustionLevel,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ExhaustionLevel {
    Fresh,     // 75-100% stamina
    Tired,     // 50-75% stamina
    Winded,    // 25-50% stamina
    Exhausted, // 0-25% stamina
}

impl Default for StaminaEffects {
    fn default() -> Self {
        Self {
            is_sprinting: false,
            movement_multiplier: 1.0,
            accuracy_multiplier: 1.0,
            last_jump_time: 0.0,
            last_shot_time: 0.0,
            exhaustion_level: ExhaustionLevel::Fresh,
        }
    }
}

impl StaminaEffects {
    pub fn update_from_stamina(&mut self, stamina: &Stamina) {
        self.movement_multiplier = stamina.get_movement_multiplier();
        self.accuracy_multiplier = stamina.get_accuracy_multiplier();

        // Update exhaustion level
        let percentage = stamina.percentage();
        self.exhaustion_level = if percentage >= 0.75 {
            ExhaustionLevel::Fresh
        } else if percentage >= 0.5 {
            ExhaustionLevel::Tired
        } else if percentage >= 0.25 {
            ExhaustionLevel::Winded
        } else {
            ExhaustionLevel::Exhausted
        };
    }
}

/// Enhanced sprint action for stamina system
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Reflect, Serialize, Deserialize)]
pub enum StaminaAction {
    Sprint,
}

/// System to handle stamina consumption based on player actions
fn update_stamina_consumption(
    mut stamina_query: Query<(
        &mut Stamina,
        &mut StaminaEffects,
        &ActionState<PlayerAction>,
    )>,
    time: Res<Time>,
) {
    for (mut stamina, mut effects, action_state) in stamina_query.iter_mut() {
        let dt = time.delta().as_secs_f32();

        // Check for sprinting (holding movement with high intensity)
        let movement = action_state.axis_pair(&PlayerAction::Move);
        let is_moving = movement.length() > 0.5;
        let wants_to_sprint = is_moving && movement.length() > 0.8; // High movement intensity = sprint intent

        // Update sprinting state
        effects.is_sprinting = wants_to_sprint && stamina.can_sprint();

        // Consume stamina for sprinting
        if effects.is_sprinting {
            let drain_rate = stamina.sprint_drain_rate;
            stamina.consume(drain_rate * dt);
        }

        // Handle jumping
        if action_state.just_pressed(&PlayerAction::Jump) && stamina.can_jump() {
            let jump_cost = stamina.jump_cost;
            stamina.consume(jump_cost);
            effects.last_jump_time = time.elapsed().as_secs_f32();
        }

        // Handle shooting
        if action_state.just_pressed(&PlayerAction::Shoot) && stamina.can_shoot() {
            let shoot_cost = stamina.shoot_cost;
            stamina.consume(shoot_cost);
            effects.last_shot_time = time.elapsed().as_secs_f32();
        }

        // Update effects based on current stamina
        effects.update_from_stamina(&stamina);
    }
}

/// System to handle stamina regeneration
fn update_stamina_regeneration(
    mut stamina_query: Query<(&mut Stamina, &StaminaEffects)>,
    time: Res<Time>,
) {
    for (mut stamina, effects) in stamina_query.iter_mut() {
        stamina.regeneration_timer.tick(time.delta());

        // Only regenerate if not actively using stamina and delay has passed
        if !effects.is_sprinting && stamina.regeneration_timer.is_finished() {
            let regen_amount = stamina.regeneration_rate * time.delta().as_secs_f32();
            stamina.current = (stamina.current + regen_amount).min(stamina.max);
        }
    }
}

/// System to apply stamina effects to player movement and other systems
fn apply_stamina_effects(mut velocity_query: Query<(&mut LinearVelocity, &StaminaEffects)>) {
    for (mut velocity, effects) in velocity_query.iter_mut() {
        // Apply movement speed multiplier
        let horizontal_velocity = Vec3::new(velocity.0.x, 0.0, velocity.0.z);
        let vertical_velocity = velocity.0.y;

        // Calculate modified horizontal velocity
        let mut modified_horizontal = horizontal_velocity * effects.movement_multiplier;

        // Apply sprint multiplier if sprinting
        if effects.is_sprinting {
            let sprint_multiplier = match effects.exhaustion_level {
                ExhaustionLevel::Fresh => 1.5,
                ExhaustionLevel::Tired => 1.3,
                ExhaustionLevel::Winded => 1.1,
                ExhaustionLevel::Exhausted => 1.0, // Can't sprint when exhausted
            };
            modified_horizontal *= sprint_multiplier;
        }

        // Update velocity while preserving vertical component
        velocity.0 = Vec3::new(
            modified_horizontal.x,
            vertical_velocity,
            modified_horizontal.z,
        );
    }
}

/// Helper function to add stamina to an entity
pub fn add_stamina_to_player(commands: &mut Commands, entity: Entity) {
    commands
        .entity(entity)
        .insert((Stamina::default(), StaminaEffects::default()));
}

/// Helper function to add stamina with custom configuration
pub fn add_stamina_with_config(
    commands: &mut Commands,
    entity: Entity,
    max: f32,
    regen_rate: f32,
    regen_delay: f32,
    sprint_drain: f32,
    jump_cost: f32,
    shoot_cost: f32,
) {
    commands.entity(entity).insert((
        Stamina::with_config(
            max,
            regen_rate,
            regen_delay,
            sprint_drain,
            jump_cost,
            shoot_cost,
        ),
        StaminaEffects::default(),
    ));
}

/// Stamina events for other systems to react to
#[derive(Clone, Debug, Message)]
pub struct StaminaDepletedEvent {
    pub entity: Entity,
    pub stamina_type: String, // "low", "critical", "empty"
}

#[derive(Clone, Debug, Message)]
pub struct StaminaRecoveredEvent {
    pub entity: Entity,
    pub new_level: ExhaustionLevel,
}
