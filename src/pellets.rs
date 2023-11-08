use bevy::prelude::*;

use crate::services::{map::Location, events::PlayerAt};

#[derive(Component, Copy, Clone)]
enum PelletType {
    Regular,
    Power,
}

pub struct PelletsPlugin;

impl Plugin for PelletsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_pellets);
        app.add_systems(FixedUpdate, remove_pellets);
    }
}

fn spawn_pellets(mut commands: Commands, asset_server: Res<AssetServer>) {
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

    for (x, y, pellet_type) in pellets_iter {
        commands.spawn(
            (pellet_type, 
             Location::new(x, y),
             SpriteBundle {
                 texture: asset_server.load( match pellet_type {
                     PelletType::Regular => "pellet.png",
                     PelletType::Power => "power_pellet.png",
                 }),
                 transform: Transform::from_xyz(0.0, 0.0, 10.0),
                 ..default()
             }));
    }
}

fn remove_pellets(mut commands: Commands, 
                  query: Query<(Entity, &Location), With<PelletType>>, 
                  mut player_at_events: EventReader<PlayerAt>) {
    let player_locations = player_at_events
        .read()
        .map(|event| event.location)
        .collect::<Vec<_>>();

    for (entity, location) in query.iter() {
        if player_locations.contains(location) {
            commands.entity(entity).despawn();
        }
    }
}
