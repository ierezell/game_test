//! Client-side flashlight integration tests

use bevy::prelude::*;
use shared::components::flashlight::*;
use shared::input::PlayerAction;

#[test]
fn test_flashlight_toggle_action() {
    // Verify F key action is properly defined
    let action = PlayerAction::ToggleFlashlight;
    assert_eq!(
        format!("{:?}", action),
        "ToggleFlashlight",
        "ToggleFlashlight action should exist"
    );
}

#[test]
fn test_flashlight_beam_spawning() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    
    // Spawn player with flashlight
    let player = app.world_mut().spawn((
        PlayerFlashlight {
            is_on: true,
            intensity: 100_000.0,
            range: 50.0,
            inner_angle: 20.0,
            outer_angle: 40.0,
        },
        Transform::default(),
    )).id();
    
    app.update();
    
    // Verify flashlight component exists
    let flashlight = app.world().get::<PlayerFlashlight>(player);
    assert!(flashlight.is_some(), "PlayerFlashlight component should exist");
    assert!(flashlight.unwrap().is_on, "Flashlight should be on");
}

#[test]
fn test_flashlight_beam_marker() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    
    // Spawn flashlight beam entity
    app.world_mut().spawn((
        FlashlightBeam,
        Transform::default(),
    ));
    
    app.update();
    
    // Verify beam marker exists
    let mut query = app.world_mut().query::<&FlashlightBeam>();
    let beam_count = query.iter(app.world()).count();
    
    assert_eq!(beam_count, 1, "Should have one flashlight beam");
}

#[test]
fn test_flashlight_state_persistence() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    
    // Spawn flashlight and toggle it
    let entity = app.world_mut().spawn(PlayerFlashlight::default()).id();
    
    app.update();
    
    // Toggle on
    {
        let mut flashlight = app.world_mut().get_mut::<PlayerFlashlight>(entity).unwrap();
        flashlight.toggle();
    }
    
    app.update();
    
    // Verify state persisted
    let flashlight = app.world().get::<PlayerFlashlight>(entity).unwrap();
    assert!(flashlight.is_on, "Flashlight state should persist after toggle");
}
