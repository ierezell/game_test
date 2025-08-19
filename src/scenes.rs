use avian3d::prelude::{Collider, RigidBody};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

// Scene components for serializable entities
#[derive(Component, Reflect, Serialize, Deserialize, Clone)]
#[reflect(Component, Serialize, Deserialize)]
pub struct LevelData {
    pub name: String,
    pub difficulty: u32,
    pub enemy_spawn_points: Vec<Vec3>,
    pub player_spawn_point: Vec3,
    pub ambient_light_color: Color,
    pub fog_settings: FogSettings,
}

#[derive(Component, Reflect, Serialize, Deserialize, Clone)]
#[reflect(Component, Serialize, Deserialize)]
pub struct RoomData {
    pub room_type: RoomType,
    pub dimensions: Vec3,
    pub lighting_intensity: f32,
    pub has_ceiling: bool,
    pub wall_material: WallMaterial,
}

#[derive(Reflect, Serialize, Deserialize, Clone, Debug)]
pub enum RoomType {
    Corridor,
    LargeRoom,
    SmallRoom,
    Junction,
    Stairwell,
}

#[derive(Reflect, Serialize, Deserialize, Clone, Debug)]
pub enum WallMaterial {
    Concrete,
    Metal,
    Wood,
    Brick,
}

#[derive(Component, Reflect, Serialize, Deserialize, Clone)]
#[reflect(Component, Serialize, Deserialize)]
pub struct EnemySpawnData {
    pub enemy_type: EnemyType,
    pub patrol_points: Vec<Vec3>,
    pub alert_radius: f32,
    pub initial_state: EnemySpawnState,
    pub equipment: Vec<Equipment>,
}

#[derive(Reflect, Serialize, Deserialize, Clone, Debug)]
pub enum EnemyType {
    Basic,
    Heavy,
    Scout,
    Elite,
}

#[derive(Reflect, Serialize, Deserialize, Clone, Debug)]
pub enum EnemySpawnState {
    Dormant,
    Patrolling,
    Alert,
}

#[derive(Reflect, Serialize, Deserialize, Clone, Debug)]
pub enum Equipment {
    Rifle,
    Flashlight,
    RadioDevice,
    Armor,
}

#[derive(Component, Reflect, Serialize, Deserialize, Clone)]
#[reflect(Component, Serialize, Deserialize)]
pub struct InteractableData {
    pub interaction_type: InteractionType,
    pub interaction_text: String,
    pub requires_key: bool,
    pub one_time_use: bool,
}

#[derive(Reflect, Serialize, Deserialize, Clone, Debug)]
pub enum InteractionType {
    Door,
    Switch,
    Pickup,
    Terminal,
    Keycard,
}

#[derive(Component, Reflect, Serialize, Deserialize, Clone)]
#[reflect(Component, Serialize, Deserialize)]
pub struct LightData {
    pub light_type: LightType,
    pub intensity: f32,
    pub color: Color,
    pub range: f32,
    pub flicker: bool,
    pub shadow_enabled: bool,
}

#[derive(Reflect, Serialize, Deserialize, Clone, Debug)]
pub enum LightType {
    Point,
    Spot,
    Directional,
    Area,
}

#[derive(Reflect, Serialize, Deserialize, Clone)]
pub struct FogSettings {
    pub color: Color,
    pub density: f32,
    pub start_distance: f32,
    pub end_distance: f32,
}

// Marker components for scene management
#[derive(Component)]
pub struct SceneEntity;

#[derive(Component)]
pub struct LevelRoot;

#[derive(Component)]
pub struct RoomRoot;

// Scene templates for different room types (simplified to remove unused fields)
pub struct RoomTemplate {
    pub room_type: RoomType,
    pub prefab_path: String,
}

impl Default for RoomTemplate {
    fn default() -> Self {
        Self {
            room_type: RoomType::SmallRoom,
            prefab_path: "scenes/rooms/small_room.scn.ron".to_string(),
        }
    }
}

// Level generation using scenes
pub fn create_level_scene(
    commands: &mut Commands,
    asset_server: Res<AssetServer>,
    level_data: &LevelData,
) -> Entity {
    let level_entity = commands
        .spawn((
            SceneRoot::default(),
            LevelRoot,
            level_data.clone(),
            Transform::default(),
            Visibility::default(),
            Name::new(format!("Level: {}", level_data.name)),
        ))
        .id();

    // Load room scenes based on level data
    let room_scenes = get_room_scenes_for_level(level_data);

    for (room_position, room_template) in room_scenes {
        let room_scene: Handle<Scene> = asset_server.load(&room_template.prefab_path);

        commands.spawn((
            SceneRoot(room_scene),
            Transform::from_translation(room_position),
            RoomRoot,
            RoomData {
                room_type: room_template.room_type.clone(),
                dimensions: Vec3::new(10.0, 3.0, 10.0), // Default room size
                lighting_intensity: 1.0,
                has_ceiling: true,
                wall_material: WallMaterial::Concrete,
            },
            Name::new(format!("Room: {:?}", room_template.room_type)),
            ChildOf(level_entity),
        ));
    }

    level_entity
}

// Room scene templates
fn get_room_scenes_for_level(level_data: &LevelData) -> Vec<(Vec3, RoomTemplate)> {
    let mut rooms = Vec::new();

    // Generate a simple linear layout for now
    let room_spacing = 20.0;
    let room_count = (level_data.difficulty + 2) as usize;

    for i in 0..room_count {
        let position = Vec3::new(i as f32 * room_spacing, 0.0, 0.0);

        let room_type = match i {
            0 => RoomType::SmallRoom,                        // Starting room
            i if i == room_count - 1 => RoomType::LargeRoom, // Final room
            _ => {
                if i % 3 == 0 {
                    RoomType::Junction
                } else {
                    RoomType::Corridor
                }
            }
        };

        let template = RoomTemplate {
            room_type: room_type.clone(),
            prefab_path: get_scene_path_for_room_type(&room_type),
        };

        rooms.push((position, template));
    }

    rooms
}

fn get_scene_path_for_room_type(room_type: &RoomType) -> String {
    match room_type {
        RoomType::Corridor => "scenes/rooms/corridor.scn.ron".to_string(),
        RoomType::LargeRoom => "scenes/rooms/large_room.scn.ron".to_string(),
        RoomType::SmallRoom => "scenes/rooms/small_room.scn.ron".to_string(),
        RoomType::Junction => "scenes/rooms/junction.scn.ron".to_string(),
        RoomType::Stairwell => "scenes/rooms/stairwell.scn.ron".to_string(),
    }
}

// Scene spawning system
pub fn spawn_enemies_from_scene_data(
    mut commands: Commands,
    spawn_data_query: Query<(&EnemySpawnData, &Transform), Added<EnemySpawnData>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (spawn_data, transform) in spawn_data_query.iter() {
        spawn_enemy_at_position(
            &mut commands,
            spawn_data,
            transform.translation,
            &mut meshes,
            &mut materials,
        );
    }
}

fn spawn_enemy_at_position(
    commands: &mut Commands,
    spawn_data: &EnemySpawnData,
    position: Vec3,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    let enemy_color = match spawn_data.enemy_type {
        EnemyType::Basic => Color::srgb(0.8, 0.2, 0.2),
        EnemyType::Heavy => Color::srgb(0.2, 0.2, 0.8),
        EnemyType::Scout => Color::srgb(0.2, 0.8, 0.2),
        EnemyType::Elite => Color::srgb(0.8, 0.8, 0.2),
    };

    commands.spawn((
        Mesh3d(meshes.add(Mesh::from(Capsule3d::new(0.5, 2.0)))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: enemy_color,
            ..default()
        })),
        Transform::from_translation(position),
        RigidBody::Dynamic,
        Collider::capsule(2.0, 0.5),
        spawn_data.clone(),
        SceneEntity,
        Name::new(format!("Enemy: {:?}", spawn_data.enemy_type)),
    ));
}

// Scene serialization and loading functions removed - unused dead code

// Scene spawn events removed - unused dead code

// Scene validation system
pub fn validate_scene_integrity(
    scene_query: Query<(Entity, &SceneRoot), With<LevelRoot>>,
    room_query: Query<&RoomData>,
    enemy_query: Query<&EnemySpawnData>,
) {
    for (level_entity, _scene_root) in scene_query.iter() {
        let room_count = room_query.iter().count();
        let enemy_count = enemy_query.iter().count();

        info!(
            "Level {:?} validation: {} rooms, {} enemies",
            level_entity, room_count, enemy_count
        );

        // Validate minimum requirements
        if room_count == 0 {
            warn!("Level {:?} has no rooms!", level_entity);
        }

        if enemy_count == 0 {
            warn!("Level {:?} has no enemies!", level_entity);
        }
    }
}

// Scene cleanup system removed - unused dead code

// Plugin to register all scene-related systems
pub struct GameScenesPlugin;

impl Plugin for GameScenesPlugin {
    fn build(&self, app: &mut App) {
        app
            // Register reflected types for serialization
            .register_type::<LevelData>()
            .register_type::<RoomData>()
            .register_type::<EnemySpawnData>()
            .register_type::<InteractableData>()
            .register_type::<LightData>()
            .register_type::<RoomType>()
            .register_type::<WallMaterial>()
            .register_type::<EnemyType>()
            .register_type::<EnemySpawnState>()
            .register_type::<Equipment>()
            .register_type::<InteractionType>()
            .register_type::<LightType>()
            .register_type::<FogSettings>()
            // Register systems
            .add_systems(
                Update,
                (spawn_enemies_from_scene_data, validate_scene_integrity),
            )
            .add_systems(
                PostUpdate,
                (
                    // Scene validation runs after all spawning
                    validate_scene_integrity,
                ),
            );
    }
}
