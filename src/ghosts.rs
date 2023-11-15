use std::time::Duration;

use bevy::prelude::*;
use strum::{ EnumIter, IntoEnumIterator };

use crate::common::layers::Layers;
use crate::common::sets::GameLoop;
use crate::player::Player;
use crate::services::map::{Direction, Map, Location};

const DIRECTIONS_CAPACITY: usize = 3;

#[derive(Component, EnumIter, Clone, Copy)]
enum Ghost {
    Blinky,
    Pinky,
    Inky,
    Clyde,
}

#[derive(Component, Debug, Clone, Copy)]
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

    fn replace_next(&mut self, direction: Direction) {
        self.directions[(self.current + 1) % DIRECTIONS_CAPACITY] = Some(direction);
    }

    fn replace_planned(&mut self, direction: Direction) {
        let planned = &mut self.directions[(self.current + 2) % DIRECTIONS_CAPACITY];
        if planned.is_none() {
            panic!("Trying to replace a non-planned direction");
        }
        *planned = Some(direction);
    }
}

#[derive(Resource, Debug, Clone, Copy, PartialEq)]
enum GhostMode {
    Chase,
    Scatter
}

#[derive(Resource)]
struct GhostModeTimer{
    timer: Timer,
    duration_index: usize,
}

const CHANGE_DURATIONS: [u64; 7] = [7, 20, 7, 20, 5, 20, 5];

#[derive(Event)]
pub struct GhostModeChange {
    mode: GhostMode,
}

pub struct GhostPlugin;

impl Plugin for GhostPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GhostMode::Scatter);
        app.insert_resource(GhostModeTimer {
            timer: Timer::from_seconds(CHANGE_DURATIONS[0] as f32, TimerMode::Once),
            duration_index: 0,
        });
        app.add_event::<GhostModeChange>();
        app.add_systems(Startup, spawn_ghosts);
        app.add_systems(FixedUpdate, (update_ghosts.in_set(GameLoop::Planning),
                                        evaluate_ghost_mode.in_set(GameLoop::Planning),
                                        move_ghosts.in_set(GameLoop::Movement), 
                                        draw_ghosts.after(GameLoop::Planning)
                                                    .before(GameLoop::Movement)));
    }
}

fn spawn_ghosts(mut commands: Commands,
                asset_server: Res<AssetServer>,
                mut texture_atlases: ResMut<Assets<TextureAtlas>>) {
     for ghost in Ghost::iter() {
         spawn_ghost(ghost, &mut commands, &asset_server, &mut texture_atlases);
     }
}

fn spawn_ghost(ghost: Ghost,
               commands: &mut Commands,
               asset_server: &Res<AssetServer>,
               texture_atlases: &mut ResMut<Assets<TextureAtlas>>) {
    let texture_path = match ghost {
        Ghost::Blinky => "blinky.png",
        Ghost::Pinky => "pinky.png",
        Ghost::Inky => "inky.png",
        Ghost::Clyde => "clyde.png",
    };

    let texture_handle = asset_server.load(texture_path);
    let texture_atlas =
            TextureAtlas::from_grid(texture_handle, Vec2::new(16.0, 16.0), 8, 1, None, None);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);

    commands.spawn((
                Location::new(13.0, 19.0),
                ghost,
                GhostDirections::new(),
                SpriteSheetBundle {
                    texture_atlas: texture_atlas_handle,
                    sprite: TextureAtlasSprite::new(0),
                    transform: Transform::from_xyz(0.0, 0.0, Layers::Ghosts.as_f32()),
                    ..default()
                }));
}



fn update_ghosts(mut query: Query<(&Location, &mut GhostDirections, &Ghost), Without<Player>>,
                 player_query: Query<(&Location, &Direction), With<Player>>,
                 map: Res<Map>,
                 ghost_mode: Res<GhostMode>,
                 mut mode_change_events: EventReader<GhostModeChange>) {
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

    let mode_changed = mode_change_events.read().next();

    for (location, mut directions, ghost) in query.iter_mut() {
        let current_direction = directions.current();

        let current_tile = location.get_tile(current_direction);


        if location.is_tile_center() {
            directions.remove_first();
        }

        if let Some(GhostModeChange { mode }) = mode_changed {
            if mode != &GhostMode::Chase {
                directions.replace_next(current_direction.opposite());

                if directions.size == 3 {
                    let planning_state = PlanningState {
                        directions: *directions,
                        current_tile,
                        ghost_mode: *mode,
                        ghost: *ghost,
                        player_tile,
                        player_direction: *player_direction,
                        blinky_tile,
                        map,
                    };

                    directions.replace_planned(next_planned_direction(&planning_state));
                }
            }
        }

        if !location.is_on_tile_edge() {
            continue;
        }

        if !map.is_in_map(current_tile) {
            let last_direction = directions.last();
            directions.push(last_direction);
            continue;
        }

        let planning_state = PlanningState {
            directions: *directions,
            current_tile,
            ghost_mode: *ghost_mode,
            ghost: *ghost,
            player_tile,
            player_direction: *player_direction,
            blinky_tile,
            map,
        };

        directions.push(next_planned_direction(&planning_state));
    }
}

struct PlanningState<'a> {
    directions: GhostDirections,
    current_tile: Location,
    ghost_mode: GhostMode,
    ghost: Ghost,
    player_tile: Location,
    player_direction: Direction,
    blinky_tile: Location,
    map: &'a Map,
}

fn scatter(ghost: &Ghost) -> Location {
    match ghost {
        Ghost::Blinky => Location::new(28.0, 30.0),
        Ghost::Pinky => Location::new(2.0, 30.0),
        Ghost::Inky => Location::new(30.0, -1.0),
        Ghost::Clyde => Location::new(0.0, -1.0),
    }
}

fn chase_target(planning_state: &PlanningState) -> Location {
    let PlanningState { ghost, player_tile, player_direction, blinky_tile, current_tile, .. } = planning_state;
    let player_tile = *player_tile;
    match ghost {
        Ghost::Blinky => player_tile,
        Ghost::Pinky => player_tile + player_direction.get_vec() * 4.0,
        Ghost::Inky => {
            let offset_tile = player_tile + player_direction.get_vec() * 2.0;
            let blinky_offset_vector = offset_tile - *blinky_tile;
            *blinky_tile + blinky_offset_vector * 2.0
        },
        Ghost::Clyde => {
            let distance = (player_tile - *current_tile).length_squared();
            if distance > 8.0 * 8.0 {
                player_tile
            } else {
                scatter(ghost)
            }
        },
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

fn draw_ghosts(mut query: Query<(&mut TextureAtlasSprite, &GhostDirections, &Location), With<Ghost>>) {
    query.par_iter_mut().for_each(|(mut sprite, directions, location)| {
        if !location.is_tile_center() {
            return;
        }

        let current_direction = directions.current();

        let rotation = (current_direction.rotation() * 4.0) as usize;

        let variation = (sprite.index + 1) % 2;

        sprite.index = rotation * 2 + variation;
    });
}

fn evaluate_ghost_mode(mut ghost_mode: ResMut<GhostMode>,
                       mut mode: ResMut<GhostModeTimer>,
                       time: Res<Time>, 
                       mut mode_change_events: EventWriter<GhostModeChange>) {
    if !mode.timer.tick(time.delta()).just_finished() {
        return;
    }

    *ghost_mode = match *ghost_mode {
        GhostMode::Chase => GhostMode::Scatter,
        GhostMode::Scatter => GhostMode::Chase,
    };

    mode_change_events.send(GhostModeChange { mode: *ghost_mode });

    mode.duration_index += 1;
    if let Some(duration) = CHANGE_DURATIONS.get(mode.duration_index) {
        mode.timer.set_duration(Duration::from_secs(*duration));
        mode.timer.reset();
    }
}

fn next_planned_direction(planning_state: &PlanningState) -> Direction {
    let PlanningState { directions, current_tile, ghost_mode, ghost, map, .. } = planning_state;
    let next_direction = directions.next();
    let next_tile = current_tile.next_tile(next_direction);

    let target_tile = match ghost_mode {
                GhostMode::Chase => chase_target(planning_state),
                GhostMode::Scatter => scatter(ghost)
            };

    ghost_path_finder(next_tile,
                              target_tile,
                              map,
                              next_direction)
}
