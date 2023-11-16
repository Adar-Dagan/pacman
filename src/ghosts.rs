use std::time::Duration;

use bevy::prelude::*;
use strum::{ EnumIter, IntoEnumIterator };

use crate::common::events::PelletEaten;
use crate::common::layers::Layers;
use crate::common::sets::GameLoop;
use crate::player::Player;
use crate::services::map::{Direction, Map, Location};

#[derive(Component, EnumIter, Clone, Copy, PartialEq)]
enum Ghost {
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
    //TODO: Temporary initialization to be removed when ghost initial state is implemented
    fn new() -> Self {
        Self {
            current: Direction::Left,
            planned: Some(Direction::Left),
        }
    }

    fn advance(&mut self) {
        self.current = self.planned.expect("Advanced with no planned direction");
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
enum GhostMode {
    Chase,
    Scatter,
    Frightened,
    Dead,
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

#[derive(Event)]
struct Collision {
    ghost: Ghost,
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
        app.add_event::<Collision>();
        app.add_systems(Startup, spawn_ghosts);
        app.add_systems(FixedUpdate, (update_global_ghost_mode.before(GameLoop::Planning),
                                      (update_ghost_mode, ghost_tile_change_detection, plan_ghosts)
                                                 .chain()
                                                 .in_set(GameLoop::Planning),
                                      move_ghosts.in_set(GameLoop::Movement), 
                                      draw_ghosts.after(GameLoop::Movement),
                                      collision_detection.after(GameLoop::Collisions)));
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
        Ghost::Blinky => "blinky_body.png",
        Ghost::Pinky => "pinky_body.png",
        Ghost::Inky => "inky_body.png",
        Ghost::Clyde => "clyde_body.png",
    };

    commands.spawn((
            Location::new(13.0, 19.0),
            ghost,
            GhostDirections::new(),
            GhostMode::Scatter,
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

fn update_ghost_mode(mut query: Query<(&mut GhostMode, &mut GhostDirections, &Location, &Ghost)>,
                     global_ghost_mode: Res<GhostMode>,
                     mut pellet_eaten_events: EventReader<PelletEaten>,
                     mut collision_events: EventReader<Collision>,
                     mut frite_timer: ResMut<FriteTimer>,
                     time: Res<Time>) {
    let power_pellet_eaten = pellet_eaten_events.read().find(|event| event.power).is_some();
    let frite_timer_finished = frite_timer.0.tick(time.delta()).just_finished();
    if power_pellet_eaten {
        frite_timer.0.reset();
        frite_timer.0.set_duration(Duration::from_secs(7));
    }
    let collided_ghosts = collision_events.read().map(|event| event.ghost).collect::<Vec<_>>();
    query.par_iter_mut().for_each(|(mut mode, mut directions, location, ghost)| {
        if power_pellet_eaten {
            if !matches!(*mode, GhostMode::Frightened) {
                directions.reverse();
            }
            *mode = GhostMode::Frightened;

            return;
        }

        match *mode {
            GhostMode::Frightened => {
                if collided_ghosts.contains(ghost) {
                    *mode = GhostMode::Dead;
                } else if frite_timer_finished {
                    *mode = *global_ghost_mode;
                }
            },
            GhostMode::Dead => {
                if *location == Location::new(13.0, 19.0) {
                    *mode = *global_ghost_mode;
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

fn ghost_tile_change_detection(mut query: Query<(&Location, &mut GhostDirections), With<Ghost>>) {
    query.par_iter_mut().for_each(|(location, mut directions)| {
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
            GhostMode::Dead => Some(Location::new(13.0, 19.0)),
        };

        let next_tile = location.next_tile(directions.current);
        let planned_direction = ghost_path_finder(next_tile,
                                                  target_tile,
                                                  map,
                                                  directions.current);

        directions.set_plan(planned_direction);
    });
}

fn scatter(ghost: Ghost) -> Location {
    match ghost {
        Ghost::Blinky => Location::new(28.0, 30.0),
        Ghost::Pinky => Location::new(2.0, 30.0),
        Ghost::Inky => Location::new(30.0, -1.0),
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
                     current_direction: Direction) -> Direction {
    let mut possible_directions = map.possible_directions(next_tile);

    possible_directions.retain(|direction| {
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

fn move_ghosts(mut query: Query<(&mut Location, &GhostDirections), With<Ghost>>) {
    query.par_iter_mut().for_each(|(mut location, directions)| {
        location.advance(directions.current);
    });
}

fn draw_ghosts(mut query: Query<(&GhostDirections,
                                 &Location,
                                 &GhostMode,
                                 &Children),
                                 With<Ghost>>,
               mut eyes_query: Query<(&mut TextureAtlasSprite, 
                                      &mut Visibility,
                                      &GhostSprite),
                                      Without<Ghost>>,
              frite_timer: Res<FriteTimer>) {
    for (directions, location, mode, children) in query.iter_mut() {
        for child in children.iter() {
            let (mut sprite, mut visibility, sprite_type) = eyes_query.get_mut(*child).expect("Ghost without sprite");
            match sprite_type {
                GhostSprite::Body => {
                    if let GhostMode::Chase | GhostMode::Scatter = *mode {
                        *visibility = Visibility::Inherited;
                    } else {
                        *visibility = Visibility::Hidden;
                    }

                    if location.is_tile_center() {
                        sprite.index = (sprite.index + 1) % 2;
                    }
                },
                GhostSprite::Eyes => {
                    if let GhostMode::Frightened = *mode {
                        *visibility = Visibility::Hidden;
                    } else {
                        *visibility = Visibility::Inherited;
                    }

                    if location.is_tile_center() {
                        let rotation = (directions.current.rotation() * 4.0) as usize;
                        sprite.index = rotation;
                    }
                },
                GhostSprite::Frightened => {
                    if let GhostMode::Frightened = *mode {
                        *visibility = Visibility::Inherited;
                    } else {
                        *visibility = Visibility::Hidden;
                    }

                    let remaining_time = frite_timer.0.remaining_secs();

                    if location.is_tile_center() {
                        let variation = (sprite.index + 1) % 2;
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

fn collision_detection(query: Query<(&Location, &Ghost)>,
                       player_query: Query<&Location, With<Player>>,
                       mut collision_events: EventWriter<Collision>) {
    let player_location = player_query.single();

    for (location, ghost) in query.iter() {
        let location_dif = *location - *player_location;
        let distance_squared = location_dif.length_squared();
        if distance_squared < 0.5 * 0.5 {
            collision_events.send(Collision { ghost: *ghost });
        }
    }
}
