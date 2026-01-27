#[cfg(test)]
mod test {
    // TODO: This test uses ClientServerStepper which is not implemented
    // The new tests in movement_replication_test.rs cover the same scenarios
    /*
    use crate::tests::common::test::create_test_client;
    use avian3d::prelude::*;
    use bevy::prelude::*;
    use shared::entities::PlayerPhysicsBundle;
    use shared::protocol::{CharacterMarker, PlayerColor, PlayerId};

    #[test]
    fn test_player_movement() {
        // Create Stepper with 1 client
        let mut stepper = ClientServerStepper::new(1, true);
        stepper.init();

        // Access client world manually to spawn a test entity purely for physics test
        // NOTE: The stepper creates a "connected" client.
        // If we want to test pure local movement without server replication interfering,
        // we can still spawn a local entity.
        // However, the stepper's client app has plugins that might predict/rollback.
        // For this basic test, we just want to see if setting velocity moves the thing.

        let client_idx = 0;
        let client_world = stepper.client_world_mut(client_idx);

        // Manually spawn a player entity for testing movement
        let player_entity = client_world
            .spawn((
                Name::new("TestPlayer"),
                Transform::from_xyz(0.0, 0.0, 0.0),
                GlobalTransform::default(),
                Visibility::default(),
                InheritedVisibility::default(),
                ViewVisibility::default(),
                PlayerId(lightyear::prelude::PeerId::Netcode(1)),
                PlayerColor(Color::srgb(1.0, 0.0, 0.0)),
                CharacterMarker,
                PlayerPhysicsBundle::default(),
                LinearVelocity::ZERO,
                AngularVelocity::ZERO,
            ))
            .id();

        // Run initial updates to initialize physics
        stepper.loop_step(10);

        // Record initial position
        let initial_pos = stepper
            .client_world(client_idx)
            .get::<Transform>(player_entity)
            .expect("Player should have transform")
            .translation;

        // Apply movement by modifying velocity directly (simulating input processing)
        if let Some(mut velocity) = stepper
            .client_world_mut(client_idx)
            .get_mut::<LinearVelocity>(player_entity)
        {
            velocity.0 = Vec3::new(5.0, 0.0, 0.0); // Move in X direction
        }

        // Run physics updates to process movement
        stepper.loop_step(60);

        // Get final position
        let final_pos = stepper
            .client_world(client_idx)
            .get::<Transform>(player_entity)
            .expect("Player should have transform")
            .translation;

        // Assert that the player moved (physics simulation should have processed the velocity)
        let movement_delta = final_pos - initial_pos;

        println!("Initial position: {:?}", initial_pos);
        println!("Final position: {:?}", final_pos);
        println!("Movement delta: {:?}", movement_delta);

        // Check that there was movement in the X direction
        // We don't expect exact values because physics simulation with time steps
        // will have accumulated some movement over 60 frames
        assert!(
            movement_delta.x > 0.1,
            "Player should have moved in X direction, got delta: {:?}",
            movement_delta
        );

        // Verify the test system is working by checking entity exists
        let world = stepper.client_world(client_idx);
        assert!(
            world.get::<PlayerId>(player_entity).is_some(),
            "PlayerId component should exist"
        );
        assert!(
            world.get::<Transform>(player_entity).is_some(),
            "Transform component should exist"
        );
        assert!(
            world.get::<LinearVelocity>(player_entity).is_some(),
            "LinearVelocity component should exist"
        );
    }
    */
}
