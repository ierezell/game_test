#[cfg(test)]
mod test {
    use avian3d::prelude::*;
    use bevy::prelude::*;
    use leafwing_input_manager::prelude::*;
    use lightyear::prelude::PeerId;
    use server::input::server_player_movement;
    use shared::input::{FpsController, PlayerAction};
    use shared::protocol::PlayerId;

    #[test]
    fn test_server_player_movement_logic() {
        // 1. Setup App with necessary plugins/resources
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        // app.insert_resource(Time::default()); // MinimalPlugins adds Time
        app.insert_resource(SpatialQueryPipeline::default());

        // 2. Spawn a player entity
        let start_pos = Vec3::ZERO;
        let player_id = PlayerId(PeerId::Netcode(1));

        // Enable input
        let mut action_state = ActionState::<PlayerAction>::default();
        // W key is mapped to Y axis. AxisPair(0.0, 1.0)
        action_state.set_axis_pair(&PlayerAction::Move, Vec2::new(0.0, 1.0)); // Move forward
        // Do NOT call press() on DualAxis!

        // Setup controller with Yaw to test directionality
        // Yaw = -90 degrees (facing Right/+X)
        // Forward input should result in +X velocity
        let mut controller = FpsController {
            yaw: -std::f32::consts::FRAC_PI_2,
            air_acceleration: 1000.0,
            air_speed_cap: 100.0,
            ..Default::default()
        };

        let player_entity = app
            .world_mut()
            .spawn((
                player_id,
                action_state,
                controller,
                Transform::from_translation(start_pos),
                LinearVelocity::default(),
                Collider::capsule(1.0, 0.5),
            ))
            .id();

        // 3. Run the system directly
        let mut schedule = Schedule::default();
        schedule.add_systems(server_player_movement);

        // Run once
        // Advance time to ensure delta is non-zero
        if let Some(mut time) = app.world_mut().get_resource_mut::<Time>() {
            time.advance_by(std::time::Duration::from_secs_f32(1.0 / 60.0));
        }

        schedule.run(app.world_mut());

        // 4. Verify Movement
        let mut query = app.world_mut().query::<(&LinearVelocity, &Transform)>();
        let (velocity, _transform) = query
            .get(app.world(), player_entity)
            .expect("Player entity not found");

        println!("Velocity: {:?}", velocity);

        // Check if velocity is non-zero (indicating input was processed)
        assert!(
            velocity.length() > 0.0,
            "Player should have velocity after processing input. Velocity: {:?}",
            velocity
        );

        // Verify direction (+X)
        // Note: 100 * 20 * 0.016 is approx 32.0.
        // It should be positive X.
        assert!(
            velocity.0.x > 0.1,
            "Should move validly in +X direction. Velocity: {:?}",
            velocity
        );
        assert!(
            velocity.0.z.abs() < 0.1,
            "Should NOT move in Z direction. Velocity: {:?}",
            velocity
        );

        println!("Test Passed: server_player_movement correctly processed input and direction.");
    }
}
