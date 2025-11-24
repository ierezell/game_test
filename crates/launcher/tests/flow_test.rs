mod common;
use bevy::prelude::*;
use client::ClientGameState;
use common::{create_test_client, create_test_server, setup_two_player_game, get_spawned_players};
use server::ServerGameState;
use lightyear::prelude::{MessageSender, MetadataChannel};
use shared::protocol::HostStartGameEvent;

#[test]
fn test_server_stability() {
    let mut server_app = create_test_server();
    

    for _ in 0..100 {
        server_app.update();
    }
    let final_state = server_app.world().resource::<State<ServerGameState>>();
    assert_eq!(
        *final_state.get(),
        ServerGameState::Lobby,
        "Server should remain in Lobby state"
    );
}

#[test]
fn test_client_stability() {
    let mut client_app = create_test_client(1, false, false, false);
    for _ in 0..100 {
        client_app.update();
    }

    let final_state = client_app.world().resource::<State<ClientGameState>>();
    assert!(*final_state.get() == ClientGameState::LocalMenu);
}

#[test]
fn test_basic_app_creation() {
    let mut server_app = create_test_server(); 
    let mut client_app = create_test_client(1, false, false, false);

    let server_state = server_app.world().resource::<State<ServerGameState>>();
    assert_eq!(
        *server_state.get(),
        ServerGameState::Lobby,
        "Server should start in Lobby state"
    );

    let client_state = client_app.world().resource::<State<ClientGameState>>();
    assert_eq!(
        *client_state.get(),
        ClientGameState::LocalMenu,
        "Client should start in LocalMenu state"
    );

    for _ in 0..5 {
        server_app.update();
        client_app.update();
    }
}


#[test]
fn test_multiple_client_creation() {
    let mut server_app = create_test_server();
    let mut client1 = create_test_client(1, false, false, false);
    let mut client2 = create_test_client(1, false, false, false);
    let mut client3 = create_test_client(1, false, false, false);

    for _ in 0..20 {
        server_app.update();
        client1.update();
        client2.update();
        client3.update();
    }

    assert!(
        server_app
            .world()
            .resource::<State<ServerGameState>>()
            .get()
            == &ServerGameState::Lobby
    );

    assert!(
        client1.world().resource::<State<ClientGameState>>().get() == &ClientGameState::LocalMenu
    );

    assert!(
        client2.world().resource::<State<ClientGameState>>().get() == &ClientGameState::LocalMenu
    );

    assert!(
        client3.world().resource::<State<ClientGameState>>().get() == &ClientGameState::LocalMenu
    );
}



#[test]
fn test_one_client_join() {
    let mut server_app = create_test_server(); 

     for _ in 0..100 {
        server_app.update();
     }

    let mut client_app = create_test_client(1, false, false, true);
      for _ in 0..100 {
        client_app.update();
        server_app.update();
     }

    let server_state = server_app.world().resource::<State<ServerGameState>>();
    assert_eq!(
        *server_state.get(),
        ServerGameState::Lobby,
        "Server should start in Lobby state"
    );

    let client_state = client_app.world().resource::<State<ClientGameState>>();
    assert_eq!(
        *client_state.get(),
        ClientGameState::Lobby,
        "Client should start in Lobby state"
    );

    for _ in 0..50 {
        server_app.update();
        client_app.update();
    }
}


#[test]
fn test_multiple_client_join() {
    let mut server_app = create_test_server();
    for _ in 0..20 {
        server_app.update();
    }

    let mut client1 = create_test_client(1, false, false, true);
    let mut client2 = create_test_client(1, false, false, true);
    let mut client3 = create_test_client(1, false, false, true);
    
    for _ in 0..100 {
        server_app.update();
        client1.update();
        client2.update();
        client3.update();
    }

    assert!(
        server_app
            .world()
            .resource::<State<ServerGameState>>()
            .get()
            == &ServerGameState::Lobby
    );

    assert!(
        client1.world().resource::<State<ClientGameState>>().get() == &ClientGameState::Lobby
    );

    assert!(
        client2.world().resource::<State<ClientGameState>>().get() == &ClientGameState::Lobby
    );

    assert!(
        client3.world().resource::<State<ClientGameState>>().get() == &ClientGameState::Lobby
    );
}


#[test]
fn test_one_client_start() {
    let mut server_app = create_test_server(); 

     for _ in 0..100 {
        server_app.update();
     }

    let mut client_app = create_test_client(1, false, false, true);
    for _ in 0..100 {
        client_app.update();
        server_app.update();
     }

    let server_state = server_app.world().resource::<State<ServerGameState>>();
    assert_eq!(
        *server_state.get(),
        ServerGameState::Lobby,
        "Server should start in Lobby state"
    );

    let client_state = client_app.world().resource::<State<ClientGameState>>();
    assert_eq!(
        *client_state.get(),
        ClientGameState::Lobby,
        "Client should start in Lobby state"
    );

    for _ in 0..50 {
        server_app.update();
        client_app.update();
    }

    let mut sender_query = client_app.world_mut().query::<&mut MessageSender<HostStartGameEvent>>();
    if let Ok(mut sender) = sender_query.single_mut(client_app.world_mut()) {
        let _ = sender.send::<MetadataChannel>(HostStartGameEvent);
    }
    
    for _ in 0..500 {
        server_app.update();
        client_app.update();
    }

    let server_state = server_app.world().resource::<State<ServerGameState>>();
    assert_eq!(
        *server_state.get(),
        ServerGameState::Playing,
        "Server should start in Lobby state"
    );

    let client_state = client_app.world().resource::<State<ClientGameState>>();
    assert_eq!(
        *client_state.get(),
        ClientGameState::Playing,
        "Client should start in Lobby state"
    );
}



#[test]
fn test_multiple_client_start() {
    let mut server_app = create_test_server();
    for _ in 0..20 {
        server_app.update();
    }

    let mut client1 = create_test_client(1, false, false, true);
    let mut client2 = create_test_client(1, false, false, true);
    let mut client3 = create_test_client(1, false, false, true);
    
    for _ in 0..100 {
        server_app.update();
        client1.update();
        client2.update();
        client3.update();
    }

    assert!(
        server_app
            .world()
            .resource::<State<ServerGameState>>()
            .get()
            == &ServerGameState::Lobby
    );

    assert!(
        client1.world().resource::<State<ClientGameState>>().get() == &ClientGameState::Lobby
    );

    assert!(
        client2.world().resource::<State<ClientGameState>>().get() == &ClientGameState::Lobby
    );

    assert!(
        client3.world().resource::<State<ClientGameState>>().get() == &ClientGameState::Lobby
    );

    let mut sender_query = client1.world_mut().query::<&mut MessageSender<HostStartGameEvent>>();
    if let Ok(mut sender) = sender_query.single_mut(client1.world_mut()) {
        let _ = sender.send::<MetadataChannel>(HostStartGameEvent);
    }
    
    for _ in 0..500 {
        server_app.update();
        client1.update();
        client2.update();
        client3.update();
    }

    let server_state = server_app.world().resource::<State<ServerGameState>>();
    assert_eq!(
        *server_state.get(),
        ServerGameState::Playing,
        "Server should start in Lobby state"
    );

    let client_state_1 = client1.world().resource::<State<ClientGameState>>();
    assert_eq!(
        *client_state_1.get(),
        ClientGameState::Playing,
        "Client should start in Lobby state"
    );

    let client_state_2 = client2.world().resource::<State<ClientGameState>>();
    assert_eq!(
        *client_state_2.get(),
        ClientGameState::Playing,
        "Client should start in Lobby state"
    );
    let client_state_3 = client3.world().resource::<State<ClientGameState>>();
    assert_eq!(
        *client_state_3.get(),
        ClientGameState::Playing,
        "Client should start in Lobby state"
    );
}


#[test]
fn test_spawned_entities() {
    let (mut server, mut client1, mut client2) = setup_two_player_game();
    let client_1_players = get_spawned_players(client1.world_mut());
    let client_2_players = get_spawned_players(client2.world_mut());
    let server_players = get_spawned_players(server.world_mut());

    assert_eq!(client_1_players.len(), 2, "Client 1 should see exactly 2 players");
    assert_eq!(client_2_players.len(), 2, "Client 2 should see exactly 2 players");
    assert_eq!(server_players.len(), 2, "Server should have exactly 2 players");

    // Assert controlled player is one per client 
    // Assert 1 predicted player per client and one interpolated (other player) per client 
}