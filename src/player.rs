use crate::combat::{Health, Weapon};
use crate::game_state::GameState;
use avian3d::prelude::*;
use bevy::prelude::*;
use bevy::window::CursorGrabMode;
use leafwing_input_manager::prelude::*;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputManagerPlugin::<PlayerAction>::default())
            .add_systems(Startup, spawn_player)
            .add_systems(
                Update,
                (
                    player_movement,
                    player_look,
                    update_camera_to_player,
                    update_flashlight,
                    player_shooting,
                    stamina_system,
                    handle_cursor_grab,
                )
                    .run_if(in_state(GameState::InGame)),
            );
    }
}

#[derive(Component)]
pub struct FirstPersonCamera;

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
pub enum PlayerAction {
    #[actionlike(DualAxis)]
    Move,
    #[actionlike(DualAxis)]
    Look,
    Jump,
    Sprint,
    Crouch,
    Flashlight,
    PrimaryFire,
    SecondaryFire,
    Reload,
    Interact,
}

impl PlayerAction {
    fn default_input_map() -> InputMap<Self> {
        use PlayerAction::*;
        let mut input_map = InputMap::default();

        // Movement (WASD)
        input_map.insert_dual_axis(Move, VirtualDPad::wasd());

        // Mouse look
        input_map.insert_dual_axis(Look, MouseMove::default());

        // Individual key bindings
        input_map.insert(Jump, KeyCode::Space);
        input_map.insert(Sprint, KeyCode::ShiftLeft);
        input_map.insert(Crouch, KeyCode::ControlLeft);
        input_map.insert(Flashlight, KeyCode::KeyF);
        input_map.insert(PrimaryFire, MouseButton::Left);
        input_map.insert(SecondaryFire, MouseButton::Right);
        input_map.insert(Reload, KeyCode::KeyR);
        input_map.insert(Interact, KeyCode::KeyE);

        input_map
    }
}

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct PlayerController {
    pub speed: f32,
    pub sprint_speed: f32,
    pub crouch_speed: f32,
    pub jump_force: f32,
    pub sensitivity: f32,
    pub is_sprinting: bool,
    pub is_crouching: bool,
    pub stamina: f32,
    pub max_stamina: f32,
    pub pitch: f32,
    pub yaw: f32,
}

impl Default for PlayerController {
    fn default() -> Self {
        Self {
            speed: 5.0,
            sprint_speed: 8.0,
            crouch_speed: 2.0,
            jump_force: 5.0,
            sensitivity: 0.002,
            is_sprinting: false,
            is_crouching: false,
            stamina: 100.0,
            max_stamina: 100.0,
            pitch: 0.0,
            yaw: 0.0,
        }
    }
}

#[derive(Component)]
pub struct Flashlight {
    pub is_on: bool,
    pub intensity: f32,
    pub range: f32,
    pub angle: f32,
    pub battery: f32,
    pub max_battery: f32,
    pub drain_rate: f32,
}

impl Default for Flashlight {
    fn default() -> Self {
        Self {
            is_on: true,        // Start with flashlight on for testing
            intensity: 15000.0, // Increased from 10000 for better visibility
            range: 25.0,
            angle: 45.0,
            battery: 100.0,
            max_battery: 100.0,
            drain_rate: 3.0, // Reduced drain rate for testing
        }
    }
}

#[derive(Bundle)]
pub struct PlayerBundle {
    pub player: Player,
    pub controller: PlayerController,
    pub flashlight: Flashlight,
    pub weapon: Weapon,
    pub health: Health,
    pub rigid_body: RigidBody,
    pub collider: Collider,
    pub mass: Mass,
    pub locked_axes: LockedAxes,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub inherited_visibility: InheritedVisibility,
    pub view_visibility: ViewVisibility,
    pub input_map: InputMap<PlayerAction>,
    pub action_state: ActionState<PlayerAction>,
    pub name: Name,
}

fn spawn_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let player = commands
        .spawn(PlayerBundle {
            player: Player,
            controller: PlayerController::default(),
            flashlight: Flashlight::default(),
            weapon: Weapon::new_assault_rifle(),
            health: Health {
                current: 100.0,
                maximum: 100.0,
                regeneration_rate: 0.0,
                last_damage_time: 0.0,
            },
            rigid_body: RigidBody::Dynamic,
            collider: Collider::capsule(0.5, 1.8),
            mass: Mass(70.0),
            locked_axes: LockedAxes::ROTATION_LOCKED,
            transform: Transform::from_xyz(-2.0, 0.9, 0.0),
            global_transform: GlobalTransform::default(),
            visibility: Visibility::default(),
            inherited_visibility: InheritedVisibility::default(),
            view_visibility: ViewVisibility::default(),
            input_map: PlayerAction::default_input_map(),
            action_state: ActionState::<PlayerAction>::default(),
            name: Name::new("Player"),
        })
        .id();

    // Add flashlight as a child entity (starts on for testing)
    commands.entity(player).with_children(|parent| {
        parent.spawn((
            SpotLight {
                intensity: 15000.0, // Start bright for testing
                range: 25.0,
                radius: 0.1,
                outer_angle: 0.8,
                inner_angle: 0.6,
                shadows_enabled: true,
                color: Color::srgb(1.0, 0.95, 0.8),
                ..default()
            },
            Transform::from_xyz(0.2, 0.5, 0.3),
            GlobalTransform::default(),
            Visibility::default(),
            InheritedVisibility::default(),
            ViewVisibility::default(),
            Name::new("Flashlight"),
        ));
    });
}

fn player_movement(
    time: Res<Time>,
    mut query: Query<
        (
            &mut Transform,
            &mut LinearVelocity,
            &mut PlayerController,
            &ActionState<PlayerAction>,
        ),
        With<Player>,
    >,
) {
    for (mut transform, mut velocity, mut controller, action_state) in query.iter_mut() {
        let mut movement = Vec3::ZERO;

        // Get movement input
        let move_data = action_state.axis_pair(&PlayerAction::Move);
        if move_data != Vec2::ZERO {
            movement.x = move_data.x;
            movement.z = move_data.y; // W = forward (+Z), S = backward (-Z)
        }

        // Handle sprinting and crouching
        controller.is_sprinting =
            action_state.pressed(&PlayerAction::Sprint) && controller.stamina > 0.0;
        controller.is_crouching = action_state.pressed(&PlayerAction::Crouch);

        // Calculate speed
        let speed = if controller.is_sprinting {
            controller.sprint_speed
        } else if controller.is_crouching {
            controller.crouch_speed
        } else {
            controller.speed
        };

        // Apply movement relative to player rotation
        let forward: Vec3 = transform.forward().into();
        let right: Vec3 = transform.right().into();
        let movement_world: Vec3 = (right * movement.x + forward * movement.z) * speed;

        // Apply movement to velocity, preserving Y velocity for gravity
        velocity.x = movement_world.x;
        velocity.z = movement_world.z;

        // Jumping
        if action_state.just_pressed(&PlayerAction::Jump) && velocity.y.abs() < 0.1 {
            velocity.y = controller.jump_force;
        }
    }
}

fn player_look(
    mut query: Query<
        (
            &mut Transform,
            &mut PlayerController,
            &ActionState<PlayerAction>,
        ),
        With<Player>,
    >,
) {
    for (mut transform, mut controller, action_state) in query.iter_mut() {
        let look_delta = action_state.axis_pair(&PlayerAction::Look);
        if look_delta != Vec2::ZERO {
            // Update yaw (horizontal rotation)
            controller.yaw -= look_delta.x * controller.sensitivity;

            // Update pitch (vertical rotation) and clamp it
            controller.pitch -= look_delta.y * controller.sensitivity;
            controller.pitch = controller.pitch.clamp(-1.5, 1.5);

            // Apply yaw rotation to player transform
            transform.rotation = Quat::from_rotation_y(controller.yaw);
        }
    }
}

fn update_camera_to_player(
    player_query: Query<
        (&Transform, &PlayerController),
        (With<Player>, Without<FirstPersonCamera>),
    >,
    mut camera_query: Query<&mut Transform, (With<FirstPersonCamera>, Without<Player>)>,
) {
    if let (Ok((player_transform, controller)), Ok(mut camera_transform)) =
        (player_query.single(), camera_query.single_mut())
    {
        // Position camera at player head height
        let eye_height = if controller.is_crouching { 1.2 } else { 1.7 };
        let camera_position = player_transform.translation + Vec3::new(0.0, eye_height, 0.0);
        camera_transform.translation = camera_position;

        // Apply player's yaw and camera's pitch
        let yaw_rotation = Quat::from_rotation_y(controller.yaw);
        let pitch_rotation = Quat::from_rotation_x(controller.pitch);
        camera_transform.rotation = yaw_rotation * pitch_rotation;
    }
}

fn update_flashlight(
    time: Res<Time>,
    mut flashlight_query: Query<&mut Flashlight, With<Player>>,
    mut light_query: Query<&mut SpotLight>,
    action_query: Query<&ActionState<PlayerAction>, With<Player>>,
) {
    for mut flashlight in flashlight_query.iter_mut() {
        if let Ok(action_state) = action_query.single() {
            // Toggle flashlight
            if action_state.just_pressed(&PlayerAction::Flashlight) {
                flashlight.is_on = !flashlight.is_on;
            }

            // Drain battery when on
            if flashlight.is_on && flashlight.battery > 0.0 {
                flashlight.battery -= flashlight.drain_rate * time.delta_secs();
                flashlight.battery = flashlight.battery.max(0.0);
            }

            // Turn off if battery is dead
            if flashlight.battery <= 0.0 {
                flashlight.is_on = false;
            }

            // Update light intensity
            for mut light in light_query.iter_mut() {
                light.intensity = if flashlight.is_on {
                    flashlight.intensity * (flashlight.battery / flashlight.max_battery).max(0.1)
                } else {
                    0.0
                };
            }
        }
    }
}

fn player_shooting(
    time: Res<Time>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut player_query: Query<(&Transform, &mut Weapon, &ActionState<PlayerAction>), With<Player>>,
    camera_query: Query<&Transform, (With<FirstPersonCamera>, Without<Player>)>,
) {
    for (player_transform, mut weapon, action_state) in player_query.iter_mut() {
        if let Ok(camera_transform) = camera_query.single() {
            // Handle shooting
            if action_state.pressed(&PlayerAction::PrimaryFire) {
                let current_time = time.elapsed_secs();
                let time_since_last_shot = current_time - weapon.last_shot_time;
                let fire_interval = 1.0 / weapon.rate_of_fire;

                if time_since_last_shot >= fire_interval
                    && weapon.ammo_current > 0
                    && !weapon.is_reloading
                {
                    weapon.last_shot_time = current_time;
                    weapon.ammo_current -= 1;

                    // Create projectile
                    let projectile_material = materials.add(StandardMaterial {
                        base_color: Color::srgb(1.0, 1.0, 0.5),
                        emissive: Color::srgb(0.8, 0.8, 0.2).into(),
                        ..default()
                    });

                    // Shoot from camera position in camera direction
                    let shoot_origin =
                        camera_transform.translation + camera_transform.forward() * 0.5;
                    let shoot_direction = camera_transform.forward();

                    commands.spawn((
                        Mesh3d(meshes.add(Sphere::new(0.05))),
                        MeshMaterial3d(projectile_material),
                        Transform::from_translation(shoot_origin),
                        RigidBody::Dynamic,
                        Collider::sphere(0.05),
                        LinearVelocity(shoot_direction * 50.0), // Fast projectile
                        Mass(0.1),
                        crate::combat::Projectile {
                            damage: weapon.damage,
                            speed: 50.0,
                            lifetime: 5.0,
                            penetration: 1,
                            owner: Entity::PLACEHOLDER, // Temporary placeholder
                        },
                        Name::new("Bullet"),
                    ));

                    info!("Shot fired! Ammo remaining: {}", weapon.ammo_current);
                }
            }

            // Handle reloading
            if action_state.just_pressed(&PlayerAction::Reload)
                && !weapon.is_reloading
                && weapon.ammo_current < weapon.ammo_max
            {
                weapon.is_reloading = true;
                weapon.reload_start_time = time.elapsed_secs();
                info!("Reloading...");
            }

            // Check if reload is complete
            if weapon.is_reloading {
                let reload_elapsed = time.elapsed_secs() - weapon.reload_start_time;
                if reload_elapsed >= weapon.reload_time {
                    weapon.is_reloading = false;
                    weapon.ammo_current = weapon.ammo_max;
                    info!("Reload complete! Ammo: {}", weapon.ammo_current);
                }
            }
        }
    }
}

fn stamina_system(time: Res<Time>, mut query: Query<&mut PlayerController, With<Player>>) {
    for mut controller in query.iter_mut() {
        if controller.is_sprinting {
            // Drain stamina while sprinting
            controller.stamina -= 20.0 * time.delta_secs();
            controller.stamina = controller.stamina.max(0.0);
            info!("Sprinting! Stamina: {:.1}", controller.stamina);
        } else {
            // Regenerate stamina when not sprinting
            controller.stamina += 15.0 * time.delta_secs();
            controller.stamina = controller.stamina.min(controller.max_stamina);
        }
    }
}

fn handle_cursor_grab(
    mut windows: Query<&mut Window>,
    mouse: Res<ButtonInput<MouseButton>>,
    key: Res<ButtonInput<KeyCode>>,
) {
    if let Ok(mut window) = windows.single_mut() {
        if mouse.just_pressed(MouseButton::Left) {
            window.cursor_options.visible = false;
            window.cursor_options.grab_mode = CursorGrabMode::Locked;
        }

        if key.just_pressed(KeyCode::Escape) {
            window.cursor_options.visible = true;
            window.cursor_options.grab_mode = CursorGrabMode::None;
        }
    }
}
