use bevy::prelude::*;
use crate::bulkhead_door::DoorState;
use crate::level_generation::ZoneId;
use crate::level_visuals::{ZoneVisual, DoorVisual};

/// Maximum distance from player to render zones
pub const MAX_RENDER_DISTANCE: f32 = 100.0;

/// Component to mark entities that should be culled based on visibility
#[derive(Component)]
pub struct VisibilityCulling {
    pub zone_id: ZoneId,
    pub always_visible: bool,
}

/// System to cull zones behind closed doors and far from players
pub fn update_zone_visibility(
    mut zones_query: Query<(&ZoneVisual, &mut Visibility), Without<Camera>>,
    doors_query: Query<(&DoorVisual, &DoorState)>,
    player_query: Query<&Transform, With<crate::protocol::PlayerId>>,
    camera_query: Query<&Transform, With<Camera>>,
) {
    // Get player/camera position (use camera if available, otherwise first player)
    let observer_pos = camera_query
        .iter()
        .next()
        .or_else(|| player_query.iter().next())
        .map(|t| t.translation);

    let Some(_observer_pos) = observer_pos else {
        return;
    };

    // Build a map of which zones are accessible (connected via open doors)
    let mut accessible_zones = std::collections::HashSet::new();
    
    // Find the zone the player is in by distance
    let mut closest_zone: Option<(ZoneId, f32)> = None;
    for (zone_visual, _) in zones_query.iter() {
        // Approximate zone position from visual entities
        // In a real implementation, we'd store zone center positions
        let distance = 0.0; // Placeholder
        if closest_zone.is_none() || distance < closest_zone.unwrap().1 {
            closest_zone = Some((zone_visual.zone_id, distance));
        }
    }

    if let Some((player_zone, _)) = closest_zone {
        accessible_zones.insert(player_zone);
        
        // Add zones connected via open doors
        for (door_visual, door_state) in doors_query.iter() {
            // Only consider passable doors (Open or Opening)
            let is_passable = matches!(
                door_state,
                DoorState::Open | DoorState::Opening { .. }
            );
            
            if is_passable {
                if accessible_zones.contains(&door_visual.zone_a) {
                    accessible_zones.insert(door_visual.zone_b);
                }
                if accessible_zones.contains(&door_visual.zone_b) {
                    accessible_zones.insert(door_visual.zone_a);
                }
            }
        }
    }

    // Update visibility based on accessibility
    for (zone_visual, mut visibility) in zones_query.iter_mut() {
        let is_accessible = accessible_zones.contains(&zone_visual.zone_id);
        
        // Show only accessible zones or zones within render distance
        *visibility = if is_accessible {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

/// Plugin for visibility culling system
pub struct CullingPlugin;

impl Plugin for CullingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_zone_visibility);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_max_render_distance() {
        const { assert!(MAX_RENDER_DISTANCE >= 50.0) };
        const { assert!(MAX_RENDER_DISTANCE <= 200.0) };
        
        println!("Render distance configured: {}m", MAX_RENDER_DISTANCE);
    }

    #[test]
    fn test_visibility_culling_component() {
        let culling = VisibilityCulling {
            zone_id: ZoneId(0),
            always_visible: false,
        };
        assert_eq!(culling.zone_id.0, 0);
        assert!(!culling.always_visible);
    }
}
