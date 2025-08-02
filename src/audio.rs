use bevy::prelude::*;
use bevy_kira_audio::AudioSource as KiraAudioSource;
use bevy_kira_audio::prelude::*;
use crate::game_state::GameState;

pub struct GameAudioPlugin;

impl Plugin for GameAudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(AudioPlugin)
            .add_systems(Startup, setup_audio)
            .add_systems(
                Update,
                (play_footsteps, play_weapon_sounds, play_ambient_sounds)
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

#[derive(Component)]
pub struct AudioEmitter {
    pub sound_type: SoundType,
    pub volume: f32,
    pub range: f32,
}

#[derive(Clone, Copy, PartialEq)]
pub enum SoundType {
    Footstep,
    WeaponFire,
    Reload,
    Ambient,
    EnemyAlert,
    DamageTaken,
}

fn setup_audio(mut commands: Commands, asset_server: Res<AssetServer>) {
    // In a real game, you would load actual audio files
    // For now, we'll just create handles to placeholder audio
    let game_audio = GameAudio {
        footstep_concrete: asset_server.load("audio/footstep_concrete.mp3"),
        weapon_fire: asset_server.load("audio/weapon_fire.mp3"),
        weapon_reload: asset_server.load("audio/weapon_reload.mp3"),
        ambient_hum: asset_server.load("audio/ambient_hum.mp3"),
        enemy_alert: asset_server.load("audio/enemy_alert.mp3"),
        damage_taken: asset_server.load("audio/damage_taken.mp3"),
    };

    commands.insert_resource(game_audio);

    // Create ambient audio emitter
    commands.spawn((
        Transform::default(),
        Visibility::default(),
        AudioEmitter {
            sound_type: SoundType::Ambient,
            volume: 0.3,
            range: 50.0,
        },
        Name::new("Ambient Audio"),
    ));

    info!("Audio system initialized");
}

fn play_footsteps(
    audio: Res<GameAudio>,
    kira_audio: Res<bevy_kira_audio::Audio>,
    player_query: Query<
        (
            &crate::player::PlayerController,
            &avian3d::prelude::LinearVelocity,
        ),
        Changed<avian3d::prelude::LinearVelocity>,
    >,
) {
    for (player_controller, velocity) in player_query.iter() {
        let speed = velocity.length();

        // Play footsteps based on movement speed
        if speed > 0.5 {
            let footstep_volume = if player_controller.is_sprinting {
                0.8
            } else if player_controller.is_crouching {
                0.2
            } else {
                0.5
            };

            // In a real implementation, you'd want to throttle this
            // and play footsteps at regular intervals based on speed
            if speed > 2.0 {
                // Only play if moving at reasonable speed
                kira_audio
                    .play(audio.footstep_concrete.clone())
                    .with_volume(footstep_volume as f64);
            }
        }
    }
}

fn play_weapon_sounds(
    audio: Res<GameAudio>,
    kira_audio: Res<bevy_kira_audio::Audio>,
    weapon_query: Query<&crate::combat::Weapon, Changed<crate::combat::Weapon>>,
) {
    for weapon in weapon_query.iter() {
        // Play reload sound when starting to reload
        if weapon.is_reloading {
            kira_audio
                .play(audio.weapon_reload.clone())
                .with_volume(0.6);
        }
    }
}

fn play_ambient_sounds(
    time: Res<Time>,
    audio: Res<GameAudio>,
    kira_audio: Res<bevy_kira_audio::Audio>,
    mut last_ambient_time: Local<f32>,
) {
    let current_time = time.elapsed_secs();

    // Play ambient sounds every 30 seconds
    if current_time - *last_ambient_time > 30.0 {
        kira_audio
            .play(audio.ambient_hum.clone())
            .with_volume(0.1)
            .looped();

        *last_ambient_time = current_time;
    }
}

// Helper functions for playing specific sounds
pub fn play_weapon_fire_sound(audio: &GameAudio, kira_audio: &bevy_kira_audio::Audio) {
    kira_audio.play(audio.weapon_fire.clone()).with_volume(0.7);
}

pub fn play_damage_sound(audio: &GameAudio, kira_audio: &bevy_kira_audio::Audio) {
    kira_audio.play(audio.damage_taken.clone()).with_volume(0.5);
}

pub fn play_enemy_alert_sound(
    audio: &GameAudio,
    kira_audio: &bevy_kira_audio::Audio,
    _position: Vec3,
) {
    // Note: Spatial audio might need additional setup in bevy_kira_audio
    kira_audio.play(audio.enemy_alert.clone()).with_volume(0.6);
}
