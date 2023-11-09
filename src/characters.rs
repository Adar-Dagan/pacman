use std::f32::consts::TAU;

use bevy::prelude::*;
use strum::{EnumIter, IntoEnumIterator};

use crate::services::{map::{Direction, Map, Location}, events::PlayerAt};

#[derive(Component)]
struct Player {
    pub is_blocked: bool
}

#[derive(Component, EnumIter)]
enum Ghost {
    Blinky,
    Pinky,
    Inky,
    Clyde,
}

#[derive(Component)]
struct GhostState {
    pub directions: Vec<Direction>,
}

#[derive(Component)]
struct Sprites([Handle<Image>; 3]);

pub struct CharacterPlugin;

impl Plugin for CharacterPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_characters);
        app.add_systems(FixedUpdate, (update_ghosts, update_player, map_wrap.after(update_player)));
        app.add_systems(Update, update_pacman_sprite);
    }
}

fn spawn_characters(mut commands: Commands,
                    asset_server: Res<AssetServer>) {
    commands.spawn((
        Location::new(13.5, 7.0),
        Player { is_blocked: false },
        Direction::Right,
        Sprites([
            asset_server.load("pacman_closed.png"),
            asset_server.load("pacman_open_small.png"),
            asset_server.load("pacman_open_large.png"),
        ]),
        SpriteBundle {
            texture: asset_server.load("pacman_closed.png"),
            transform: Transform::from_xyz(0.0, 0.0, 20.0),
            ..default()
        }));

    commands.spawn((
            Location::new(13.0, 19.0),
            Ghost::Blinky,
            GhostState {
                directions: vec![Direction::Left, Direction::Left]
            },
            SpriteBundle {
                sprite: Sprite {
                    color: Color::rgb(1.0, 0.0, 0.0),
                    custom_size: Some(Vec2::new(4.0, 4.0)),
                    ..default()
                },
                transform: Transform::from_xyz(0.0, 0.0, 990.0),
                ..default()
            }));

    commands.spawn((
            Location::new(11.0, 19.0),
            Ghost::Pinky,
            GhostState {
                directions: vec![Direction::Left, Direction::Left]
            },
            SpriteBundle {
                sprite: Sprite {
                    color: Color::rgb(1.0, 0.75, 0.79),
                    custom_size: Some(Vec2::new(4.0, 4.0)),
                    ..default()
                },
                transform: Transform::from_xyz(0.0, 0.0, 990.0),
                ..default()
            }));

    commands.spawn((
            Location::new(11.0, 19.0),
            Ghost::Inky,
            GhostState {
                directions: vec![Direction::Left, Direction::Left]
            },
            SpriteBundle {
                sprite: Sprite {
                    color: Color::rgb(0.0, 0.75, 1.0),
                    custom_size: Some(Vec2::new(4.0, 4.0)),
                    ..default()
                },
                transform: Transform::from_xyz(0.0, 0.0, 990.0),
                ..default()
            }));

    commands.spawn((
            Location::new(11.0, 19.0),
            Ghost::Clyde,
            GhostState {
                directions: vec![Direction::Left, Direction::Left]
            },
            SpriteBundle {
                sprite: Sprite {
                    color: Color::rgb(1.0, 0.75, 0.0),
                    custom_size: Some(Vec2::new(4.0, 4.0)),
                    ..default()
                },
                transform: Transform::from_xyz(0.0, 0.0, 990.0),
                ..default()
            }));
}

fn update_player(mut query: Query<(&mut Location, 
                                   &mut Direction, 
                                   &mut Player)>,
                 map: Res<Map>,
                 key: Res<Input<KeyCode>>,
                 mut player_at_events: EventWriter<PlayerAt>) {
    let (mut location, mut direction, mut player) = query.single_mut();

    let possible_directions = map.possible_directions(*location);

    let new_direction = possible_directions.iter().filter(|direction| {
        match **direction {
            Direction::Up => key.pressed(KeyCode::Up),
            Direction::Down => key.pressed(KeyCode::Down),
            Direction::Left => key.pressed(KeyCode::Left),
            Direction::Right => key.pressed(KeyCode::Right),
        }
    }).next();

    if let Some(d) = new_direction {
        if *d != *direction {
            *direction = *d;
        }
    }

    let mut new_location = location.clone();
    new_location.advance(*direction);

    if map.is_blocked(new_location + direction.get_vec() * 0.5) {
        player.is_blocked = true;
        new_location = Location::from_vec(new_location.round());
    } else {
        player.is_blocked = false;
        match *direction {
            Direction::Up | Direction::Down => {
                new_location.x = bring_to_center(new_location.x);
            },
            Direction::Left | Direction::Right => {
                new_location.y = bring_to_center(new_location.y);
            },
        };
    }
    *location = new_location;

    player_at_events.send(PlayerAt { location: location.get_tile(*direction) });
}

fn bring_to_center(location: f32) -> f32 {
    if location.fract() == 0.0 {
        return location;
    }

    let dif_from_center = location.round() - location; 
    let dif_sign = dif_from_center.signum();
    let location = location + dif_sign * Location::ADVANCEMENT_DELTA;
    
    location
}

fn map_wrap(mut query: Query<&mut Location>, map: Res<Map>) {
    query.par_iter_mut().for_each(|mut location| {
        if location.x == -2.0 {
            location.x = map.width() as f32 + 1.0;
        } else if location.x == (map.width() as f32 + 1.0) {
            location.x = -2.0;
        }

        if location.y == -2.0 {
            location.y = map.height() as f32 + 1.0;
        } else if location.y == (map.height() as f32 + 1.0) {
            location.y = -2.0;
        }
    });
}

fn update_pacman_sprite(mut query: Query<(&Location, &mut Handle<Image>, &mut Transform, &Direction, &Sprites, &Player)>) {
    let (location, mut sprite, mut transform, direction, sprites, player) = 
        query.single_mut();

    let sprite_index = if player.is_blocked {
        1
    } else {
        let masked_location = *location * *direction.get_vec();
        let value_in_direction = if masked_location.x.fract() == 0.0 {
            location.y
        } else {
            location.x
        };

        let quarter = ((value_in_direction * 4.0).floor() as usize + 1) % 4;
        if quarter == 3 { 1 } else { quarter }
    };

    *sprite = sprites.0[sprite_index].clone();

    let rotation_multiplier = match *direction {
        Direction::Left => 0.0,
        Direction::Down => 1.0,
        Direction::Right => 2.0,
        Direction::Up => 3.0,
    };

    let rotation = Quat::from_rotation_z(rotation_multiplier * TAU / 4.0);
    if transform.rotation != rotation {
        transform.rotation = rotation;
    }
}

fn update_ghosts(mut query: Query<(&mut Location, &mut GhostState, &Ghost), Without<Player>>,
                 player_query: Query<(&Location, &Direction), With<Player>>,
                 map: Res<Map>) {
    let map = &*map;
    let (player_location, player_direction) = player_query.single();
    let player_tile = player_location.get_tile(*player_direction);

    let mut blinky_tile_iter = query.iter().filter_map(|(location, state, ghost)| {
        if let Ghost::Blinky = ghost {
            let mut location = location.clone();
            location.advance(state.directions[0]);
            Some(location.get_tile(state.directions[0]))
        } else { None }
    });

    let blinky_tile = blinky_tile_iter.next().expect("No blinky");
    if blinky_tile_iter.next().is_some() {
        panic!("More than one blinky");
    }

    for (mut location, mut state, ghost) in query.iter_mut() {
        let current_direction = state.directions[0];
        location.advance(current_direction);

        let current_tile = location.get_tile(current_direction);

        if *location == current_tile {
            state.directions.remove(0);
        }

        if !location.is_on_tile_edge() {
            continue;
        }

        if !map.is_in_map(current_tile) {
            let last_direction = *state.directions.last().expect("No directions");
            state.directions.push(last_direction);
            continue;
        }

        let next_direction = state.directions[1];
        let next_tile = current_tile.next_tile(next_direction);

        let target_tile = match ghost {
            Ghost::Blinky => player_tile,
            Ghost::Pinky => player_tile + player_direction.get_vec() * 4.0,
            Ghost::Inky => {
                let offset_tile = player_tile + player_direction.get_vec() * 2.0;
                let blinky_offset_vector = offset_tile - blinky_tile;
                blinky_tile + blinky_offset_vector * 2.0
            },
            Ghost::Clyde => {
                let distance = (player_tile - current_tile).length_squared();
                if distance > 8.0 * 8.0 {
                    player_tile
                } else {
                    Location::new(0.0, -1.0)
                }
            },
        };


        state.directions.push(ghost_path_finder(next_tile,
                                                 target_tile,
                                                 map,
                                                 next_direction));
    }
}

fn ghost_path_finder(next_tile: Location,
                     target_tile: Location,
                     map: &Map,
                     current_direction: Direction) -> Direction {
    let mut possible_directions = map.possible_directions(next_tile);

    possible_directions.retain(|direction| {
        *direction != current_direction.opposite()
    });

    possible_directions.sort_by(|direction1, direction2| {
        let tile1 = next_tile.next_tile(*direction1);
        let tile2 = next_tile.next_tile(*direction2);

        let distance1 = (tile1 - target_tile).length_squared();
        let distance2 = (tile2 - target_tile).length_squared();

        distance1.partial_cmp(&distance2).unwrap()
    });

    possible_directions[0]
}

