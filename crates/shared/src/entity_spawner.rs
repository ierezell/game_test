use super::entity_implementations::*;
use super::entity_traits::*;

use avian3d::prelude::Position;
use bevy::prelude::{
    Assets, Commands, Entity, Mesh, Mesh3d, MeshMaterial3d, Name, ResMut, StandardMaterial, Vec3,
    debug,
};

/// Generic system for adding visuals to any entity that implements VisualProvider
pub fn add_entity_visuals<T: VisualProvider + GameEntity + 'static>(
    entity: &T,
    commands: &mut Commands,
    entity_id: Entity,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    let mesh = entity.get_mesh();
    let material = entity.get_material();

    commands.entity(entity_id).insert((
        Mesh3d(meshes.add(mesh)),
        MeshMaterial3d(materials.add(material)),
    ));

    debug!(
        "Added {} visuals to entity {:?}",
        entity.entity_type(),
        entity_id
    );
}

/// Generic system for adding physics to any entity that implements PhysicsProvider
pub fn add_entity_physics<T: PhysicsProvider + GameEntity + 'static>(
    entity: &T,
    commands: &mut Commands,
    entity_id: Entity,
) {
    let physics_bundle = entity.get_physics_bundle();
    commands.entity(entity_id).insert(physics_bundle);

    debug!(
        "Added {} physics to entity {:?}",
        entity.entity_type(),
        entity_id
    );
}

/// Generic spawning function for any spawnable entity
pub fn spawn_entity<T: Spawnable + 'static>(
    entity: T,
    commands: &mut Commands,
    position: Vec3,
    name: Option<String>,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) -> Entity {
    let spawn_position = position + entity.get_spawn_offset().unwrap_or(Vec3::ZERO);
    let entity_name =
        name.unwrap_or_else(|| format!("{}_{:?}", entity.entity_type(), spawn_position));

    let entity_id = commands
        .spawn((Position(spawn_position), Name::new(entity_name)))
        .id();

    // Add visuals and physics
    add_entity_visuals(&entity, commands, entity_id, meshes, materials);
    add_entity_physics(&entity, commands, entity_id);

    debug!(
        "Spawned {} at position {:?}",
        entity.entity_type(),
        spawn_position
    );
    entity_id
}

/// Convenience functions for spawning specific entities
pub fn spawn_floor(
    commands: &mut Commands,
    position: Vec3,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) -> Entity {
    spawn_entity(
        FloorEntity::default(),
        commands,
        position,
        Some("Floor".to_string()),
        meshes,
        materials,
    )
}

pub fn spawn_wall(
    wall_type: WallType,
    commands: &mut Commands,
    position: Vec3,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) -> Entity {
    spawn_entity(
        WallEntity::new(wall_type.clone()),
        commands,
        position,
        Some(wall_type.get_name().to_string()),
        meshes,
        materials,
    )
}

pub fn spawn_player(
    color: bevy::prelude::Color,
    commands: &mut Commands,
    position: Vec3,
    name: String,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) -> Entity {
    spawn_entity(
        PlayerEntity::with_color(color),
        commands,
        position,
        Some(name),
        meshes,
        materials,
    )
}

pub fn spawn_enemy(
    enemy_type: EnemyType,
    commands: &mut Commands,
    position: Vec3,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) -> Entity {
    spawn_entity(
        EnemyEntity::new(enemy_type.clone()),
        commands,
        position,
        Some(format!("{:?}_Enemy", enemy_type)),
        meshes,
        materials,
    )
}
