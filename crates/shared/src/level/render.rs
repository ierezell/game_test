// use avian3d::prelude::Position;
// use bevy::prelude::{
//     Add, Assets, Commands, Dir3, Entity, Mesh, Mesh3d, MeshMaterial3d, Name, On, Plane3d, Query,
//     ResMut, StandardMaterial, Vec2, Vec3, Without, debug, default,
// };

// /// Observer function for adding floor visuals using entity system
// pub fn add_floor_visuals(
//     trigger: On<Add, FloorMarker>,
//     floor_query: Query<(Entity, &Position)>,
//     mut commands: Commands,
//     mut meshes: ResMut<Assets<Mesh>>,
//     mut materials: ResMut<Assets<StandardMaterial>>,
// ) {
//     let Ok((_entity, _position)) = floor_query.get(trigger.entity) else {
//         debug!("Failed to get floor entity for visual addition.");
//         return;
//     };
//     commands.spawn((
//         Mesh3d(meshes.add(Plane3d {
//             normal: Dir3::Y,
//             half_size: Vec2::splat(50.0),
//         })),
//         MeshMaterial3d(materials.add(StandardMaterial { ..default() })),
//     ));
// }

// /// Observer function for adding wall visuals using entity system
// pub fn add_wall_visuals(
//     trigger: On<Add, WallMarker>,
//     wall_query: Query<(Entity, &Position, &Name), Without<Mesh3d>>,
//     mut commands: Commands,
//     mut meshes: ResMut<Assets<Mesh>>,
//     mut materials: ResMut<Assets<StandardMaterial>>,
// ) {
//     let Ok((_entity, _position, _name)) = wall_query.get(trigger.entity) else {
//         debug!("Failed to get wall entity for visual addition.");
//         return;
//     };

//     for (entity, _position, name) in wall_query.iter() {
//         // Determine wall type from name and create appropriate physics bundle
//         let size = if name.as_str().contains("North") || name.as_str().contains("South") {
//             Vec3::new(ROOM_SIZE, WALL_HEIGHT, WALL_THICKNESS)
//         } else {
//             Vec3::new(WALL_THICKNESS, WALL_HEIGHT, ROOM_SIZE)
//         };

//         commands.entity(entity).insert((
//             Mesh3d(meshes.add(Cuboid { half_size: size })),
//             MeshMaterial3d(materials.add(StandardMaterial { ..default() })),
//         ));
//         debug!("Added wall physics to entity {:?} ({})", entity, name);
//     }
// }

// /// Setup basic lighting for the scene
// pub fn setup_lighting(mut commands: Commands) {
//     // Add ambient lighting for better visibility
//     commands.insert_resource(AmbientLight {
//         color: WHITE.into(),
//         brightness: 0.3,
//         affects_lightmapped_meshes: true,
//     });

//     // Main directional light (sun)
//     commands.spawn((
//         DirectionalLight {
//             color: WHITE.into(),
//             illuminance: 10000.0,
//             ..default()
//         },
//         Transform::from_xyz(10.0, 10.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
//         Name::new("Sun"),
//     ));

//     debug!("✅ Lighting setup complete with ambient and directional light");
// }
