use bevy::prelude::{
    AlignItems, App, Commands, Component, FlexDirection, IntoScheduleConfigs, JustifyContent, Name,
    Node, OnEnter, OnExit, Plugin, PositionType, Query, Res, Text, TextFont, Update, Val, With,
    in_state,
};
use shared::components::weapons::Gun;
use shared::protocol::PlayerId;

use crate::{ClientGameState, Headless, LocalPlayerId};

pub struct ClientHudPlugin;

impl Plugin for ClientHudPlugin {
    fn build(&self, app: &mut App) {
        fn is_not_headless(headless: Option<Res<Headless>>) -> bool {
            !headless.map(|h| h.0).unwrap_or(false)
        }

        app.add_systems(
            OnEnter(ClientGameState::Playing),
            spawn_hud.run_if(is_not_headless),
        );
        app.add_systems(
            OnExit(ClientGameState::Playing),
            despawn_hud.run_if(is_not_headless),
        );
        app.add_systems(
            Update,
            update_ammo_text
                .run_if(in_state(ClientGameState::Playing))
                .run_if(is_not_headless),
        );
    }
}

#[derive(Component)]
struct HudRoot;

#[derive(Component)]
struct AmmoText;

fn spawn_hud(mut commands: Commands) {
    commands
        .spawn((
            Name::new("GameHud"),
            HudRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                ..Default::default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                Name::new("Crosshair"),
                Text::new("+"),
                TextFont {
                    font_size: 32.0,
                    ..Default::default()
                },
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Percent(50.0),
                    top: Val::Percent(50.0),
                    ..Default::default()
                },
            ));

            parent
                .spawn((
                    Name::new("AmmoPanel"),
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        align_items: AlignItems::End,
                        justify_content: JustifyContent::End,
                        flex_direction: FlexDirection::Column,
                        ..Default::default()
                    },
                ))
                .with_children(|panel| {
                    panel.spawn((
                        Name::new("AmmoText"),
                        AmmoText,
                        Text::new("Ammo: -- / --"),
                        TextFont {
                            font_size: 22.0,
                            ..Default::default()
                        },
                        Node {
                            position_type: PositionType::Absolute,
                            right: Val::Px(24.0),
                            bottom: Val::Px(24.0),
                            ..Default::default()
                        },
                    ));
                });
        });
}

fn update_ammo_text(
    mut ammo_text_query: Query<&mut Text, With<AmmoText>>,
    local_player_id: Res<LocalPlayerId>,
    player_gun_query: Query<(&PlayerId, &Gun), With<PlayerId>>,
) {
    let Ok(mut text) = ammo_text_query.single_mut() else {
        return;
    };

    let local_gun = player_gun_query.iter().find_map(|(player_id, gun)| {
        if player_id.0.to_bits() == local_player_id.0 {
            Some(gun)
        } else {
            None
        }
    });

    if let Some(gun) = local_gun {
        let status = if gun.is_reloading {
            " (Reloading...)"
        } else {
            ""
        };
        **text = format!(
            "Ammo: {} / {}{}",
            gun.ammo_in_magazine, gun.magazine_size, status
        );
    } else {
        **text = "Ammo: -- / --".to_string();
    }
}

fn despawn_hud(mut commands: Commands, hud_query: Query<bevy::prelude::Entity, With<HudRoot>>) {
    for hud in &hud_query {
        commands.entity(hud).despawn();
    }
}

#[cfg(test)]
mod tests {
    use super::{AmmoText, update_ammo_text};
    use crate::LocalPlayerId;
    use bevy::prelude::{App, MinimalPlugins, Text, Update, With};
    use lightyear::prelude::PeerId;
    use shared::components::weapons::Gun;
    use shared::protocol::PlayerId;

    #[test]
    fn ammo_text_uses_local_player_gun_values() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(LocalPlayerId(1));
        app.add_systems(Update, update_ammo_text);

        app.world_mut().spawn((AmmoText, Text::new("Ammo: -- / --")));

        app.world_mut().spawn((
            PlayerId(PeerId::Netcode(1)),
            Gun {
                ammo_in_magazine: 5,
                magazine_size: 8,
                ..Gun::default()
            },
        ));
        app.world_mut().spawn((
            PlayerId(PeerId::Netcode(2)),
            Gun {
                ammo_in_magazine: 1,
                magazine_size: 8,
                ..Gun::default()
            },
        ));

        app.update();

        let text = app
            .world_mut()
            .query_filtered::<&Text, With<AmmoText>>()
            .single(app.world())
            .expect("Ammo text entity should exist");

        assert_eq!(text.as_str(), "Ammo: 5 / 8");
    }

    #[test]
    fn ammo_text_stays_placeholder_without_local_player_gun() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(LocalPlayerId(1));
        app.add_systems(Update, update_ammo_text);

        app.world_mut().spawn((AmmoText, Text::new("Ammo: -- / --")));
        app.world_mut().spawn((
            PlayerId(PeerId::Netcode(2)),
            Gun {
                ammo_in_magazine: 3,
                magazine_size: 8,
                ..Gun::default()
            },
        ));

        app.update();

        let text = app
            .world_mut()
            .query_filtered::<&Text, With<AmmoText>>()
            .single(app.world())
            .expect("Ammo text entity should exist");

        assert_eq!(text.as_str(), "Ammo: -- / --");
    }
}
