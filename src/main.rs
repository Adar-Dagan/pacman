use std::time::Duration;

use bevy::{
    input::{keyboard::KeyboardInput, ButtonState},
    prelude::*,
    render::camera::ScalingMode,
};

use common::{
    app_state::{AppState, StateTimer},
    events::{CollisionPauseTimer, GhostEaten, PelletEaten, PlayerAt},
    levels::Levels,
    sets::GameLoop,
};
use services::{map::Location, text::TextProviderPlugin};

mod common;
mod ghosts;
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
        .add_event::<GhostEaten>()
        .add_state::<AppState>()
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
        ))
        .add_systems(Startup, (camera_setup, frame_rate_limiter))
        .add_systems(
            PostUpdate,
            (timed_state_transition, update_entities_location),
        )
        .add_systems(OnEnter(AppState::LevelStart), advance_level)
        .add_systems(Update, go_to_menu)
        .add_systems(OnEnter(AppState::MainMenu), init)
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
            AppState::MainMenu => next_state.set(AppState::LevelStart),
            AppState::LevelStart => next_state.set(AppState::MainGame),
            AppState::MainGame => (),
            AppState::LevelComplete => next_state.set(AppState::LevelStart),
        };
    }

    if let Some(next_state) = &next_state.0 {
        let secs_to_next_chage = match next_state {
            AppState::MainMenu => return,
            AppState::LevelStart => 3,
            AppState::MainGame => return,
            AppState::LevelComplete => 6,
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

fn go_to_menu(
    mut next_state: ResMut<NextState<AppState>>,
    mut keyboard_events: EventReader<KeyboardInput>,
    mut state_timer: ResMut<StateTimer>,
) {
    for event in keyboard_events.read() {
        if let KeyboardInput {
            state: ButtonState::Pressed,
            key_code: Some(KeyCode::Escape),
            ..
        } = event
        {
            next_state.set(AppState::MainMenu);
            state_timer.0.pause();
        }
    }
}
