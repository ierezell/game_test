use avian3d::prelude::*;
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;
use rand::Rng;
use crate::game_state::GameState;

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                weapon_fire,
                projectile_movement,
                damage_system,
                cleanup_projectiles,
                remove_dead_entities,
            ).run_if(in_state(GameState::InGame)),
        );
    }
}

#[derive(Component)]
pub struct Health {
    pub current: f32,
    pub maximum: f32,
    pub regeneration_rate: f32,
    pub last_damage_time: f32,
}

impl Health {
    pub fn new(max_health: f32) -> Self {
        Self {
            current: max_health,
            maximum: max_health,
            regeneration_rate: 0.0,
            last_damage_time: 0.0,
        }
    }

    pub fn take_damage(&mut self, damage: f32, time: f32) {
        self.current = (self.current - damage).max(0.0);
        self.last_damage_time = time;
    }

    pub fn is_dead(&self) -> bool {
        self.current <= 0.0
    }

    pub fn heal(&mut self, amount: f32) {
        self.current = (self.current + amount).min(self.maximum);
    }
}

#[derive(Component)]
pub struct Weapon {
    pub weapon_type: WeaponType,
    pub damage: f32,
    pub range: f32,
    pub rate_of_fire: f32, // shots per second
    pub last_shot_time: f32,
    pub ammo_current: u32,
    pub ammo_max: u32,
    pub reload_time: f32,
    pub is_reloading: bool,
    pub reload_start_time: f32,
    pub accuracy: f32, // 0.0 to 1.0
}

#[derive(Clone, Copy, PartialEq)]
pub enum WeaponType {
    AssaultRifle,
    Shotgun,
    Pistol,
    SniperRifle,
    SMG,
}

impl Weapon {
    pub fn new_assault_rifle() -> Self {
        Self {
            weapon_type: WeaponType::AssaultRifle,
            damage: 30.0,
            range: 50.0,
            rate_of_fire: 8.0,
            last_shot_time: 0.0,
            ammo_current: 30,
            ammo_max: 30,
            reload_time: 2.5,
            is_reloading: false,
            reload_start_time: 0.0,
            accuracy: 0.85,
        }
    }

    pub fn new_shotgun() -> Self {
        Self {
            weapon_type: WeaponType::Shotgun,
            damage: 80.0,
            range: 15.0,
            rate_of_fire: 1.2,
            last_shot_time: 0.0,
            ammo_current: 8,
            ammo_max: 8,
            reload_time: 3.0,
            is_reloading: false,
            reload_start_time: 0.0,
            accuracy: 0.7,
        }
    }

    pub fn can_fire(&self, current_time: f32) -> bool {
        !self.is_reloading
            && self.ammo_current > 0
            && current_time - self.last_shot_time >= 1.0 / self.rate_of_fire
    }

    pub fn start_reload(&mut self, current_time: f32) {
        if self.ammo_current < self.ammo_max && !self.is_reloading {
            self.is_reloading = true;
            self.reload_start_time = current_time;
        }
    }

    pub fn update_reload(&mut self, current_time: f32) {
        if self.is_reloading && current_time - self.reload_start_time >= self.reload_time {
            self.ammo_current = self.ammo_max;
            self.is_reloading = false;
        }
    }
}

#[derive(Component)]
pub struct Projectile {
    pub damage: f32,
    pub speed: f32,
    pub lifetime: f32,
    pub penetration: u32,
    pub owner: Entity,
}

fn weapon_fire(
    time: Res<Time>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut weapon_query: Query<(Entity, &mut Weapon, &GlobalTransform)>,
    input_query: Query<&ActionState<crate::player::PlayerAction>>,
) {
    let current_time = time.elapsed_secs();

    for (entity, mut weapon, transform) in weapon_query.iter_mut() {
        weapon.update_reload(current_time);

        // Check if this weapon belongs to a player who is firing
        if let Ok(action_state) = input_query.single() {
            if action_state.pressed(&crate::player::PlayerAction::PrimaryFire) {
                if weapon.can_fire(current_time) {
                    fire_weapon(
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        entity,
                        &mut weapon,
                        transform,
                        current_time,
                    );
                }
            }

            if action_state.just_pressed(&crate::player::PlayerAction::Reload) {
                weapon.start_reload(current_time);
            }
        }
    }
}

fn fire_weapon(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    owner: Entity,
    weapon: &mut Weapon,
    transform: &GlobalTransform,
    current_time: f32,
) {
    weapon.last_shot_time = current_time;
    weapon.ammo_current = weapon.ammo_current.saturating_sub(1);

    let forward = transform.forward();
    let position = transform.translation() + forward * 1.0; // Offset from muzzle

    // Add accuracy spread
    let mut rng = rand::rng();
    let spread = (1.0 - weapon.accuracy) * 0.1;
    let spread_x = rng.random_range(-spread..spread);
    let spread_y = rng.random_range(-spread..spread);
    let spread_z = rng.random_range(-spread..spread);

    let direction = (*forward + Vec3::new(spread_x, spread_y, spread_z)).normalize();

    // Create projectile
    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(0.05))),
        MeshMaterial3d(materials.add(Color::srgb(1.0, 1.0, 0.0))),
        Transform::from_translation(position),
        RigidBody::Dynamic,
        Collider::sphere(0.05),
        LinearVelocity(direction * 50.0),
        Projectile {
            damage: weapon.damage,
            speed: 50.0,
            lifetime: 5.0,
            penetration: 1,
            owner,
        },
        Name::new("Projectile"),
    ));

    info!("Fired weapon! Ammo remaining: {}", weapon.ammo_current);
}

fn projectile_movement(
    time: Res<Time>,
    mut projectile_query: Query<(Entity, &mut Projectile, &mut Transform)>,
    mut commands: Commands,
) {
    for (entity, mut projectile, _transform) in projectile_query.iter_mut() {
        projectile.lifetime -= time.delta_secs();

        if projectile.lifetime <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

fn damage_system(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionStarted>,
    projectile_query: Query<&Projectile>,
    mut health_query: Query<&mut Health>,
) {
    for CollisionStarted(entity1, entity2) in collision_events.read() {
        info!("Collision detected between {:?} and {:?}", entity1, entity2);
        
        // Debug: Check what components each entity has
        let projectile1 = projectile_query.get(*entity1).is_ok();
        let projectile2 = projectile_query.get(*entity2).is_ok();
        let health1 = health_query.get(*entity1).is_ok();
        let health2 = health_query.get(*entity2).is_ok();
        
        info!("Entity1 - Projectile: {}, Health: {}", projectile1, health1);
        info!("Entity2 - Projectile: {}, Health: {}", projectile2, health2);
        
        // Check if one entity is a projectile and the other has health
        if let Ok(projectile) = projectile_query.get(*entity1) {
            if let Ok(mut health) = health_query.get_mut(*entity2) {
                health.take_damage(projectile.damage, 0.0); // TODO: pass actual time
                commands.entity(*entity1).despawn(); // Remove projectile on hit
                info!(
                    "Hit target! Damage: {}, Health remaining: {}",
                    projectile.damage, health.current
                );
            } else {
                info!("Entity1 is projectile but Entity2 has no health component");
            }
        } else if let Ok(projectile) = projectile_query.get(*entity2) {
            if let Ok(mut health) = health_query.get_mut(*entity1) {
                health.take_damage(projectile.damage, 0.0);
                commands.entity(*entity2).despawn();
                info!(
                    "Hit target! Damage: {}, Health remaining: {}",
                    projectile.damage, health.current
                );
            } else {
                info!("Entity2 is projectile but Entity1 has no health component");
            }
        } else {
            info!("Neither entity is a projectile - collision between non-projectile entities");
        }
    }
}

fn cleanup_projectiles(
    _commands: Commands,
    _projectile_query: Query<Entity, With<Projectile>>,
    _time: Res<Time>,
) {
    // This system runs cleanup for projectiles that might have gotten stuck
    // In a real implementation, you'd want more sophisticated cleanup
}

fn remove_dead_entities(
    mut commands: Commands,
    health_query: Query<(Entity, &Health), Without<crate::player::Player>>,
) {
    for (entity, health) in health_query.iter() {
        if health.current <= 0.0 {
            info!("Entity died! Removing entity: {:?}", entity);
            commands.entity(entity).despawn();
        }
    }
}
