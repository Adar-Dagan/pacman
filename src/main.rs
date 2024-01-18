#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use std::time::Duration;

use bevy::{
    input::{keyboard::KeyboardInput, ButtonState},
    prelude::*,
    render::camera::ScalingMode,
};
use bevy_kira_audio::prelude::*;

use common::{
    app_state::{AppState, DeadState, StateTimer},
    events::{CollisionPauseTimer, GetExtraLife, GhostEaten, PelletEaten, PlayerAt},
    levels::Levels,
    sets::GameLoop,
};
use services::{map::Location, text::TextProviderPlugin};

use bevy::winit::WinitWindows;
use winit::window::Icon;

mod background_sound;
mod common;
mod game_over;
mod ghosts;
mod leaderboard;
mod map_render;
mod menu;
mod pellets;
mod player;
mod points;
mod services;

const MAX_MOVE_SPEED: f64 = 78.0; // In pixel per second

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
        .insert_resource(Time::<Fixed>::from_hz(MAX_MOVE_SPEED))
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        present_mode: bevy::window::PresentMode::AutoNoVsync,
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins(AudioPlugin)
        .add_plugins(bevy_framepace::FramepacePlugin)
        .add_plugins(TextProviderPlugin)
        .insert_resource(StateTimer(
            Timer::from_seconds(0.0, TimerMode::Once)
                .tick(Duration::from_secs(1))
                .clone(),
        ))
        .insert_resource(CollisionPauseTimer(Timer::from_seconds(
            0.0,
            TimerMode::Once,
        )))
        .insert_resource(Levels::default())
        .add_event::<PlayerAt>()
        .add_event::<PelletEaten>()
        .add_event::<GetExtraLife>()
        .add_event::<GhostEaten>()
        .add_state::<AppState>()
        .add_state::<DeadState>()
        .configure_sets(
            FixedUpdate,
            (GameLoop::Planning, GameLoop::Movement, GameLoop::Collisions)
                .chain()
                .run_if(in_state(AppState::MainGame)),
        )
        .add_plugins((
            map_render::MapRenderPlugin,
            pellets::PelletsPlugin,
            player::PlayerPlugin,
            ghosts::GhostPlugin,
            menu::MenuPlugin,
            points::PointsPlugin,
            game_over::GameOverPlugin,
            leaderboard::LeaderboardPlugin,
            background_sound::BackgroundSoundPlugin,
        ))
        .add_systems(Startup, (camera_setup, frame_rate_limiter))
        .add_systems(
            PostUpdate,
            (timed_state_transition, update_entities_location),
        )
        .add_systems(OnEnter(AppState::LevelStart), advance_level)
        .add_systems(Update, escape_press)
        .add_systems(OnEnter(AppState::MainMenu), init)
        .add_systems(Startup, set_window_icon)
        .run();
}

pub fn init(mut collision_timer: ResMut<CollisionPauseTimer>, mut levels: ResMut<Levels>) {
    collision_timer.0.set_duration(Duration::from_secs(0));
    collision_timer.0.reset();

    levels.reset();
}

fn camera_setup(mut commands: Commands) {
    let mut camera = Camera2dBundle::default();
    camera.projection.scaling_mode = ScalingMode::AutoMin {
        min_width: 226.0,
        min_height: 288.0,
    };
    commands.spawn(camera);
}

fn frame_rate_limiter(mut settings: ResMut<bevy_framepace::FramepaceSettings>) {
    settings.limiter = bevy_framepace::Limiter::from_framerate(MAX_MOVE_SPEED);
}

fn timed_state_transition(
    state: Res<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,
    mut timer: ResMut<StateTimer>,
    time: Res<Time>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        match state.get() {
            AppState::LevelStart => next_state.set(AppState::MainGame),
            AppState::LevelComplete => next_state.set(AppState::LevelStart),
            _ => (),
        };
    }

    if let Some(next_state) = &next_state.0 {
        let secs_to_next_chage = match next_state {
            AppState::LevelStart => {
                if let AppState::MainMenu = state.get() {
                    4
                } else {
                    2
                }
            }
            AppState::LevelComplete => 6,
            _ => return,
        };
        timer
            .0
            .set_duration(Duration::from_secs(secs_to_next_chage));
        timer.0.reset();
        timer.0.unpause();
    }
}

pub fn advance_level(mut levels: ResMut<Levels>) {
    levels.next();
}

fn update_entities_location(mut query: Query<(&mut Transform, &Location), Changed<Location>>) {
    query.par_iter_mut().for_each(|(mut transform, location)| {
        transform.translation.x = (location.x - 13.5) * 8.0;
        transform.translation.y = (location.y - 15.5) * 8.0;
    });
}

fn escape_press(
    state: Res<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,
    mut keyboard_events: EventReader<KeyboardInput>,
    mut state_timer: ResMut<StateTimer>,
) {
    for event in keyboard_events.read() {
        if let KeyboardInput {
            state: ButtonState::Pressed,
            key_code: Some(KeyCode::Escape | KeyCode::Back),
            ..
        } = event
        {
            next_state.set(match state.get() {
                AppState::MainMenu | AppState::GameOver | AppState::Leaderboard => {
                    AppState::MainMenu
                }
                _ => AppState::GameOver,
            });
            state_timer.0.pause();
        }
    }
}

fn set_window_icon(
    // we have to use `NonSend` here
    windows: NonSend<WinitWindows>,
) {
    // here we use the `image` crate to load our icon data from a png file
    // this is not a very bevy-native solution, but it will do
    let (icon_rgba, icon_width, icon_height) = {
        let image = image::open("assets/icon.png")
            .expect("Failed to open icon path")
            .into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };
    let icon = Icon::from_rgba(icon_rgba, icon_width, icon_height).unwrap();

    // do it for all windows
    for window in windows.windows.values() {
        window.set_window_icon(Some(icon.clone()));
    }
}
