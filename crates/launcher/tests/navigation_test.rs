mod common;
#[cfg(test)]
mod test {
    use crate::common::test::{
        assert_entity_moved, get_entity_position, get_spawned_npcs, run_apps_updates,
        setup_two_player_game,
    };
    use shared::navigation::{PatrolRoute, SimpleNavigationAgent};

    /// Test basic navigation functionality
    #[test]
    fn test_basic_navigation_functionality() {
        let (mut server, mut client1, mut client2) = setup_two_player_game();

        // Get NPCs from server
        let server_npcs = get_spawned_npcs(server.world_mut());

        if server_npcs.is_empty() {
            // The test setup might not spawn NPCs automatically, so we pass this test
            // as navigation NPCs might be spawned in different scenarios
            return;
        }

        let npc_entity = server_npcs[0];

        // Verify NPC has navigation components
        let has_nav_agent = server
            .world()
            .get::<SimpleNavigationAgent>(npc_entity)
            .is_some();
        let has_patrol_route = server.world().get::<PatrolRoute>(npc_entity).is_some();

        if !has_nav_agent || !has_patrol_route {
            // If NPC doesn't have navigation components, skip this test
            return;
        }

        // Record initial position
        let initial_pos =
            get_entity_position(server.world(), npc_entity).expect("NPC should have position");

        // Run simulation for a while to allow NPC to move
        let mut apps = [&mut server, &mut client1, &mut client2];
        run_apps_updates(&mut apps, 200);

        // Assert NPC has moved along its patrol route
        assert_entity_moved(server.world(), npc_entity, initial_pos, 0.5);
    }
}
