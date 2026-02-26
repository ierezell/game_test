use bevy::prelude::{
    AlignItems, App, Commands, Component, FlexDirection, IntoScheduleConfigs, JustifyContent, Name,
    Node, OnEnter, OnExit, Plugin, PositionType, Query, Res, Text, TextFont, Update, Val, With,
    in_state,
};
use lightyear::prelude::{Controlled, Predicted};
use shared::components::weapons::Gun;
use shared::protocol::PlayerId;

use crate::{ClientGameState, Headless};

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
    local_player_gun_query: Query<&Gun, (With<PlayerId>, With<Predicted>, With<Controlled>)>,
) {
    let Ok(mut text) = ammo_text_query.single_mut() else {
        return;
    };

    if let Ok(gun) = local_player_gun_query.single() {
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
