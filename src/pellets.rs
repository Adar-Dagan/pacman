use bevy::prelude::*;

use crate::common::app_state::AppState;
use crate::common::layers::Layers;
use crate::services::map::Location;
use crate::common::events::{PlayerAt, PelletEaten};
use crate::common::sets::GameLoop::Collisions;

#[derive(Component, Copy, Clone)]
pub enum PelletType {
    Regular,
    Power,
}

#[derive(Resource)]
pub struct TotalPellets(pub usize);

#[derive(Resource)]
struct PowerPelletFlashTimer(Timer);

pub struct PelletsPlugin;

impl Plugin for PelletsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::LevelStart), spawn_pellets);
        app.add_systems(FixedUpdate, remove_pellets
                        .in_set(Collisions));
        app.add_systems(Update, flash_power_pellets);
        app.insert_resource(PowerPelletFlashTimer(Timer::from_seconds(0.5, TimerMode::Repeating)));
        app.insert_resource(TotalPellets(0));
    }
}

fn spawn_pellets(mut commands: Commands, asset_server: Res<AssetServer>, mut total_pellets: ResMut<TotalPellets>) {
    const PELLETS_TEXT: &str = include_str!("pellets");
    const PARSING_ERROR: &str = "Error parsing pellets file";

    let pellets_iter = PELLETS_TEXT.lines().map(|line| {
        let (coordinates_text, type_text) = line.split_once(' ')?;
        let (x_text, y_text) = coordinates_text.split_once(',')?;

        let x = x_text.parse::<f32>().ok()?;
        let y = y_text.parse::<f32>().ok()?;
        let pellet_type = match type_text {
            "Regular" => PelletType::Regular,
            "Power" => PelletType::Power,
            _ => return None,
        };

        Some((x, y, pellet_type))
    }).map(|option| option.expect(PARSING_ERROR));

    total_pellets.0 = 0;
    for (x, y, pellet_type) in pellets_iter {
        total_pellets.0 += 1;
        commands.spawn(
            (pellet_type, 
             Location::new(x, y),
             SpriteBundle {
                 texture: asset_server.load( match pellet_type {
                     PelletType::Regular => "pellet.png",
                     PelletType::Power => "power_pellet.png",
                 }),
                 transform: Transform::from_xyz(0.0, 0.0, Layers::Pellets.as_f32()),
                 ..default()
             }));
    }
}

fn remove_pellets(mut commands: Commands, 
                  query: Query<(Entity, &Location, &PelletType)>, 
                  mut player_at_events: EventReader<PlayerAt>,
                  mut pellets_eaten_events: EventWriter<PelletEaten>,
                  mut next_game_state: ResMut<NextState<AppState>>) {
    let player_locations = player_at_events
        .read()
        .map(|event| event.location)
        .collect::<Vec<_>>();

    for (entity, location, pellet_type) in query.iter() {
        if player_locations.contains(location) {
            pellets_eaten_events.send(PelletEaten {
                power: matches!(pellet_type, PelletType::Power),
            });
            commands.entity(entity).despawn();
        }
    }

    let pellets_left = query.iter().count();
    if pellets_left == 0 {
        next_game_state.set(AppState::LevelComplete);
    }
}

fn flash_power_pellets(mut query: Query<(&PelletType, &mut Visibility)>,
                       mut timer: ResMut<PowerPelletFlashTimer>,
                       time: Res<Time>) {
    if !timer.0.tick(time.delta()).just_finished() {
        return;
    }

    for (pellet_type, mut visibility) in query.iter_mut() {
        if matches!(pellet_type, PelletType::Power) {
            *visibility = match *visibility {
                Visibility::Inherited => Visibility::Hidden,
                Visibility::Hidden => Visibility::Inherited,
                Visibility::Visible => unreachable!(),
            };
        }
    }
}

