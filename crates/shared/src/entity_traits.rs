use avian3d::prelude::{Collider, RigidBody};
use bevy::prelude::{Bundle, Color, Mesh, StandardMaterial};

/// Trait for entities that can provide visual representation
pub trait VisualProvider {
    /// Get the mesh for this entity type
    fn get_mesh(&self) -> Mesh;

    /// Get the material for this entity type
    fn get_material(&self) -> StandardMaterial;

    /// Get the base color for this entity type
    fn get_color(&self) -> Color;
}

/// Trait for entities that can provide physics representation
pub trait PhysicsProvider {
    /// Associated type for the physics bundle
    type PhysicsBundle: Bundle;

    /// Get the physics bundle for this entity type
    fn get_physics_bundle(&self) -> Self::PhysicsBundle;

    /// Get the collider for this entity type
    fn get_collider(&self) -> Collider;

    /// Get the rigid body type for this entity type
    fn get_rigid_body(&self) -> RigidBody;
}

/// Trait combining both visual and physics capabilities
pub trait GameEntity: VisualProvider + PhysicsProvider {
    /// Entity type identifier
    fn entity_type(&self) -> &'static str;
}

/// Marker trait for entities that can be spawned
pub trait Spawnable: GameEntity {
    /// Get spawn position offset if any
    fn get_spawn_offset(&self) -> Option<bevy::prelude::Vec3> {
        None
    }
}
