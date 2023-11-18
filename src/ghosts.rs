use std::time::Duration;

use bevy::prelude::*;
use strum::{ EnumIter, IntoEnumIterator };

use crate::common::app_state::{AppState, StateTimer};
use crate::common::events::{PelletEaten, Collision, CollisionPauseTimer};
use crate::common::layers::Layers;
use crate::common::sets::GameLoop;
use crate::pellets::TotalPellets;
use crate::player::Player;
use crate::services::map::{Direction, Map, Location};
use crate::services::speed::CharacterSpeed;

const GHOST_DEBUG: bool = false;

#[derive(Component, EnumIter, Clone, Copy, PartialEq)]
pub enum Ghost {
    Blinky,
    Pinky,
    Inky,
    Clyde,
}

#[derive(Component, Debug, Clone, Copy)]
struct GhostDirections {
    current: Direction,
    planned: Option<Direction>,
}

impl GhostDirections {
    fn new(direction: Direction) -> Self {
        Self {
            current: direction,
            planned: Some(direction),
        }
    }

    fn advance(&mut self) {
        self.current = self.planned.unwrap_or(self.current);
        self.planned = None;
    }

    fn plan_needed(&self) -> bool {
        self.planned.is_none()
    }

    fn set_plan(&mut self, direction: Direction) {
        self.planned = Some(direction);
    }

    fn reverse(&mut self) {
        self.planned = Some(self.current.opposite());
    }
}

#[derive(Resource, Component, Debug, Clone, Copy, PartialEq)]
pub enum GhostMode {
    Home(bool),
    HomeExit(bool),
    Chase,
    Scatter,
    Frightened,
    DeadPause,
    Dead(bool),
}

#[derive(Resource)]
struct FriteTimer(Timer);

#[derive(Resource)]
struct GlobalGhostModeTimer{
    timer: Timer,
    duration_index: usize,
}

#[derive(Component, EnumIter)]
enum GhostSprite {
    Body,
    Eyes,
    Frightened,
}

#[derive(Resource)]
struct GhostPelletEatenCounter {
    counter: usize,
    life_lost: bool
}

const CHANGE_DURATIONS: [u64; 7] = [7, 20, 7, 20, 5, 20, 5];

pub struct GhostPlugin;

impl Plugin for GhostPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GhostMode::Scatter);
        app.insert_resource(GlobalGhostModeTimer {
            timer: Timer::from_seconds(CHANGE_DURATIONS[0] as f32, TimerMode::Once),
            duration_index: 0,
        });
        app.insert_resource(FriteTimer(Timer::from_seconds(0.0, TimerMode::Once)));
        app.insert_resource(GhostPelletEatenCounter { counter: 0, life_lost: false });

        app.add_systems(OnEnter(AppState::LevelStart), spawn_ghosts);
        app.add_systems(OnEnter(AppState::MainGame), init_resources);
        app.add_systems(FixedUpdate, (timer_pause,
                                      update_global_ghost_mode,
                                      update_ghost_mode,
                                      update_ghost_speed,
                                      ghost_tile_change_detection,
                                      plan_ghosts)
                        .chain()
                        .in_set(GameLoop::Planning));
        app.add_systems(FixedUpdate, move_ghosts.in_set(GameLoop::Movement));
        app.add_systems(FixedUpdate, collision_detection.in_set(GameLoop::Collisions));

        app.add_systems(Update, despawn_ghosts.run_if(in_state(AppState::LevelComplete)));

        app.add_systems(Update, draw_ghosts);
    }
}

fn spawn_ghosts(mut commands: Commands,
                asset_server: Res<AssetServer>,
                mut texture_atlases: ResMut<Assets<TextureAtlas>>) {
    if GHOST_DEBUG {
        spawn_ghost(Ghost::Blinky, &mut commands, &asset_server, &mut texture_atlases);
    }else {
        for ghost in Ghost::iter() {
            spawn_ghost(ghost, &mut commands, &asset_server, &mut texture_atlases);
        }
    }
}

fn spawn_ghost(ghost: Ghost,
               commands: &mut Commands,
               asset_server: &Res<AssetServer>,
               texture_atlases: &mut ResMut<Assets<TextureAtlas>>) {
    let (texture_path, location, directions) = match ghost {
        Ghost::Blinky => ("blinky_body.png",
                          Location::new(13.5, 19.0),
                          GhostDirections::new(Direction::Left)),
        Ghost::Pinky => ("pinky_body.png",
                         Location::new(13.5, 16.0),
                         GhostDirections::new(Direction::Down)),
        Ghost::Inky => ("inky_body.png",
                        Location::new(11.5, 16.0),
                        GhostDirections::new(Direction::Up)),
        Ghost::Clyde => ("clyde_body.png",
                         Location::new(15.5, 16.0),
                         GhostDirections::new(Direction::Up)),
    };

    commands.spawn((
            location,
            ghost,
            directions,
            CharacterSpeed::new(0.75),
            if let Ghost::Blinky | Ghost::Pinky = ghost { GhostMode::HomeExit(false) } else { GhostMode::Home(false) },
            SpatialBundle::default()))
    .with_children(|parent| {
        for ghost_sprite in GhostSprite::iter() {
            let (png, number_of_sprites, layer) = match ghost_sprite {
                GhostSprite::Body => (texture_path, 2, Layers::Ghosts),
                GhostSprite::Eyes => ("ghost_eyes.png", 4, Layers::GhostsEyes),
                GhostSprite::Frightened => ("ghosts_frite.png", 4, Layers::Ghosts)
            };

            let texture_handle = asset_server.load(png);
            let texture_atlas =
                TextureAtlas::from_grid(texture_handle, Vec2::new(16.0, 16.0), number_of_sprites, 1, None, None);
            let texture_atlas_handle = texture_atlases.add(texture_atlas);

            parent.spawn((SpriteSheetBundle {
                texture_atlas: texture_atlas_handle,
                sprite: TextureAtlasSprite::new(0),
                transform: Transform::from_xyz(0.0, 0.0, layer.as_f32()),
                ..default()
            },
            ghost_sprite));
        }
    });
}

fn init_resources(mut global_ghost_mode: ResMut<GhostMode>,
                  mut global_mode_timer: ResMut<GlobalGhostModeTimer>,
                  mut pellet_eaten_counter: ResMut<GhostPelletEatenCounter>) {
    *global_ghost_mode = GhostMode::Scatter;

    global_mode_timer.timer.set_duration(Duration::from_secs(CHANGE_DURATIONS[0]));
    global_mode_timer.timer.reset();
    global_mode_timer.duration_index = 0;

    pellet_eaten_counter.counter = 0;
    pellet_eaten_counter.life_lost = false;
}

fn timer_pause(pause_timer: Res<CollisionPauseTimer>,
               mut frite_timer: ResMut<FriteTimer>,
               mut global_mode_timer: ResMut<GlobalGhostModeTimer>) {
    if pause_timer.0.finished() {
        frite_timer.0.unpause();
        global_mode_timer.timer.unpause();
    } else {
        frite_timer.0.pause();
        global_mode_timer.timer.pause();
    }
}

fn update_ghost_mode(mut query: Query<(&mut GhostMode, &mut GhostDirections, &Location, &Ghost)>,
                     global_ghost_mode: Res<GhostMode>,
                     mut pellet_eaten_events: EventReader<PelletEaten>,
                     mut ghost_pellet_eaten_counter: ResMut<GhostPelletEatenCounter>,
                     mut collision_events: EventReader<Collision>,
                     mut frite_timer: ResMut<FriteTimer>,
                     pause_timer: Res<CollisionPauseTimer>,
                     time: Res<Time>) {
    ghost_pellet_eaten_counter.counter += pellet_eaten_events.len();
    let power_pellet_eaten = pellet_eaten_events.read().find(|event| event.power).is_some();

    let frite_timer_finished = frite_timer.0.tick(time.delta()).just_finished();
    if power_pellet_eaten {
        frite_timer.0.reset();
        frite_timer.0.set_duration(Duration::from_secs(6));
    }

    let collided_ghosts = collision_events.read().map(|event| event.ghost).collect::<Vec<_>>();

    query.par_iter_mut().for_each(|(mut mode, mut directions, location, ghost)| {
        if power_pellet_eaten {
            let prev_mode = *mode;
            *mode = match *mode {
                GhostMode::Home(_) => GhostMode::Home(true),
                GhostMode::HomeExit(_) => GhostMode::HomeExit(true),
                GhostMode::DeadPause => GhostMode::DeadPause,
                GhostMode::Dead(enter_home) => GhostMode::Dead(enter_home),
                _ => GhostMode::Frightened,
            };

            if prev_mode != *mode {
                directions.reverse();
            }
            return;
        }

        match *mode {
            GhostMode::Frightened => {
                if collided_ghosts.contains(ghost) {
                    *mode = GhostMode::DeadPause;
                } else if frite_timer_finished {
                    *mode = *global_ghost_mode;
                }
            },
            GhostMode::DeadPause => {
                if pause_timer.0.finished() {
                    *mode = GhostMode::Dead(false);
                }
            },
            GhostMode::Dead(false) => {
                if *location == Location::new(13.5, 19.0) {
                    *mode = GhostMode::Dead(true);
                }
            }
            GhostMode::Dead(true) => {
                if *location == Location::new(13.5, 16.0) {
                    *mode = GhostMode::HomeExit(false);
                }
            },
            GhostMode::Home(mut frightened) => {
                if frite_timer_finished {
                    *mode = GhostMode::Home(false);
                    frightened = false;
                }

                match *ghost {
                    Ghost::Inky if ghost_pellet_eaten_counter.counter >= 30 => {
                        *mode = GhostMode::HomeExit(frightened);
                    },
                    Ghost::Clyde if ghost_pellet_eaten_counter.counter >= 90 => {
                        *mode = GhostMode::HomeExit(frightened);
                    },
                    _ => (),
                }
            },
            GhostMode::HomeExit(mut frightened) => {
                if frite_timer_finished {
                    frightened = false;
                    *mode = GhostMode::HomeExit(false);
                }
                if location.y == 19.0 {
                    directions.current = Direction::Left;
                    directions.planned = Some(Direction::Left);

                    *mode = if frightened {
                        GhostMode::Frightened
                    } else {
                        *global_ghost_mode
                    };
                }
            },
            _ if *mode != *global_ghost_mode => {
                *mode = *global_ghost_mode;
                directions.reverse();
            },
            _ => (),
        }
    });
}

fn update_ghost_speed(mut query: Query<(&mut CharacterSpeed, &GhostMode, &Location, &Ghost)>,
                      pellets_eaten_counter: Res<GhostPelletEatenCounter>,
                      total_pellets: Res<TotalPellets>,
                      pause_timer: Res<CollisionPauseTimer>) {
    query.par_iter_mut().for_each(|(mut speed, mode, location, ghost)| {
        let in_tunnel = location.y == 16.0 && (location.x <= 5.0 || location.x >= 22.0);
        
        let mode_speed = if let GhostMode::Dead(_) = *mode {
            1.05
        } else if !pause_timer.0.finished() {
            0.0
        } else if in_tunnel {
            0.4
        } else {
            let remaining_pellets = total_pellets.0 - pellets_eaten_counter.counter;
            match *mode {
                GhostMode::Frightened => 0.5,
                GhostMode::Home(_) | GhostMode::HomeExit(_) => 0.5,
                // Elroy!!!!!
                _ if matches!(*ghost, Ghost::Blinky) && remaining_pellets <= 10 => 0.85,
                _ if matches!(*ghost, Ghost::Blinky) && remaining_pellets <= 20 => 0.8,
                _ => 0.75,
            }
        };

        speed.set_speed(mode_speed);
        speed.tick();
    });
}

fn ghost_tile_change_detection(mut query: Query<(&Location, &mut GhostDirections, &CharacterSpeed), With<Ghost>>) {
    query.par_iter_mut().for_each(|(location, mut directions, speed)| {
        if speed.should_miss {
            return;
        }
        if location.is_tile_center() {
            directions.advance();
        }
    });
}

fn plan_ghosts(mut query: Query<(&Location, &mut GhostDirections, &Ghost, &GhostMode), Without<Player>>,
               player_query: Query<(&Location, &Direction), With<Player>>,
               map: Res<Map>) {
    let map = &*map;
    let (player_location, player_direction) = player_query.single();
    let player_tile = player_location.get_tile(*player_direction);

    let mut blinky_tile = Location::new(0.0, 0.0);
    for (location, directions, ghost, _) in query.iter_mut() {
        if let Ghost::Blinky = *ghost {
            blinky_tile = location.get_tile(directions.current);
            break;
        }
    }

    query.par_iter_mut().for_each(|(location, mut directions, ghost, mode)| {
        if let GhostMode::Home(_) |
                GhostMode::HomeExit(_) |
                GhostMode::Dead(true) |
                GhostMode::DeadPause = *mode {
            return;
        }

        if !directions.plan_needed() {
            return;
        }

        if !map.is_in_map(*location) {
            let current_direction = directions.current;
            directions.set_plan(current_direction);
            return;
        }

        let target_tile = match *mode {
            GhostMode::Scatter => Some(scatter(*ghost)),
            GhostMode::Chase => Some(chase_target(*ghost,
                                             location.get_tile(directions.current),
                                             blinky_tile,
                                             player_tile,
                                             *player_direction)),
            GhostMode::Frightened => None,
            GhostMode::Dead(false) => Some(Location::new(13.5, 19.0)),
            GhostMode::Home(_) |
                GhostMode::HomeExit(_) |
                GhostMode::Dead(true) | 
                GhostMode::DeadPause => unreachable!(),
        };

        let next_tile = location.next_tile(directions.current);
        let in_special_zone = 10.0 <= location.x && location.x <= 17.0 && (location.y == 7.0 || location.y == 19.0);

        if GHOST_DEBUG {
            println!("Directions: {:?}" , directions);
            map.print_7x7(location.get_tile(directions.current), next_tile);
        }

        let planned_direction = ghost_path_finder(next_tile,
                                                  target_tile,
                                                  map,
                                                  directions.current,
                                                  in_special_zone);

        directions.set_plan(planned_direction);
    });
}

fn scatter(ghost: Ghost) -> Location {
    match ghost {
        Ghost::Blinky => Location::new(25.0, 33.0),
        Ghost::Pinky => Location::new(2.0, 33.0),
        Ghost::Inky => Location::new(27.0, -1.0),
        Ghost::Clyde => Location::new(0.0, -1.0),
    }
}

fn chase_target(ghost: Ghost,
                current_tile: Location,
                blinky_tile: Location,
                player_tile: Location,
                player_direction: Direction) -> Location {
    match ghost {
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
                scatter(ghost)
            }
        },
    }
}

fn ghost_path_finder(next_tile: Location,
                     target_tile: Option<Location>,
                     map: &Map,
                     current_direction: Direction,
                     is_in_special_zone: bool) -> Direction {
    let mut possible_directions = map.possible_directions(next_tile);

    possible_directions.retain(|direction| {
        if is_in_special_zone && *direction == Direction::Up {
            return false;
        }

        *direction != current_direction.opposite()
    });

    if let Some(target_tile) = target_tile {
        possible_directions.sort_by(|direction1, direction2| {
            let tile1 = next_tile.next_tile(*direction1);
            let tile2 = next_tile.next_tile(*direction2);

            let distance1 = (tile1 - target_tile).length_squared();
            let distance2 = (tile2 - target_tile).length_squared();

            distance1.partial_cmp(&distance2).unwrap()
        });

        possible_directions[0]
    } else {
        let direction_index = fastrand::usize(0..possible_directions.len());
        possible_directions[direction_index]
    }
}

fn move_ghosts(mut query: Query<(&mut Location, &mut GhostDirections, &GhostMode, &Ghost, &CharacterSpeed)>,
               next_game_state: Res<NextState<AppState>>) {
    query.par_iter_mut().for_each(|(mut location, mut directions, mode, ghost, speed)| {
        if speed.should_miss || next_game_state.0.is_some() {
            return;
        }

        match *mode {
            GhostMode::Home(_) => {
                match ghost {
                    Ghost::Pinky => location.x = 13.5,
                    Ghost::Inky => location.x = 11.5,
                    Ghost::Clyde => location.x = 15.5,
                    Ghost::Blinky => unreachable!()
                }

                if location.y >= 16.5 {
                    directions.current = Direction::Down;
                } else if location.y <= 15.5 {
                    directions.current = Direction::Up;
                }
            },
            GhostMode::HomeExit(_) => {
                match *ghost {
                    Ghost::Blinky => {
                        debug_assert!(location.x == 13.5);
                        debug_assert!(location.y >= 15.5 && location.y <= 19.0);

                        directions.current = Direction::Up;
                    }
                    Ghost::Pinky => {
                        debug_assert!(location.x == 13.5);
                        debug_assert!(location.y >= 15.5 && location.y <= 19.0);

                        directions.current = Direction::Up;
                    }
                    Ghost::Inky => {
                        debug_assert!(location.y >= 15.5 && location.y <= 19.0);

                        if location.x != 13.5 {
                            directions.current = Direction::Right;
                        } else {
                            directions.current = Direction::Up;
                        }
                    },
                    Ghost::Clyde => {
                        debug_assert!(location.y >= 15.5 && location.y <= 19.0);

                        if location.x != 13.5 {
                            directions.current = Direction::Left;
                        } else {
                            directions.current = Direction::Up;
                        }
                    },
                }
            },
            GhostMode::Dead(true) => directions.current = Direction::Down,
            _ => (),
        }

        location.advance(directions.current);
    });
}

fn draw_ghosts(mut query: Query<(&GhostDirections,
                                 &Location,
                                 &GhostMode,
                                 &mut Visibility,
                                 &Children),
                                 With<Ghost>>,
               mut sprites_query: Query<(&mut TextureAtlasSprite, 
                                      &mut Visibility,
                                      &GhostSprite),
                                      Without<Ghost>>,
              frite_timer: Res<FriteTimer>) {
    for (directions, location, mode, mut visibility, children) in query.iter_mut() {

        if let GhostMode::DeadPause = *mode {
            *visibility = Visibility::Hidden;
            continue;
        } else {
            *visibility = Visibility::Inherited;
        }

        for child in children.iter() {
            let (mut sprite, mut visibility, sprite_type) = sprites_query.get_mut(*child).expect("Ghost without sprite");
            let is_frightened = matches!(*mode, GhostMode::Frightened | GhostMode::Home(true) | GhostMode::HomeExit(true));

            let change_variation = match *mode {
                GhostMode::Home(_) | GhostMode::HomeExit(_) => location.y.fract() == 0.5,
                _ => location.is_tile_center(),
            };
            let variation = (sprite.index + if change_variation { 1 } else { 0 }) % 2;

            match sprite_type {
                GhostSprite::Body => {
                    if is_frightened || matches!(*mode, GhostMode::Dead(_)) {
                        *visibility = Visibility::Hidden;
                    } else {
                        *visibility = Visibility::Inherited;

                        sprite.index = variation;
                    }
                },
                GhostSprite::Eyes => {
                    if is_frightened {
                        *visibility = Visibility::Hidden;
                    } else {
                        *visibility = Visibility::Inherited;

                        let rotation = (directions.current.rotation() * 4.0) as usize;
                        sprite.index = rotation;
                    }
                },
                GhostSprite::Frightened => {
                    if !is_frightened {
                        *visibility = Visibility::Hidden;
                    } else {
                        *visibility = Visibility::Inherited;

                        let remaining_time = frite_timer.0.remaining_secs();

                        const FLASHING_TIMING: f32 = 1.0 / 2.0;
                        const START_FLASHING_TIME: f32 = FLASHING_TIMING * 5.0;
                        let flashing = if remaining_time > START_FLASHING_TIME {
                            false
                        } else {
                            let cycle = (remaining_time % FLASHING_TIMING) / FLASHING_TIMING;
                            cycle > 0.5
                        };

                        sprite.index = variation + if flashing { 2 } else { 0 };
                    }
                },
            }
        }
    }
}

fn update_global_ghost_mode(mut global_ghost_mode: ResMut<GhostMode>,
                       mut mode: ResMut<GlobalGhostModeTimer>,
                       time: Res<Time>) {
    if !mode.timer.tick(time.delta()).just_finished() {
        return;
    }

    *global_ghost_mode = match *global_ghost_mode {
        GhostMode::Chase => GhostMode::Scatter,
        GhostMode::Scatter => GhostMode::Chase,
        _ => unreachable!(),
    };

    mode.duration_index += 1;
    if let Some(duration) = CHANGE_DURATIONS.get(mode.duration_index) {
        mode.timer.set_duration(Duration::from_secs(*duration));
        mode.timer.reset();
    }
}

fn collision_detection(query: Query<(&Location, &Ghost, &GhostMode)>,
                       player_query: Query<&Location, With<Player>>,
                       mut collision_events: EventWriter<Collision>) {
    let player_location = player_query.single();

    for (location, ghost, mode) in query.iter() {
        let location_dif = *location - *player_location;
        let distance_squared = location_dif.length_squared();
        if distance_squared < 0.5 * 0.5 {
            collision_events.send(Collision { ghost: *ghost, mode: *mode });
        }
    }
}

fn despawn_ghosts(mut commands: Commands,
                  query: Query<Entity, With<Ghost>>,
                  timer: Res<StateTimer>) {
    if timer.0.elapsed_secs() >= 3.0 {
        for entity in query.iter() {
            commands.entity(entity).despawn_recursive();
        }
    }
}
