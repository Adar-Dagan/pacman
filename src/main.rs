use bevy::prelude::*;

mod common;
mod services;
mod map_render;
mod pellets;
mod player;
mod ghosts;

const SCALE: f32 = 2.0;
const MAX_MOVE_SPEED: f64 = 70.0; // In pixel per second

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
        .insert_resource(Time::<Fixed>::from_hz(MAX_MOVE_SPEED))
        .add_event::<common::events::PlayerAt>()
        .add_event::<common::events::PelletEaten>()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(bevy_framepace::FramepacePlugin)
        .add_plugins((map_render::MapRenderPlugin, 
                      pellets::PelletsPlugin,
                      player::PlayerPlugin,
                      ghosts::GhostPlugin))
        .add_systems(Startup, (camera_setup, frame_rate_limiter))
        .configure_sets(FixedUpdate, (
            common::sets::GameLoop::Planning,
            common::sets::GameLoop::Movement,
            common::sets::GameLoop::Collisions,
                ).chain())
        .run();
}

fn camera_setup(mut commands: Commands) {
    let mut camera = Camera2dBundle::default();
    camera.projection.scale  = 1.0 / SCALE;
    commands.spawn(camera);
}

fn frame_rate_limiter(mut settings: ResMut<bevy_framepace::FramepaceSettings>) {
    settings.limiter = bevy_framepace::Limiter::from_framerate(60.0);
}
