use crate::game_state::GameState;

use bevy::log::debug;
use bevy::prelude::{
    App, AssetServer, Commands, Component, Handle, IntoScheduleConfigs, Local, OnEnter, Plugin,
    Query, Res, Resource, Startup, Time, Update, in_state,
};

use bevy_kira_audio::AudioControl;
use bevy_kira_audio::AudioSource;
use bevy_kira_audio::prelude::{Audio, AudioPlugin};

use avian3d::prelude::LinearVelocity;

#[derive(Component)]
pub struct PlayerController {
    pub is_sprinting: bool,
    pub is_crouching: bool,
}

pub struct GameAudioPlugin;

impl Plugin for GameAudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(AudioPlugin)
            .add_systems(Startup, setup_audio)
            .add_systems(OnEnter(GameState::Playing), cleanup_audio)
            .add_systems(
                Update,
                (play_footsteps, play_ambient_sounds).run_if(in_state(GameState::Playing)),
            );
    }
}

#[derive(Resource)]
pub struct GameAudio {
    pub footstep_concrete: Handle<AudioSource>,
    pub weapon_reload: Handle<AudioSource>,
    pub ambient_hum: Handle<AudioSource>,
    pub weapon_fire: Handle<AudioSource>,
    pub damage_taken: Handle<AudioSource>,
    pub enemy_alert: Handle<AudioSource>,
}

fn setup_audio(mut commands: Commands, asset_server: Res<AssetServer>) {
    let game_audio = GameAudio {
        footstep_concrete: asset_server.load("audio/footstep_concrete.mp3"),
        weapon_fire: asset_server.load("audio/weapon_fire.mp3"),
        weapon_reload: asset_server.load("audio/weapon_reload.mp3"),
        ambient_hum: asset_server.load("audio/ambient_hum.mp3"),
        enemy_alert: asset_server.load("audio/enemy_alert.mp3"),
        damage_taken: asset_server.load("audio/damage_taken.wav"),
    };

    commands.insert_resource(game_audio);

    debug!("Audio system initialized");
}

fn play_footsteps(
    time: Res<Time>,
    audio: Res<GameAudio>,
    kira_audio: Res<Audio>,
    player_query: Query<(&PlayerController, &LinearVelocity)>,
    mut last_footstep_time: Local<f32>,
) {
    let current_time = time.elapsed_secs();

    for (player_controller, velocity) in player_query.iter() {
        let speed = velocity.length();

        if speed > 0.5 {
            let footstep_volume = if player_controller.is_sprinting {
                0.8
            } else if player_controller.is_crouching {
                0.2
            } else {
                0.5
            };

            let footstep_interval = if player_controller.is_sprinting {
                0.3
            } else if player_controller.is_crouching {
                0.8
            } else {
                0.5
            };

            if speed > 1.0 && current_time - *last_footstep_time > footstep_interval {
                kira_audio
                    .play(audio.footstep_concrete.clone())
                    .with_volume(footstep_volume);

                *last_footstep_time = current_time;
            }
        }
    }
}

fn play_ambient_sounds(
    time: Res<Time>,
    audio: Res<GameAudio>,
    kira_audio: Res<Audio>,
    mut last_ambient_time: Local<f32>,
    mut ambient_playing: Local<bool>,
) {
    let current_time = time.elapsed_secs();

    if current_time - *last_ambient_time > 30.0 && !*ambient_playing {
        kira_audio.play(audio.ambient_hum.clone()).with_volume(0.1);

        *last_ambient_time = current_time;
        *ambient_playing = true;
    } else if current_time - *last_ambient_time > 35.0 {
        *ambient_playing = false;
    }
}

fn cleanup_audio(kira_audio: Res<Audio>) {
    debug!("Stopping all audio to prevent stacking");
    kira_audio.stop();
}
