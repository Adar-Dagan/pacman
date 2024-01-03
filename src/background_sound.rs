use bevy::prelude::*;
use bevy_kira_audio::prelude::*;

use crate::{
    common::{app_state::AppState, events::PelletEaten},
    ghosts::GhostMode,
};

#[derive(Resource, Default)]
struct BackgroundSounds {
    sirens: [Handle<AudioSource>; 5],
    ghost_going_home: Handle<AudioSource>,
    ghost_frite: Handle<AudioSource>,
    currently_playing: Option<Handle<AudioSource>>,
    playing_instance: Option<Handle<AudioInstance>>,
}

#[derive(Resource)]
struct PelletEatenCounter(usize);

pub struct BackgroundSoundPlugin;

impl Plugin for BackgroundSoundPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, load_sounds);
        app.add_systems(OnEnter(AppState::LevelStart), zero_pellet_eaten);
        app.add_systems(
            Update,
            change_background_sound.run_if(in_state(AppState::MainGame)),
        );
        app.add_systems(OnExit(AppState::MainGame), stop_sirens);
        app.insert_resource(BackgroundSounds::default());
        app.insert_resource(PelletEatenCounter(0));
    }
}

fn load_sounds(mut background_sounds: ResMut<BackgroundSounds>, asset_server: Res<AssetServer>) {
    for i in 0..5 {
        background_sounds.sirens[i] = asset_server.load(format!("sounds/siren_{}.wav", i + 1));
    }
    background_sounds.ghost_going_home = asset_server.load("sounds/ghost_going_home.wav");
    background_sounds.ghost_frite = asset_server.load("sounds/ghosts_frite.wav");
}

fn zero_pellet_eaten(mut pellet_eaten: ResMut<PelletEatenCounter>) {
    pellet_eaten.0 = 0;
}

fn change_background_sound(
    mut background_sounds: ResMut<BackgroundSounds>,
    audio: Res<Audio>,
    mut audio_instances: ResMut<Assets<AudioInstance>>,
    mut pellet_eaten: ResMut<PelletEatenCounter>,
    mut pellet_eaten_events: EventReader<PelletEaten>,
    ghost_mode_query: Query<&GhostMode>,
) {
    pellet_eaten.0 += pellet_eaten_events.read().count();
    let siren = match pellet_eaten.0 {
        0..=114 => 0,
        115..=179 => 1,
        180..=209 => 2,
        210..=224 => 3,
        225.. => 4,
        _ => unreachable!(),
    };

    let ghosts_mode = ghost_mode_query
        .iter()
        .fold(GhostMode::Scatter, |acc, mode| {
            if let GhostMode::Dead = acc {
                return acc;
            }

            match mode {
                GhostMode::Dead | GhostMode::DeadEnterHome => return GhostMode::Dead,
                GhostMode::Frightened | GhostMode::DeadPause => return GhostMode::Frightened,
                _ => (),
            }

            acc
        });

    let background_sound_handle = if let GhostMode::Dead = ghosts_mode {
        background_sounds.ghost_going_home.clone()
    } else if let GhostMode::Frightened = ghosts_mode {
        background_sounds.ghost_frite.clone()
    } else {
        background_sounds.sirens[siren].clone()
    };

    if background_sounds.currently_playing.is_none()
        || background_sound_handle != background_sounds.currently_playing.clone().unwrap()
    {
        if let Some(instance) = background_sounds
            .playing_instance
            .clone()
            .and_then(|handle| audio_instances.get_mut(handle))
        {
            instance.stop(AudioTween::default());
        }

        let handle = audio
            .play(background_sound_handle.clone())
            .looped()
            .handle();

        background_sounds.playing_instance = Some(handle);
        background_sounds.currently_playing = Some(background_sound_handle);
    }
}

fn stop_sirens(
    mut background_sounds: ResMut<BackgroundSounds>,
    mut audio_instances: ResMut<Assets<AudioInstance>>,
) {
    if let Some(instance) =
        audio_instances.get_mut(&background_sounds.playing_instance.clone().unwrap())
    {
        instance.stop(AudioTween::default());
    }

    background_sounds.playing_instance = None;
    background_sounds.currently_playing = None;
}
