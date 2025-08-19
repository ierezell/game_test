use crate::menu::GameState;
use crate::events::WeaponFireEvent;
use bevy::prelude::*;
use bevy_kira_audio::AudioSource as KiraAudioSource;
use bevy_kira_audio::prelude::*;



pub struct GameAudioPlugin;

impl Plugin for GameAudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(AudioPlugin)
            .add_systems(Startup, setup_audio)
            .add_systems(OnEnter(GameState::InGame), cleanup_audio)
            .add_systems(
                Update,
                (
                    play_footsteps,
                    play_weapon_reload_sounds,
                    play_ambient_sounds,
                    play_weapon_fire_sounds,
                    play_damage_sounds,
                    play_enemy_detection_sounds,
                )
                    .run_if(in_state(GameState::InGame)),
            );
    }
}

#[derive(Resource)]
pub struct GameAudio {
    pub footstep_concrete: Handle<KiraAudioSource>,
    pub weapon_fire: Handle<KiraAudioSource>,
    pub weapon_reload: Handle<KiraAudioSource>,
    pub ambient_hum: Handle<KiraAudioSource>,
    pub enemy_alert: Handle<KiraAudioSource>,
    pub damage_taken: Handle<KiraAudioSource>,
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
    kira_audio: Res<bevy_kira_audio::Audio>,
    player_query: Query<(
        &crate::player::PlayerController,
        &avian3d::prelude::LinearVelocity,
    )>,
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

fn play_weapon_reload_sounds(
    audio: Res<GameAudio>,
    kira_audio: Res<bevy_kira_audio::Audio>,
    weapon_query: Query<(Entity, &crate::combat::Weapon), Changed<crate::combat::Weapon>>,
    mut last_reload_sounds: Local<std::collections::HashMap<bevy::prelude::Entity, f32>>,
    time: Res<Time>,
) {
    let current_time = time.elapsed_secs();

    for (entity, weapon) in weapon_query.iter() {
        if weapon.is_reloading {
            let last_reload_time = last_reload_sounds.get(&entity).copied().unwrap_or(0.0);

            if current_time - last_reload_time > 0.5 {
                kira_audio
                    .play(audio.weapon_reload.clone())
                    .with_volume(0.6);

                last_reload_sounds.insert(entity, current_time);
            }
        }
    }
}

fn play_ambient_sounds(
    time: Res<Time>,
    audio: Res<GameAudio>,
    kira_audio: Res<bevy_kira_audio::Audio>,
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

fn play_weapon_fire_sounds(
    mut weapon_fire_events: EventReader<WeaponFireEvent>,
    audio: Res<GameAudio>,
    kira_audio: Res<bevy_kira_audio::Audio>,
) {
    for _event in weapon_fire_events.read() {
        kira_audio.play(audio.weapon_fire.clone()).with_volume(0.7);
    }
}

fn cleanup_audio(kira_audio: Res<bevy_kira_audio::Audio>) {
    debug!("Stopping all audio to prevent stacking");
    kira_audio.stop();
}

fn play_damage_sounds(
    audio: Res<GameAudio>,
    kira_audio: Res<bevy_kira_audio::Audio>,
    time: Res<Time>,
    health_query: Query<
        (Entity, &crate::combat::Health),
        (
            Changed<crate::combat::Health>,
            Or<(With<crate::player::Player>, With<crate::enemies::Enemy>)>,
        ),
    >,
    player_query: Query<(), With<crate::player::Player>>,
    enemy_query: Query<(), With<crate::enemies::Enemy>>,
    mut last_damage_times: Local<std::collections::HashMap<bevy::prelude::Entity, f32>>,
) {
    let current_time = time.elapsed_secs();

    for (entity, health) in health_query.iter() {
        let last_damage_time = last_damage_times.get(&entity).copied().unwrap_or(0.0);

        // Check if this entity recently took damage (within the last 0.1 seconds to avoid spam)
        if health.last_damage_time > last_damage_time
            && current_time - health.last_damage_time < 0.1
        {
            if player_query.get(entity).is_ok() {
                // Player took damage
                debug!("Playing player damage sound");
                kira_audio.play(audio.damage_taken.clone()).with_volume(0.7);
            } else if enemy_query.get(entity).is_ok() {
                // Enemy took damage
                debug!("Playing enemy damage sound");
                kira_audio.play(audio.damage_taken.clone()).with_volume(0.4);
            }

            last_damage_times.insert(entity, health.last_damage_time);
        }
    }
}

fn play_enemy_detection_sounds(
    audio: Res<GameAudio>,
    kira_audio: Res<bevy_kira_audio::Audio>,
    enemy_query: Query<
        (Entity, &crate::enemies::Enemy, &Transform),
        (Changed<crate::enemies::Enemy>, With<crate::enemies::Enemy>),
    >,
    mut last_alert_times: Local<std::collections::HashMap<bevy::prelude::Entity, f32>>,
    time: Res<Time>,
) {
    let current_time = time.elapsed_secs();

    for (entity, enemy, _transform) in enemy_query.iter() {
        let last_alert_time = last_alert_times.get(&entity).copied().unwrap_or(0.0);

        if enemy.state == crate::enemies::EnemyState::Chasing
            && current_time - last_alert_time > 3.0
        {
            debug!("Enemy detected player! Playing alert sound");
            kira_audio.play(audio.enemy_alert.clone()).with_volume(0.6);
            last_alert_times.insert(entity, current_time);
        }
    }
}
