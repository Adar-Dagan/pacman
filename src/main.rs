use std::time::Duration;

use bevy::prelude::*;

use common::{app_state::{AppState, StateTimer}, events::{PelletEaten, PlayerAt, Collision, CollisionPauseTimer}, sets::GameLoop};
use ghosts::GhostMode;

mod common;
mod services;
mod map_render;
mod pellets;
mod player;
mod ghosts;

const SCALE: f32 = 2.0;
const MAX_MOVE_SPEED: f64 = 78.0; // In pixel per second

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
        .insert_resource(Time::<Fixed>::from_hz(MAX_MOVE_SPEED))
        .add_event::<PlayerAt>()
        .add_event::<PelletEaten>()
        .add_event::<Collision>()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(bevy_framepace::FramepacePlugin)
        .add_plugins((map_render::MapRenderPlugin, 
                      pellets::PelletsPlugin,
                      player::PlayerPlugin,
                      ghosts::GhostPlugin))
        .add_state::<AppState>()
        .insert_resource(StateTimer(Timer::from_seconds(5.0, TimerMode::Once)))
        .insert_resource(CollisionPauseTimer(Timer::from_seconds(0.0, TimerMode::Once)))
        .add_systems(Startup, (camera_setup, frame_rate_limiter))
        .add_systems(Update, state_transition)
        .add_systems(PostUpdate, state_transition_timer)
        .add_systems(FixedUpdate, advance_global_timer.before(GameLoop::Planning))
        .configure_sets(FixedUpdate, (
            common::sets::GameLoop::Planning,
            common::sets::GameLoop::Movement,
            common::sets::GameLoop::Collisions,
                ).chain()
                 .run_if(in_state(AppState::MainGame)))
        .run();
}

fn camera_setup(mut commands: Commands) {
    let mut camera = Camera2dBundle::default();
    camera.projection.scale  = 1.0 / SCALE;
    commands.spawn(camera);
}

fn frame_rate_limiter(mut settings: ResMut<bevy_framepace::FramepaceSettings>) {
    settings.limiter = bevy_framepace::Limiter::from_framerate(MAX_MOVE_SPEED);
}

fn state_transition(state: Res<State<AppState>>, 
                    mut next_state: ResMut<NextState<AppState>>,
                    mut timer: ResMut<StateTimer>,
                    time: Res<Time>) {
    if timer.0.tick(time.delta()).just_finished() {
         match state.get() {
            AppState::MainMenu => next_state.set(AppState::LevelStart),
            AppState::LevelStart => next_state.set(AppState::MainGame),
            AppState::MainGame => (),
            AppState::LevelComplete => next_state.set(AppState::LevelStart),
            AppState::GameOver => next_state.set(AppState::MainMenu),
        };
    }
}

fn state_transition_timer(mut timer: ResMut<StateTimer>, next_state: Res<NextState<AppState>>) {
    if let Some(next_state) = &next_state.0 {
        let secs_to_next_chage = match next_state {
            AppState::MainMenu => 3,
            AppState::LevelStart => 3,
            AppState::MainGame => return,
            AppState::LevelComplete => 6,
            AppState::GameOver => return,
        };
        timer.0.set_duration(Duration::from_secs(secs_to_next_chage));
        timer.0.reset();
    }
}

fn advance_global_timer(mut pause_timer: ResMut<CollisionPauseTimer>, 
                        time: Res<Time>,
                        mut collisions_events: EventReader<Collision>) {
    pause_timer.0.tick(time.delta());

    for event in collisions_events.read() {
        if matches!(event.mode, GhostMode::Frightened) {
            pause_timer.0.set_duration(Duration::from_secs(1));
            pause_timer.0.reset();
        }
    }
}
