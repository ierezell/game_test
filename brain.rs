    app.insert_resource(avian3d::physics_transform::PhysicsTransformConfig {
        transform_to_position: false,
        position_to_transform: true,
        ..default()
    });

    app.add_plugins(PhysicsPlugins::default().build()
        // disable the position<>transform sync plugins as it is handled by lightyear_avian
        .disable::<PhysicsTransformPlugin>()
        .disable::<PhysicsInterpolationPlugin>()
        .disable::<IslandPlugin>()
        .disable::<IslandSleepingPlugin>(),
    );

    // Tnua will run in rollback
    app.add_plugins(TnuaControllerPlugin::new(FixedUpdate));
    app.add_plugins(TnuaAvian3dPlugin::new(FixedUpdate));

    #[debug_fail]
fn spawn_avatar_observer(
    trigger: On<PlayerSpawnedEvent>,
    mut commands: Commands,
    players: PlayerParam,
) -> Result {
    let PlayerSpawnedEvent { player } = *trigger;

    #[fail_msg = "Unable to retrieve client entity when spawning player avatar"]
    let owner = players.get_client(player)?;

    commands.spawn((
        ReplicationGroup::new_id(player.to_bits()),
        AvatarComponent {
            avatar_color: get_random_avatar_color(),
        },
        ControllingPlayerRelation { player },
        DespawnOnExit(ServerStates::Listening),
        PredictionTarget::to_clients(NetworkTarget::All),
        // TODO: (Brain) remove control when player posesses a different one and re-add when possessing
        ControlledBy {
            owner,
            lifetime: Lifetime::SessionBased,
        },
        NetworkInputComponent,
        DelayedReplicationComponent,
        DisableReplicateHierarchy,
        Replicate::to_clients(NetworkTarget::All),
    ));

    OK
}

    // Unable to convert to an observer since lightyear components won't be on the entity on the host at trigger time
    #[debug_fail]
    fn avatar_replicated_system(
        mut commands: Commands,
        avatar_query: Populated<(Entity, Has<Controlled>), Added<AvatarComponent>>,
        descriptors: DescriptorParam,
    ) -> Result {
        #[fail_msg = " Unable to find avatar descriptor"]
        let descriptor = descriptors.get_id("avatar")?;

        for (entity, is_controlled) in avatar_query.iter() {
            commands.entity(entity).try_insert((
                // TODO: (Brain) Gameplay scoping here instead?
                DespawnOnExit(ClientStates::Connected),
                Transform::default(),
                // FrameInterpolate::<Transform>::default(),
            ));

            // Make the avatar model a child of the avatar so that we can do transforms to it separately
            let Ok(scene_handle) = descriptors.get(descriptor)?.get_animated_scene() else {
                fail!("Unable to get avatar asset scene");
                continue;
            };
            commands.spawn((
                ChildOf(entity),
                Transform::default().with_translation(Vec3::ZERO.with_y(-1.0)),
                AnimatedSceneHandle {
                    handle: scene_handle.clone(),
                    ..default()
                },
            ));

            // Everything beyond this point should only be for the player's avatar
            if !is_controlled {
                continue;
            }

            // Set up player controls
            commands.entity(entity).try_insert((
                LocalInputComponent,
            ));

            commands.trigger(ControlledAvatarReplicatedEvent {
                avatar: entity
            });
        }

        OK
    }

    fn handle_player_spawn(
    trigger: On<Add, Controlled>,
    player_query: Query<
        (&Name, &PlayerColor, &PlayerId),
        (With<Predicted>, With<Controlled>, With<PlayerId>),
    >,


        fn add_local_input_observer(
        trigger: On<Add, LocalInputComponent>,
        mut commands: Commands,
    ) {
        // TODO: (Brain) input map in settings file
        commands.entity(trigger.entity).insert((
            // Networked actions
            Actions::<NetworkInputComponent>::spawn(SpawnWith(|context: &mut ActionSpawner<_>| {
                context.spawn((
                    Action::<MoveAction>::new(),
                    LocalNetworkedActionComponent, // needed to target since remote players will have
                    Negate::y(),
                    Bindings::spawn((
                        Cardinal::wasd_keys(),
                        Axial::left_stick().with((
                            DeadZone::default(),
                        ))
                    )),
                ));

                context.spawn((
                    Action::<CameraForwardAction>::new(),
                    LocalNetworkedActionComponent, // needed to target since remote players will have
                    // no bindings since this is mocked
                    ActionMock::new(
                        ActionState::Fired,
                        ActionValue::Axis3D(Dir3::NEG_Z.as_vec3()),
                        MockSpan::Manual,
                    ),
                ));

                context.spawn((
                    Action::<JumpAction>::new(),
                    LocalNetworkedActionComponent, // needed to target since remote players will have
                    bindings![KeyCode::Space, GamepadButton::South]
                ));

                context.spawn((
                    Action::<SprintAction>::new(),
                    LocalNetworkedActionComponent, // needed to target since remote players will have
                    bindings![KeyCode::ShiftLeft, GamepadButton::LeftTrigger]
                ));

                #[cfg(feature = "dev")]
                context.spawn((
                    Action::<DebugSuperSpeedAction>::new(),
                    LocalNetworkedActionComponent, // needed to target since remote players will have
                    // No binding since this is toggled elsewhere
                    ActionMock::new(
                        ActionState::Fired,
                        ActionValue::Bool(false),
                        MockSpan::Manual,
                    ),
                ));
            })),

            Actions::<LocalInputComponent>::spawn(SpawnWith(|context: &mut ActionSpawner<_>| {
                // ... Local actions
            })),
        ));
    }

    #[derive(InputAction, Debug)]
#[action_output(Vec2)]
pub struct MoveAction;

#[derive(InputAction, Debug)]
#[action_output(Vec3)]
pub struct CameraForwardAction;

#[derive(InputAction, Debug)]
#[action_output(bool)]
pub struct SprintAction;

#[cfg(feature = "dev")]
#[derive(InputAction, Debug)]
#[action_output(bool)]
pub struct DebugSuperSpeedAction;

#[derive(InputAction, Debug)]
#[action_output(bool)]
pub struct JumpAction;


#[derive(Component, Debug, Reflect, Serialize, Deserialize, Clone, PartialEq)]
#[reflect(Component)]
#[cfg_attr(feature = "dev", require(
    Name::new("Avatar"),
))]
#[cfg_attr(feature = "client", require(
    NoCameraCollision,
    Visibility::default(),
    bevy_tnua::TnuaAnimatingState::<PlayerAnimationStates>::default(),
))]
#[require(
    Position::new(get_random_vec3(Vec3::new(-5.0, 10.0, -5.0), Vec3::new(5.0, 10.0, 5.0))),
    RigidBody::Dynamic,
    Collider::capsule(0.3, 0.8),
    LockedAxes::ROTATION_LOCKED,
    Transform,
    SleepingDisabled,

    TnuaController,
    TnuaAvian3dSensorShape(Collider::sphere(0.1)),
    InputAccumulationComponent,
)]
pub struct AvatarComponent {
    pub avatar_color: Color,
}

#[derive(Component, Default, Debug, Reflect, Serialize, Deserialize, Clone, PartialEq)]
#[reflect(Component)]
pub struct NetworkInputComponent;

#[derive(Component, Default, Debug, Reflect, Clone, PartialEq)]
#[reflect(Component)]
pub struct InputAccumulationComponent {
    pub movement: Vec2,
    pub camera: Vec3,
    pub sprint: bool,
    pub jump: bool,

    #[cfg(feature = "dev")]
    pub debug_super_speed: bool,
}


    fn handle_new_client(
        trigger: On<Add, Connected>,
        mut commands: Commands,

        // Guards
        _server: Single<Entity, With<Server>>, // Make observer only run if it is the server
    ) {
        let client = trigger.entity;

        // (SG) I'm not sure what the send interval for replication should be
        // Using the 64 fps tick rate for now
        let send_interval = Duration::from_secs_f64(1.0 / 64.0);

        // Configure the client connection by adding components
        commands.entity(client).insert((
            ReplicationSender::new(
                send_interval,
                SendUpdatesMode::SinceLastAck,
                false,
            ),

            // We need a ReplicationReceiver on the server side because the Action entities are spawned
            // on the client and replicated to the server.
            ReplicationReceiver::default(),
        ));

        commands.trigger(SpawnPlayerEvent { client });
    }


            let client = commands.spawn((
            Client::default(),
            LocalAddr(client_addr),
            PeerAddr(server_addr),
            Link::new(None),
            ReplicationReceiver::default(),
            PredictionManager {
                rollback_policy: RollbackPolicy {
                    state: RollbackMode::Check,
                    input: RollbackMode::Disabled,
                    ..default()
                },
                ..default()
            },
            NetcodeClient::new(auth, netcode_config)?,
            UdpIo::default(),
            // we need to add a ReplicationSender to the client entity to replicate the Action entities to the server
            ReplicationSender::new(
                send_interval,
                SendUpdatesMode::SinceLastAck,
                false,
            ),
            // set some input-delay since we are predicting all entities
            InputTimeline(Timeline::from(
                Input::default().with_input_delay(InputDelayConfig::balanced())
            )),
        )).id();


        fn position_should_rollback(this: &Position, that: &Position) -> bool {
    (this.0 - that.0).length() >= 0.1
}

fn rotation_should_rollback(this: &Rotation, that: &Rotation) -> bool {
    this.angle_between(*that) >= 0.1
}

fn linear_velocity_should_rollback(_this: &LinearVelocity, _that: &LinearVelocity) -> bool {
    false
}

fn angular_velocity_should_rollback(_this: &AngularVelocity, _that: &AngularVelocity) -> bool {
    false
    

}


    app.add_plugins(InputPlugin::<NetworkInputComponent> {
        config: InputConfig {
            rebroadcast_inputs: true,
            lag_compensation: true,
            ..default()
        },
    });

    app.add_plugins(LightyearAvianPlugin {
        replication_mode: AvianReplicationMode::PositionButInterpolateTransform,
        ..default()
    });

    app.register_type::<Actions<NetworkInputComponent>>();
    app.register_type::<ActionOf<NetworkInputComponent>>();

    app.register_input_action::<MoveAction>();
    app.register_input_action::<CameraForwardAction>();
    app.register_input_action::<SprintAction>();
    app.register_input_action::<JumpAction>();


    app.register_component::<Position>()
        .add_prediction()
        .add_should_rollback(position_should_rollback)
        .register_linear_interpolation()
        .add_linear_correction_fn()
        .enable_correction();

    app.register_component::<Rotation>()
        .add_prediction()
        .add_should_rollback(rotation_should_rollback)
        .register_linear_interpolation()
        .add_linear_correction_fn()
        .enable_correction();

    app.register_component::<LinearVelocity>()
        .add_prediction()
        .add_should_rollback(linear_velocity_should_rollback);

    app.register_component::<AngularVelocity>()
        .add_prediction()
        .add_should_rollback(angular_velocity_should_rollback);


    #[debug_fail]
fn accumulate_movement_observer(
    trigger: On<Fire<MoveAction>>,
    mut accumulation_query: Query<&mut InputAccumulationComponent>,
) -> Result {
    if trigger.value == Vec2::ZERO {
        return OK;
    }

    #[fail_msg = "Unable to find input accumulation associated with MoveAction"]
    let mut input = accumulation_query.get_mut(trigger.context)?;

    input.movement = trigger.value;

    OK
}

#[debug_fail]
fn accumulate_camera_observer(
    trigger: On<Fire<CameraForwardAction>>,
    mut accumulation_query: Query<&mut InputAccumulationComponent>,
) -> Result {
    // Always set camera_forward since it's a rotation and not a momentary value
    #[fail_msg = "Unable to find input accumulation associated with CameraForwardAction"]
    let mut input = accumulation_query.get_mut(trigger.context)?;

    input.camera = trigger.value;

    OK
}

#[debug_fail]
fn accumulate_sprint_observer(
    trigger: On<Fire<SprintAction>>,
    mut accumulation_query: Query<&mut InputAccumulationComponent>,
) -> Result {
    if !trigger.value {
        return OK;
    }

    #[fail_msg = "Unable to find input accumulation associated with SprintAction"]
    let mut input = accumulation_query.get_mut(trigger.context)?;

    input.sprint = true;

    OK
}

#[debug_fail]
fn accumulate_jump_observer(
    trigger: On<Fire<JumpAction>>,
    mut accumulation_query: Query<&mut InputAccumulationComponent>,
) -> Result {
    if !trigger.value {
        return OK;
    }

    #[fail_msg = "Unable to find input accumulation associated with JumpAction"]
    let mut input = accumulation_query.get_mut(trigger.context)?;

    input.jump = true;

    OK
}


fn avatar_controller_system(
    mut commands: Commands,
    mut avatar_actions: Query<(Entity, &InputAccumulationComponent, &mut TnuaController)>,
) {
    for (entity, accumulation, mut controller) in avatar_actions.iter_mut() {
        // using infailable match statements so we always run the controller and we don't return
        // early before processing other avatars if one happens to have input failures

        let InputAccumulationComponent {
            movement,
            camera,
            sprint,
            jump,
            ..
        } = *accumulation;

        // queue reset of input accumulation for a clean state before next tick
        commands.entity(entity).try_insert(InputAccumulationComponent {
            // Copy camera_forward over because it should remain on last input
            camera,
            ..default()
        });

        // Align movement direction to camera direction
        let camera_forward = Dir3::new(camera).unwrap_or(Dir3::NEG_Z);
        let mut direction = Transform::default()
            .looking_to(camera_forward, Vec3::Y)
            .transform_point(movement.xxy().with_y(0.0));
        direction *= Vec3::ONE.with_y(0.0);

        // Adjust desired movement speed based on sprint keys
        let mut velocity_magnitude = 7.5;

        if sprint {
            velocity_magnitude = 15.0;
        }

        // Adjust desired movement speed based on super speed debug toggle
        #[cfg(feature = "dev")]
        if accumulation.debug_super_speed {
            velocity_magnitude = 200.0
        }

        controller.basis(TnuaBuiltinWalk {
            desired_velocity: direction.adjust_precision().normalize_or_zero() * velocity_magnitude,
            // TODO: (Brain) I don't think we want to rotate the capsule, just animate rotate on movement
            // desired_forward: camera_component.forward,
            float_height: 0.9,
            ..default()
        });

        // TODO: (Brain) can we move this to an observer?
        if jump {
            controller.action(TnuaBuiltinJump {
                height: 5.0,
                // fall_extra_gravity: 10.0,
                ..default()
            });
        }
    }
}