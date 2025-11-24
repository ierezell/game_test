mod common;
use bevy::prelude::*;
use common::setup_two_player_game;
use shared::protocol::PlayerId;

#[test]
fn test_spawned_entities() {
    let (mut server, mut client1, mut client2) = setup_two_player_game();

    let client_1_mesh: Vec<Entity> = client1
        .world_mut()
        .query::<(Entity, &PlayerId, &Mesh3d)>()
        .iter(client1.world())
        .map(|(entity, _, _)| entity)
        .collect();
    let client_2_mesh: Vec<Entity> = client2
        .world_mut()
        .query::<(Entity, &PlayerId, &Mesh3d)>()
        .iter(client2.world())
        .map(|(entity, _, _)| entity)
        .collect();
    let server_mesh: Vec<Entity> = server
        .world_mut()
        .query::<(Entity, &PlayerId, &Mesh3d)>()
        .iter(server.world())
        .map(|(entity, _, _)| entity)
        .collect();

    // Assert all players have a mesh (so they're visible to others.)
    assert!(
        client_1_mesh.len() >= 1,
        "Client 1 should have at least one mesh for players"
    );
    assert!(
        client_2_mesh.len() >= 1,
        "Client 2 should have at least one mesh for players"
    );
    assert!(
        server_mesh.is_empty(),
        "Server should not render meshes (headless)"
    );
}
