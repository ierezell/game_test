//! Tests for flashlight component functionality

use bevy::prelude::*;
use crate::components::flashlight::*;

#[test]
fn test_flashlight_default_state() {
    let flashlight = PlayerFlashlight::default();
    assert!(!flashlight.is_on, "Flashlight should default to off");
    // Default trait uses 0.0 for all fields, new() uses specific values
    assert_eq!(flashlight.intensity, 0.0, "Default intensity is 0.0");
    assert_eq!(flashlight.range, 0.0, "Default range is 0.0");
}

#[test]
fn test_flashlight_new_state() {
    let flashlight = PlayerFlashlight::new();
    assert!(flashlight.is_on, "Flashlight should start ON");
    assert_eq!(flashlight.intensity, 1000000.0, "New intensity mismatch");
    assert_eq!(flashlight.range, 80.0, "New range mismatch");
}

#[test]
fn test_flashlight_toggle() {
    let mut flashlight = PlayerFlashlight::default();
    assert!(!flashlight.is_on, "Initial state should be off");
    
    flashlight.toggle();
    assert!(flashlight.is_on, "Flashlight should be on after toggle");
    
    flashlight.toggle();
    assert!(!flashlight.is_on, "Flashlight should be off after second toggle");
}

#[test]
fn test_flashlight_beam_marker() {
    // FlashlightBeam is a marker component - just verify it can be created
    let _beam = FlashlightBeam;
}

#[test]
fn test_flashlight_component_creation() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    
    // Spawn entity with flashlight
    app.world_mut().spawn(PlayerFlashlight {
        is_on: true,
        intensity: 50_000.0,
        range: 30.0,
        inner_angle: 15.0,
        outer_angle: 30.0,
    });
    
    app.update();
    
    let mut query = app.world_mut().query::<&PlayerFlashlight>();
    let flashlight = query.single(app.world()).expect("Should have PlayerFlashlight");
    
    assert!(flashlight.is_on, "Flashlight should be on");
    assert_eq!(flashlight.intensity, 50_000.0, "Intensity mismatch");
}

#[test]
fn test_flashlight_clone() {
    let original = PlayerFlashlight {
        is_on: true,
        intensity: 75_000.0,
        range: 40.0,
        inner_angle: 20.0,
        outer_angle: 35.0,
    };
    
    let cloned = original;
    assert_eq!(original, cloned, "Clone should equal original");
}
