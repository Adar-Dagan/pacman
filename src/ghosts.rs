use std::time::Duration;

use bevy::prelude::*;
use bevy_kira_audio::prelude::*;
use strum::{EnumIter, IntoEnumIterator};

use crate::advance_level;
use crate::common::app_state::{AppState, DeadState, StateTimer};
use crate::common::events::{CollisionPauseTimer, GhostEaten, PelletEaten};
use crate::common::layers::Layers;
use crate::common::levels::Levels;
use crate::common::sets::GameLoop;
use crate::pellets::TotalPellets;
use crate::player::Player;
use crate::services::map::{Direction, Location, Map};
use crate::services::speed::CharacterSpeed;

const GHOST_DEBUG: bool = false;

#[derive(Resource)]
pub struct FriteTimer(pub Timer);

#[derive(Resource, Default)]
struct ExitHomeTimer(Timer);

#[derive(Resource, Default)]
struct GlobalGhostModeTimer {
    timer: Timer,
    duration_index: usize,
}

#[derive(Resource, Default)]
struct GhostPelletEatenCounter {
    counter: usize,
    life_lost: bool,
}

#[derive(Resource, Component, Debug, Clone, Copy, PartialEq, Default)]
pub enum GhostMode {
    Home(bool),
    HomeExit(bool),
    Chase,
    #[default]
    Scatter,
    Frightened,
    DeadPause,
    Dead,
    DeadEnterHome,
}

#[derive(Component, EnumIter)]
enum GhostSprite {
    Body,
    Eyes,
    Frightened,
}

#[derive(Component, EnumIter, Clone, Copy, PartialEq, Debug)]
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

#[derive(Bundle)]
struct GhostBundle {
    location: Location,
    ghost: Ghost,
    directions: GhostDirections,
    speed: CharacterSpeed,
    mode: GhostMode,
}

pub struct GhostPlugin;

impl Plugin for GhostPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GhostMode::default());
        app.insert_resource(GlobalGhostModeTimer::default());
        app.insert_resource(FriteTimer(Timer::from_seconds(0.0, TimerMode::Once)));
        app.insert_resource(GhostPelletEatenCounter::default());
        app.insert_resource(ExitHomeTimer(Timer::from_seconds(
            0.0,
            TimerMode::Repeating,
        )));

        app.add_systems(
            OnEnter(AppState::LevelStart),
            (init_level_resources.after(advance_level), spawn_ghosts).chain(),
        );
        app.add_systems(
            OnEnter(DeadState::Restart),
            (reset_resources_on_death, spawn_ghosts).chain(),
        );

        app.add_systems(FixedUpdate, ghost_eaten_system.before(GameLoop::Planning));
        app.add_systems(
            FixedUpdate,
            (
                timer_pause,
                update_global_ghost_mode,
                update_ghost_mode,
                detect_power_pellet,
                update_ghost_speed,
                ghost_tile_change_detection,
                plan_ghosts,
            )
                .chain()
                .in_set(GameLoop::Planning),
        );
        app.add_systems(FixedUpdate, move_ghosts.in_set(GameLoop::Movement));
        app.add_systems(
            FixedUpdate,
            collision_detection.in_set(GameLoop::Collisions),
        );

        app.add_systems(
            Update,
            despawn_ghosts.run_if(in_state(AppState::LevelComplete).and_then(despawn_timer_check)),
        );
        app.add_systems(OnEnter(AppState::GameOver), despawn_ghosts);
        app.add_systems(OnEnter(DeadState::Animation), despawn_ghosts);

        app.add_systems(
            Update,
            draw_ghosts.run_if(
                in_state(AppState::MainGame)
                    .or_else(in_state(AppState::LevelStart))
                    .or_else(in_state(DeadState::Restart)),
            ),
        );
    }
}

fn spawn_ghosts(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    pellets_eaten_counter: Res<GhostPelletEatenCounter>,
) {
    if GHOST_DEBUG {
        spawn_ghost(
            Ghost::Blinky,
            &mut commands,
            &asset_server,
            &mut texture_atlases,
            false,
        );
    } else {
        for ghost in Ghost::iter() {
            spawn_ghost(
                ghost,
                &mut commands,
                &asset_server,
                &mut texture_atlases,
                pellets_eaten_counter.life_lost,
            );
        }
    }
}

fn spawn_ghost(
    ghost: Ghost,
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    texture_atlases: &mut ResMut<Assets<TextureAtlas>>,
    life_lost: bool,
) {
    let (texture_path, location, directions, mode) = match ghost {
        Ghost::Blinky => (
            "blinky_body.png",
            Location::new(13.5, 19.0),
            GhostDirections::new(Direction::Left),
            GhostMode::HomeExit(false),
        ),
        Ghost::Pinky => (
            "pinky_body.png",
            Location::new(13.5, 16.0),
            GhostDirections::new(Direction::Down),
            if life_lost {
                GhostMode::Home(false)
            } else {
                GhostMode::HomeExit(false)
            },
        ),
        Ghost::Inky => (
            "inky_body.png",
            Location::new(11.5, 16.0),
            GhostDirections::new(Direction::Up),
            GhostMode::Home(false),
        ),
        Ghost::Clyde => (
            "clyde_body.png",
            Location::new(15.5, 16.0),
            GhostDirections::new(Direction::Up),
            GhostMode::Home(false),
        ),
    };

    commands
        .spawn((
            GhostBundle {
                location,
                ghost,
                directions,
                speed: CharacterSpeed::new(0.75),
                mode,
            },
            SpatialBundle::default(),
        ))
        .with_children(|parent| {
            for ghost_sprite in GhostSprite::iter() {
                let (png_path, number_of_sprites, layer) = match ghost_sprite {
                    GhostSprite::Body => (texture_path, 2, Layers::Ghosts),
                    GhostSprite::Eyes => ("ghost_eyes.png", 4, Layers::GhostsEyes),
                    GhostSprite::Frightened => ("ghosts_frite.png", 4, Layers::Ghosts),
                };

                let texture_handle = asset_server.load(png_path);
                let texture_atlas = TextureAtlas::from_grid(
                    texture_handle,
                    Vec2::new(16.0, 16.0),
                    number_of_sprites,
                    1,
                    None,
                    None,
                );
                let texture_atlas_handle = texture_atlases.add(texture_atlas);

                parent.spawn((
                    SpriteSheetBundle {
                        texture_atlas: texture_atlas_handle,
                        sprite: TextureAtlasSprite::new(0),
                        transform: Transform::from_xyz(0.0, 0.0, layer.as_f32()),
                        ..default()
                    },
                    ghost_sprite,
                ));
            }
        });
}

fn init_level_resources(
    mut global_ghost_mode: ResMut<GhostMode>,
    mut global_mode_timer: ResMut<GlobalGhostModeTimer>,
    mut pellet_eaten_counter: ResMut<GhostPelletEatenCounter>,
    mut exit_home_timer: ResMut<ExitHomeTimer>,
    levels: Res<Levels>,
) {
    *global_ghost_mode = GhostMode::Scatter;

    global_mode_timer
        .timer
        .set_duration(Duration::from_secs_f32(
            levels.ghost_switch_global_mode(0).unwrap(),
        ));
    global_mode_timer.timer.reset();
    global_mode_timer.duration_index = 0;

    pellet_eaten_counter.counter = 0;
    pellet_eaten_counter.life_lost = false;

    exit_home_timer
        .0
        .set_duration(Duration::from_secs(levels.ghost_exit_home_duration()));
    exit_home_timer.0.reset();
}

fn reset_resources_on_death(
    mut pellet_eaten_counter: ResMut<GhostPelletEatenCounter>,
    mut exit_home_timer: ResMut<ExitHomeTimer>,
) {
    pellet_eaten_counter.life_lost = true;
    pellet_eaten_counter.counter = 0;

    exit_home_timer.0.reset();
}

fn timer_pause(
    pause_timer: Res<CollisionPauseTimer>,
    mut frite_timer: ResMut<FriteTimer>,
    mut exit_home_timer: ResMut<ExitHomeTimer>,
    mut global_mode_timer: ResMut<GlobalGhostModeTimer>,
) {
    if pause_timer.0.finished() {
        frite_timer.0.unpause();
        exit_home_timer.0.unpause();
    } else {
        frite_timer.0.pause();
        exit_home_timer.0.pause();
    }

    if pause_timer.0.finished() && frite_timer.0.finished() {
        global_mode_timer.timer.unpause();
    } else {
        global_mode_timer.timer.pause();
    }
}

fn update_ghost_mode(
    mut query: Query<(&mut GhostMode, &mut GhostDirections, &Location, &Ghost)>,
    global_ghost_mode: Res<GhostMode>,
    mut pellet_eaten_events: EventReader<PelletEaten>,
    mut ghost_pellet_eaten_counter: ResMut<GhostPelletEatenCounter>,
    mut ghost_eaten_events: EventReader<GhostEaten>,
    mut frite_timer: ResMut<FriteTimer>,
    mut exit_home_timer: ResMut<ExitHomeTimer>,
    pause_timer: Res<CollisionPauseTimer>,
    levels: Res<Levels>,
    time: Res<Time>,
) {
    let frite_timer_finished = frite_timer.0.tick(time.delta()).just_finished();

    ghost_pellet_eaten_counter.counter += pellet_eaten_events.len();

    if pellet_eaten_events.read().count() != 0 {
        exit_home_timer.0.reset();
    }
    let exit_home_timer_finished = exit_home_timer.0.tick(time.delta()).just_finished();

    if exit_home_timer_finished {
        ghost_pellet_eaten_counter.counter = 0;
    }

    let eaten_ghosts = ghost_eaten_events
        .read()
        .map(|event| event.ghost)
        .collect::<Vec<_>>();

    let (inky_is_in_home, pinky_is_in_home) = query
        .iter()
        .filter(|(_, _, _, ghost)| matches!(ghost, Ghost::Inky | Ghost::Pinky))
        .map(|(mode, _, _, ghost)| (matches!(*mode, GhostMode::Home(_)), ghost))
        .fold((false, false), |acc, (in_home, ghost)| match ghost {
            Ghost::Inky => (in_home, acc.1),
            Ghost::Pinky => (acc.0, in_home),
            _ => unreachable!(),
        });

    for (mut mode, mut directions, location, ghost) in query.iter_mut() {
        match *mode {
            GhostMode::Frightened => {
                if eaten_ghosts.contains(ghost) {
                    *mode = GhostMode::DeadPause;
                } else if frite_timer_finished {
                    *mode = *global_ghost_mode;
                }
            }
            GhostMode::DeadPause => {
                if pause_timer.0.finished() {
                    *mode = GhostMode::Dead;
                }
            }
            GhostMode::Dead => {
                if *location == Location::new(13.5, 19.0) {
                    *mode = GhostMode::DeadEnterHome;
                }
            }
            GhostMode::DeadEnterHome => {
                if *location == Location::new(13.5, 16.0) {
                    *mode = GhostMode::HomeExit(false);
                }
            }
            GhostMode::Home(mut frightened) => {
                if frite_timer_finished {
                    *mode = GhostMode::Home(false);
                    frightened = false;
                }

                let can_leave = match *ghost {
                    Ghost::Blinky => unreachable!(),
                    Ghost::Pinky => true,
                    Ghost::Inky => !pinky_is_in_home,
                    Ghost::Clyde => !pinky_is_in_home && !inky_is_in_home,
                };

                if can_leave
                    && (ghost_pellet_eaten_counter.counter
                        >= levels.home_exit_dots(*ghost, ghost_pellet_eaten_counter.life_lost)
                        || exit_home_timer_finished)
                {
                    *mode = GhostMode::HomeExit(frightened);
                    ghost_pellet_eaten_counter.counter = 0;
                }
            }
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
            }
            _ if *mode != *global_ghost_mode => {
                *mode = *global_ghost_mode;
                directions.reverse();
            }
            _ => (),
        }
    }
}

fn detect_power_pellet(
    mut query: Query<(&mut GhostMode, &mut GhostDirections), With<Ghost>>,
    mut frite_timer: ResMut<FriteTimer>,
    mut pellet_eaten_events: EventReader<PelletEaten>,
    levels: Res<Levels>,
) {
    let power_pellet_eaten = pellet_eaten_events
        .read()
        .find(|event| event.power)
        .is_some();

    if power_pellet_eaten {
        frite_timer.0.reset();
        frite_timer
            .0
            .set_duration(Duration::from_secs(levels.frite_duration()));

        for (mut mode, mut directions) in query.iter_mut() {
            let prev_mode = *mode;
            *mode = match *mode {
                GhostMode::Home(_) => GhostMode::Home(true),
                GhostMode::HomeExit(_) => GhostMode::HomeExit(true),
                GhostMode::DeadPause => GhostMode::DeadPause,
                GhostMode::Dead => GhostMode::Dead,
                GhostMode::DeadEnterHome => GhostMode::DeadEnterHome,
                _ => GhostMode::Frightened,
            };

            if prev_mode != *mode {
                directions.reverse();
            }
        }
    }
}

fn update_ghost_speed(
    mut query: Query<(&mut CharacterSpeed, &GhostMode, &Location, &Ghost)>,
    pellets_eaten_counter: Res<GhostPelletEatenCounter>,
    total_pellets: Res<TotalPellets>,
    pause_timer: Res<CollisionPauseTimer>,
    levels: Res<Levels>,
) {
    query
        .par_iter_mut()
        .for_each(|(mut speed, mode, location, ghost)| {
            let in_tunnel = location.y == 16.0 && (location.x <= 5.0 || location.x >= 22.0);

            let mode_speed = if let GhostMode::Dead | GhostMode::DeadEnterHome = *mode {
                1.05
            } else if !pause_timer.0.finished() {
                0.0
            } else if in_tunnel {
                levels.ghost_tunnel_speed()
            } else {
                let remaining_pellets = total_pellets.0 - pellets_eaten_counter.counter;
                match *mode {
                    GhostMode::Frightened => levels.ghost_frite_speed(),
                    GhostMode::Home(_) | GhostMode::HomeExit(_) => 0.4,
                    // Elroy!!!!!
                    _ if matches!(*ghost, Ghost::Blinky)
                        && remaining_pellets <= levels.elroy_2_dots() =>
                    {
                        levels.elroy_2_speed()
                    }
                    _ if matches!(*ghost, Ghost::Blinky)
                        && remaining_pellets <= levels.elroy_1_dots() =>
                    {
                        levels.elroy_1_speed()
                    }
                    _ => levels.ghost_normal_speed(),
                }
            };

            speed.set_speed(mode_speed);
            speed.tick();
        });
}

fn ghost_tile_change_detection(
    mut query: Query<(&Location, &mut GhostDirections, &CharacterSpeed), With<Ghost>>,
) {
    query
        .par_iter_mut()
        .for_each(|(location, mut directions, speed)| {
            if speed.should_miss {
                return;
            }
            if location.is_tile_center() {
                directions.advance();
            }
        });
}

fn plan_ghosts(
    mut query: Query<(&Location, &mut GhostDirections, &Ghost, &GhostMode), Without<Player>>,
    player_query: Query<(&Location, &Direction), With<Player>>,
    map: Res<Map>,
) {
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

    query
        .par_iter_mut()
        .for_each(|(location, mut directions, ghost, mode)| {
            if let GhostMode::Home(_)
            | GhostMode::HomeExit(_)
            | GhostMode::DeadEnterHome
            | GhostMode::DeadPause = *mode
            {
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
                GhostMode::Chase => Some(chase_target(
                    *ghost,
                    location.get_tile(directions.current),
                    blinky_tile,
                    player_tile,
                    *player_direction,
                )),
                GhostMode::Frightened => None,
                GhostMode::Dead => Some(Location::new(13.5, 19.0)),
                GhostMode::Home(_)
                | GhostMode::HomeExit(_)
                | GhostMode::DeadEnterHome
                | GhostMode::DeadPause => unreachable!(),
            };

            let next_tile = location.next_tile(directions.current);
            let in_special_zone = 10.0 <= location.x
                && location.x <= 17.0
                && (location.y == 7.0 || location.y == 19.0);

            let planned_direction = ghost_path_finder(
                next_tile,
                target_tile,
                map,
                directions.current,
                in_special_zone,
            );

            if GHOST_DEBUG || planned_direction.is_none() {
                println!("Directions: {:?}", directions);
                map.print_7x7(location.get_tile(directions.current), next_tile);
            }

            let planned_direction = planned_direction.unwrap();

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

fn chase_target(
    ghost: Ghost,
    current_tile: Location,
    blinky_tile: Location,
    player_tile: Location,
    player_direction: Direction,
) -> Location {
    match ghost {
        Ghost::Blinky => player_tile,
        Ghost::Pinky => player_tile + player_direction.get_vec() * 4.0,
        Ghost::Inky => {
            let offset_tile = player_tile + player_direction.get_vec() * 2.0;
            let blinky_offset_vector = offset_tile - blinky_tile;
            blinky_tile + blinky_offset_vector * 2.0
        }
        Ghost::Clyde => {
            let distance = (player_tile - current_tile).length_squared();
            if distance > 8.0 * 8.0 {
                player_tile
            } else {
                scatter(ghost)
            }
        }
    }
}

fn ghost_path_finder(
    next_tile: Location,
    target_tile: Option<Location>,
    map: &Map,
    current_direction: Direction,
    is_in_special_zone: bool,
) -> Option<Direction> {
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

        possible_directions.get(0).copied()
    } else {
        let range = 0..possible_directions.len();
        if range.is_empty() {
            return None;
        }
        let direction_index = fastrand::usize(range);
        possible_directions.get(direction_index).copied()
    }
}

fn move_ghosts(
    mut query: Query<(
        &mut Location,
        &mut GhostDirections,
        &GhostMode,
        &Ghost,
        &CharacterSpeed,
    )>,
    next_game_state: Res<NextState<AppState>>,
) {
    query
        .par_iter_mut()
        .for_each(|(mut location, mut directions, mode, ghost, speed)| {
            if speed.should_miss || next_game_state.0.is_some() {
                return;
            }

            match *mode {
                GhostMode::Home(_) => {
                    match ghost {
                        Ghost::Pinky => location.x = 13.5,
                        Ghost::Inky => location.x = 11.5,
                        Ghost::Clyde => location.x = 15.5,
                        Ghost::Blinky => unreachable!(),
                    }

                    if location.y >= 16.5 {
                        directions.current = Direction::Down;
                    } else if location.y <= 15.5 {
                        directions.current = Direction::Up;
                    }
                }
                GhostMode::HomeExit(_) => match *ghost {
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
                    }
                    Ghost::Clyde => {
                        debug_assert!(location.y >= 15.5 && location.y <= 19.0);

                        if location.x != 13.5 {
                            directions.current = Direction::Left;
                        } else {
                            directions.current = Direction::Up;
                        }
                    }
                },
                GhostMode::DeadEnterHome => directions.current = Direction::Down,
                _ => (),
            }

            location.advance(directions.current);
        });
}

fn draw_ghosts(
    mut query: Query<
        (
            &GhostDirections,
            &Location,
            &GhostMode,
            &mut Visibility,
            &Children,
        ),
        With<Ghost>,
    >,
    mut sprites_query: Query<
        (&mut TextureAtlasSprite, &mut Visibility, &GhostSprite),
        Without<Ghost>,
    >,
    frite_timer: Res<FriteTimer>,
    levels: Res<Levels>,
    pause_timer: Res<CollisionPauseTimer>,
) {
    for (directions, location, mode, mut visibility, children) in query.iter_mut() {
        if let GhostMode::DeadPause = *mode {
            *visibility = Visibility::Hidden;
            continue;
        } else {
            *visibility = Visibility::Inherited;
        }

        for child in children.iter() {
            let (mut sprite, mut visibility, sprite_type) =
                sprites_query.get_mut(*child).expect("Ghost without sprite");

            let is_frightened = matches!(
                *mode,
                GhostMode::Frightened | GhostMode::Home(true) | GhostMode::HomeExit(true)
            );

            let change_variation = pause_timer.0.finished()
                && match *mode {
                    GhostMode::Home(_) | GhostMode::HomeExit(_) => location.y.fract() == 0.5,
                    _ => location.is_tile_center(),
                };
            let variation = (sprite.index + if change_variation { 1 } else { 0 }) % 2;

            match sprite_type {
                GhostSprite::Body => {
                    if is_frightened || matches!(*mode, GhostMode::Dead | GhostMode::DeadEnterHome)
                    {
                        *visibility = Visibility::Hidden;
                    } else {
                        *visibility = Visibility::Inherited;

                        sprite.index = variation;
                    }
                }
                GhostSprite::Eyes => {
                    if is_frightened {
                        *visibility = Visibility::Hidden;
                    } else {
                        *visibility = Visibility::Inherited;

                        let rotation = (directions.current.rotation() * 4.0) as usize;
                        sprite.index = rotation;
                    }
                }
                GhostSprite::Frightened => {
                    if !is_frightened {
                        *visibility = Visibility::Hidden;
                    } else {
                        *visibility = Visibility::Inherited;

                        let remaining_time = frite_timer.0.remaining_secs();

                        const FLASHING_TIMING: f32 = 1.0 / 4.0;
                        let start_flashing_time: f32 =
                            FLASHING_TIMING * levels.number_of_frite_flashes();
                        let flashing = if remaining_time > start_flashing_time {
                            false
                        } else {
                            let cycle = (remaining_time % FLASHING_TIMING) / FLASHING_TIMING;
                            cycle > 0.5
                        };

                        sprite.index = variation + if flashing { 2 } else { 0 };
                    }
                }
            }
        }
    }
}

fn update_global_ghost_mode(
    mut global_ghost_mode: ResMut<GhostMode>,
    mut mode: ResMut<GlobalGhostModeTimer>,
    time: Res<Time>,
    levels: Res<Levels>,
) {
    if !mode.timer.tick(time.delta()).just_finished() {
        return;
    }

    *global_ghost_mode = match *global_ghost_mode {
        GhostMode::Chase => GhostMode::Scatter,
        GhostMode::Scatter => GhostMode::Chase,
        _ => unreachable!(),
    };

    mode.duration_index += 1;
    if let Some(duration) = levels.ghost_switch_global_mode(mode.duration_index) {
        mode.timer.set_duration(Duration::from_secs_f32(duration));
        mode.timer.reset();
    }
}

fn collision_detection(
    query: Query<(&Location, &Ghost, &GhostMode)>,
    player_query: Query<&Location, With<Player>>,
    mut ghost_eaten_events: EventWriter<GhostEaten>,
    asset_server: Res<AssetServer>,
    audio: Res<Audio>,
    mut next_state: ResMut<NextState<AppState>>,
    mut next_dead_state: ResMut<NextState<DeadState>>,
) {
    let player_location = player_query.single();
    let number_of_fritened_ghosts = query
        .iter()
        .filter(|(_, _, mode)| {
            matches!(
                mode,
                GhostMode::Frightened | GhostMode::Home(true) | GhostMode::HomeExit(true)
            )
        })
        .count();

    for (location, ghost, mode) in query.iter() {
        let location_dif = *location - *player_location;
        let distance_squared = location_dif.length_squared();

        if distance_squared < 0.5 * 0.5 {
            match mode {
                GhostMode::Frightened => {
                    ghost_eaten_events.send(GhostEaten {
                        ghost: *ghost,
                        eaten_ghosts: 4 - number_of_fritened_ghosts,
                    });

                    audio.play(asset_server.load("sounds/eat_ghost.wav"));
                }
                GhostMode::Scatter | GhostMode::Chase => {
                    next_state.set(AppState::PlayerDied);
                    next_dead_state.set(DeadState::Pause);
                }
                _ => (),
            }
        }
    }
}

fn despawn_timer_check(timer: Res<StateTimer>) -> bool {
    timer.0.elapsed_secs() >= 3.0
}

fn despawn_ghosts(mut commands: Commands, query: Query<Entity, With<Ghost>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

#[derive(Component)]
struct GhostEatenText;

fn ghost_eaten_system(
    mut commands: Commands,
    ghost_query: Query<(&Ghost, &Location), Without<GhostEatenText>>,
    eaten_text_query: Query<Entity, With<GhostEatenText>>,
    mut pause_timer: ResMut<CollisionPauseTimer>,
    time: Res<Time>,
    mut ghost_eaten_events: EventReader<GhostEaten>,
    asset_server: Res<AssetServer>,
) {
    if pause_timer.0.tick(time.delta()).just_finished() {
        for entity in eaten_text_query.iter() {
            commands.entity(entity).despawn_recursive();
        }
    }

    for event in ghost_eaten_events.read() {
        pause_timer.0.set_duration(Duration::from_secs(1));
        pause_timer.0.reset();

        let text_location = ghost_query
            .iter()
            .find_map(|(ghost, location)| {
                if *ghost == event.ghost {
                    Some(*location)
                } else {
                    None
                }
            })
            .expect("Ghost not found");
        let text_asset = asset_server.load(match event.eaten_ghosts {
            0 => "ghosts_death_points_200.png",
            1 => "ghosts_death_points_400.png",
            2 => "ghosts_death_points_800.png",
            3 => "ghosts_death_points_1600.png",
            _ => unreachable!(),
        });
        commands.spawn((
            SpriteBundle {
                texture: text_asset,
                transform: Transform::from_xyz(0.0, 0.0, Layers::OnMapText.as_f32()),
                ..default()
            },
            text_location,
            GhostEatenText,
        ));
    }
}
