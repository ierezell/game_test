use crate::input::get_player_input_map;
use avian3d::prelude::{LinearVelocity, Position};
use bevy::app::Update;
use bevy::prelude::{
    App, Assets, Capsule3d, Color, Commands, Entity, FixedUpdate, Mesh, Mesh3d, MeshMaterial3d,
    Name, Plugin, Query, Res, ResMut, Single, StandardMaterial, Transform, Vec3, With, Without,
    debug, default, info,
};
use leafwing_input_manager::prelude::ActionState;
use shared::entities::{NpcPhysicsBundle, PlayerPhysicsBundle};
use shared::input::PlayerAction;

use crate::LocalPlayerId;
use lightyear::prelude::{
    Controlled, Interpolated, LocalTimeline, NetworkTimeline, Predicted, PredictionManager,
};
use shared::input::{PLAYER_CAPSULE_HEIGHT, PLAYER_CAPSULE_RADIUS};

use shared::protocol::{CharacterMarker, PlayerColor, PlayerId};

pub struct ClientEntitiesPlugin;

impl Plugin for ClientEntitiesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, debug_interpolated_npc_positions);
        app.add_systems(Update, handle_interpolated_npcs_setup);
        app.add_systems(FixedUpdate, debug_player_position);
        app.add_systems(Update, handle_local_player_setup);
        app.add_systems(Update, handle_interpolated_players_setup);
    }
}

/// Debug system: log replicated Position for interpolated NPCs so we can verify replication
fn debug_interpolated_npc_positions(
    query: Query<
        (Entity, &Name, &Position, Option<&Transform>),
        (With<Interpolated>, Without<PlayerId>),
    >,
) {
    for (entity, name, position, maybe_transform) in query.iter() {
        match maybe_transform {
            Some(transform) => {
                if transform.translation == Vec3::ZERO && position.0 != Vec3::ZERO {
                    info!(
                        "C: INTERP NPC {:?} ({}) has Position {:?} but Transform is ZERO (replication may not have written yet)",
                        entity,
                        name.as_str(),
                        position.0
                    );
                } else {
                    debug!(
                        "C: INTERP NPC {:?} ({}) Position: {:?} Transform: {:?}",
                        entity,
                        name.as_str(),
                        position.0,
                        transform.translation
                    );
                }
            }
            None => {
                // Transform missing — this will render at the origin until a Transform exists.
                if position.0 == Vec3::ZERO {
                    info!(
                        "C: INTERP NPC {:?} ({}) has zero Position and no Transform: {:?}",
                        entity,
                        name.as_str(),
                        position
                    );
                } else {
                    info!(
                        "C: INTERP NPC {:?} ({}) has Position {:?} but no Transform present — inserting visuals without Transform will render at origin",
                        entity,
                        name.as_str(),
                        position.0
                    );
                }
            }
        }
    }
}

fn debug_player_position(
    player_query: Query<
        (&Name, &Position, &LinearVelocity),
        (
            With<PlayerId>,
            With<Predicted>,
            With<Controlled>,
            With<CharacterMarker>,
        ),
    >,
    timeline: Single<&LocalTimeline, With<PredictionManager>>,
) {
    for (name, position, linear_velocity) in player_query.iter() {
        debug!(
            "C:{:?} pos:{:?} vel:{:?} tick:{:?}",
            name,
            position,
            linear_velocity,
            timeline.tick()
        );
    }
}

fn handle_local_player_setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    player_query: Query<
        (Entity, &Name, &PlayerColor, &PlayerId),
        (
            With<Predicted>,
            With<Controlled>,
            With<PlayerId>,
            Without<Mesh3d>,
        ),
    >,
    local_player_id: Res<LocalPlayerId>,
) {
    // TODO : may use single here ?
    for (entity, name, color, player_id) in player_query.iter() {
        info!(
            "🔍 CLIENT: Entity ready for setup: player_id={:?}, local_player_id={}",
            player_id.0, local_player_id.0
        );

        if player_id.0.to_bits() == local_player_id.0 {
            info!(
                "✅ CLIENT: This is our local player! Adding rendering and input to entity {:?}",
                name
            );

            let input_map = get_player_input_map();
            commands.entity(entity).insert((
                Mesh3d(meshes.add(Capsule3d::new(PLAYER_CAPSULE_RADIUS, PLAYER_CAPSULE_HEIGHT))),
                MeshMaterial3d(materials.add(color.0)),
                input_map,
                ActionState::<PlayerAction>::default(),
                PlayerPhysicsBundle::default(),
            ));

            info!(
                "✅ CLIENT: Local player rendering and input setup complete for entity {:?}",
                entity
            );
        } else {
            info!(
                "ℹ️ CLIENT: Predicted/Controlled entity is not local (id: {:?})",
                player_id.0
            );
        }
    }
}

fn handle_interpolated_players_setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    player_query: Query<
        (Entity, &Name, &PlayerColor),
        (With<Interpolated>, With<CharacterMarker>, Without<Mesh3d>),
    >,
) {
    for (entity, name, color) in player_query.iter() {
        // Insert a default Transform so the physics_transform system can populate
        // it from replicated `Position` (see `PhysicsTransformConfig`).
        commands.entity(entity).insert((
            Mesh3d(meshes.add(Capsule3d::new(PLAYER_CAPSULE_RADIUS, PLAYER_CAPSULE_HEIGHT))),
            MeshMaterial3d(materials.add(color.0)),
            NpcPhysicsBundle::default(),
        ));
        info!(
            "✅ CLIENT: INTERPOLATED player setup complete! Entity: {:?} Player: {:?}",
            entity, name
        );
    }
}

fn handle_interpolated_npcs_setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    npc_query: Query<
        (Entity, &Name, &Position),
        (With<Interpolated>, Without<PlayerId>, Without<Mesh3d>),
    >,
) {
    // Only spawn visuals when a replicated Position is present; this avoids visuals at (0,0,0)
    for (entity, name, position) in npc_query.iter() {
        info!(
            "🤖 CLIENT: Spawning NPC visual for entity {:?} ({}) ({:?})",
            entity,
            name.as_str(),
            position.0
        );

        // Determine NPC type and color based on name
        let color = if name.as_str().contains("Enemy") {
            Color::srgb(0.8, 0.2, 0.2) // Red enemies, taller
        } else if name.as_str().contains("Guard") {
            Color::srgb(0.9, 0.4, 0.1) // Orange guards
        } else if name.as_str().contains("Bot") {
            Color::srgb(0.2, 0.2, 0.8) // Blue bots
        } else if name.as_str().contains("Scout") {
            Color::srgb(0.1, 0.7, 0.3) // Green scouts
        } else {
            Color::srgb(0.5, 0.5, 0.5) // Default gray
        };

        // Insert a default Transform so the `PhysicsTransformConfig` (position->transform)
        // can populate the entity's Transform from the replicated `Position` when it arrives.
        commands.entity(entity).insert((
            Mesh3d(meshes.add(Capsule3d::new(PLAYER_CAPSULE_RADIUS, PLAYER_CAPSULE_HEIGHT))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: color,
                emissive: color.to_linear() * 0.5, // Slight glow
                ..default()
            })),
            NpcPhysicsBundle::default(),
        ));

        info!(
            "✅ CLIENT: NPC visual setup complete for entity {:?} ({})",
            entity,
            name.as_str()
        );
    }
}
