mod common;
#[cfg(test)]
mod test {
    use crate::common::test::create_test_client;
    use avian3d::prelude::*;
    use bevy::prelude::*;
    use shared::entities::PlayerPhysicsBundle;
    use shared::protocol::{CharacterMarker, PlayerColor, PlayerId};

    #[test]
    fn test_player_movement() {
        // Create a test client app
        let mut app = create_test_client(1, false, false, false);

        // Manually spawn a player entity for testing movement
        let player_entity = app
            .world_mut()
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
        for _ in 0..10 {
            app.update();
        }

        // Record initial position
        let initial_pos = app
            .world()
            .get::<Transform>(player_entity)
            .expect("Player should have transform")
            .translation;

        // Apply movement by modifying velocity directly (simulating input processing)
        if let Some(mut velocity) = app.world_mut().get_mut::<LinearVelocity>(player_entity) {
            velocity.0 = Vec3::new(5.0, 0.0, 0.0); // Move in X direction
        }

        // Run physics updates to process movement
        for _ in 0..60 {
            // Run for 1 second at 60fps
            app.update();
        }

        // Get final position
        let final_pos = app
            .world()
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
        assert!(
            app.world().get::<PlayerId>(player_entity).is_some(),
            "PlayerId component should exist"
        );
        assert!(
            app.world().get::<Transform>(player_entity).is_some(),
            "Transform component should exist"
        );
        assert!(
            app.world().get::<LinearVelocity>(player_entity).is_some(),
            "LinearVelocity component should exist"
        );
    }
}
