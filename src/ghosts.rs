use bevy::prelude::*;

use crate::common::layers::Layers;
use crate::common::sets::GameLoop;
use crate::player::Player;
use crate::services::map::{Direction, Map, Location};

const DIRECTIONS_CAPACITY: usize = 8;

#[derive(Component)]
enum Ghost {
    Blinky,
    Pinky,
    Inky,
    Clyde,
}

#[derive(Component, Debug)]
struct GhostDirections {
    directions: [Option<Direction>; DIRECTIONS_CAPACITY],
    current: usize,
    size: usize,
}

impl GhostDirections {

    fn new() -> Self {
        Self {
            directions: [None; DIRECTIONS_CAPACITY],
            current: 0,
            size: 0,
        }
    }

    fn current(&self) -> Direction {
        self.directions[self.current].expect("No current direction")
    }

    fn remove_first(&mut self) {
        self.directions[self.current] = None;
        self.current = (self.current + 1) % DIRECTIONS_CAPACITY;
        self.size -= 1;
        if self.size < 2 {
            panic!("Not enough directions");
        }
    }

    fn last(&self) -> Direction {
        self.directions[(self.current + self.size - 1) % DIRECTIONS_CAPACITY].expect("No last direction")
    }

    fn push(&mut self, direction: Direction) {
        self.directions[(self.current + self.size) % DIRECTIONS_CAPACITY] = Some(direction);
        self.size += 1;
        if self.size > DIRECTIONS_CAPACITY {
            panic!("Directions overflow");
        }
    }

    fn next(&self) -> Direction {
        self.directions[(self.current + 1) % DIRECTIONS_CAPACITY].expect("No next direction")
    }
}


pub struct GhostPlugin;

impl Plugin for GhostPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_ghosts);
        app.add_systems(FixedUpdate, (update_ghosts.in_set(GameLoop::Planning),
                                        move_ghosts.in_set(GameLoop::Movement)));
    }
}

fn spawn_ghosts(mut commands: Commands) {
    commands.spawn((
            Location::new(13.0, 19.0),
            Ghost::Blinky,
            GhostDirections::new(),
            SpriteBundle {
                sprite: Sprite {
                    color: Color::rgb(1.0, 0.0, 0.0),
                    custom_size: Some(Vec2::new(4.0, 4.0)),
                    ..default()
                },
                transform: Transform::from_xyz(0.0, 0.0, Layers::Ghosts.as_f32()),
                ..default()
            }));

    commands.spawn((
            Location::new(11.0, 19.0),
            Ghost::Pinky,
            GhostDirections::new(),
            SpriteBundle {
                sprite: Sprite {
                    color: Color::rgb(1.0, 0.75, 0.79),
                    custom_size: Some(Vec2::new(4.0, 4.0)),
                    ..default()
                },
                transform: Transform::from_xyz(0.0, 0.0, Layers::Ghosts.as_f32()),
                ..default()
            }));

    commands.spawn((
            Location::new(11.0, 19.0),
            Ghost::Inky,
            GhostDirections::new(),
            SpriteBundle {
                sprite: Sprite {
                    color: Color::rgb(0.0, 0.75, 1.0),
                    custom_size: Some(Vec2::new(4.0, 4.0)),
                    ..default()
                },
                transform: Transform::from_xyz(0.0, 0.0, Layers::Ghosts.as_f32()),
                ..default()
            }));

    commands.spawn((
            Location::new(11.0, 19.0),
            Ghost::Clyde,
            GhostDirections::new(),
            SpriteBundle {
                sprite: Sprite {
                    color: Color::rgb(1.0, 0.75, 0.0),
                    custom_size: Some(Vec2::new(4.0, 4.0)),
                    ..default()
                },
                transform: Transform::from_xyz(0.0, 0.0, Layers::Ghosts.as_f32()),
                ..default()
            }));
}

fn update_ghosts(mut query: Query<(&Location, &mut GhostDirections, &Ghost), Without<Player>>,
                 player_query: Query<(&Location, &Direction), With<Player>>,
                 map: Res<Map>) {
    let map = &*map;
    let (player_location, player_direction) = player_query.single();
    let player_tile = player_location.get_tile(*player_direction);

    //TODO: Temporary initialization to be removed when ghost states are implemented
    query.iter_mut().for_each(|(_, mut state, _)| {
        if state.size != 0 {
            return;
        }
        state.directions[0] = Some(Direction::Left);
        state.directions[1] = Some(Direction::Left);
        state.directions[2] = Some(Direction::Left);
        state.size = 3;
    });

    let mut blinky_tile_iter = query.iter().filter_map(|(location, directions, ghost)| {
        if let Ghost::Blinky = ghost {
            let current_direction = directions.current();
            Some(location.get_tile(current_direction))
        } else { None }
    });

    let blinky_tile = blinky_tile_iter.next().expect("No blinky");
    if blinky_tile_iter.next().is_some() {
        panic!("More than one blinky");
    }

    for (location, mut directions, ghost) in query.iter_mut() {
        let current_direction = directions.current();

        let current_tile = location.get_tile(current_direction);


        if *location == current_tile {
            directions.remove_first();
        }

        if !location.is_on_tile_edge() {
            continue;
        }

        if !map.is_in_map(current_tile) {
            let last_direction = directions.last();
            directions.push(last_direction);
            continue;
        }

        let next_direction = directions.next();
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


        directions.push(ghost_path_finder(next_tile,
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

fn move_ghosts(mut query: Query<(&mut Location, &GhostDirections), With<Ghost>>) {
    query.par_iter_mut().for_each(|(mut location, directions)| {
        let current_direction = directions.current();
        location.advance(current_direction);
    });
}
