use bevy::prelude::*;

use strum::{EnumIter, IntoEnumIterator};
use derive_more::{ Add, Mul, AddAssign, Deref, DerefMut, Sub };

enum Tile {
    Wall,
    Empty,
    GhostHouse,
    GhostHouseDoor,
}

#[derive(Component)]
#[derive(Copy, Clone, Debug, PartialEq)]
#[derive(Add, AddAssign, Sub, Mul, Deref, DerefMut)]
pub struct Location {
    vec: Vec2,
}

impl Location {
    pub const ADVANCEMENT_DELTA: f32 = 1.0 / 8.0;

    pub fn new(x: f32, y: f32) -> Self {
        Self::from_vec(Vec2::new(x, y))
    }

    pub fn from_vec(vec: Vec2) -> Self {
        assert!(vec.x.fract() % Self::ADVANCEMENT_DELTA == 0.0);
        assert!(vec.y.fract() % Self::ADVANCEMENT_DELTA == 0.0);
        Self { vec }
    }

    pub fn get_tile(&self, direction: Direction) -> Self {
        let in_tile_vec = (*self + direction.get_vec() * 0.01).vec;
        let center_tile_vec = in_tile_vec.round();
        Self::from_vec(center_tile_vec)
    }

    pub fn advance(&mut self, direction: Direction) {
        *self += direction.get_vec() * Self::ADVANCEMENT_DELTA;
    }

    pub fn next_tile(&self, direction: Direction) -> Self {
        let current_tile = self.get_tile(direction);
        current_tile + direction.get_vec()
    }

    pub fn is_tile_center(&self) -> bool {
        self.x.fract() == 0.0 && self.y.fract() == 0.0
    }
}

#[derive(Component, EnumIter)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Direction {
    Left,
    Up,
    Right,
    Down,
}

impl Direction {
    pub fn get_vec(&self) -> Location {
        let vec = match self {
            Direction::Up => Vec2::new(0.0, 1.0),
            Direction::Left => Vec2::new(-1.0, 0.0),
            Direction::Down => Vec2::new(0.0, -1.0),
            Direction::Right => Vec2::new(1.0, 0.0),
        };

        Location::from_vec(vec)
    }

    pub fn opposite(&self) -> Direction {
        match self {
            Direction::Up => Direction::Down,
            Direction::Left => Direction::Right,
            Direction::Down => Direction::Up,
            Direction::Right => Direction::Left,
        }
    }

    pub fn rotation(&self) -> f32 {
        match self {
            Direction::Left => 0.0,
            Direction::Down => 0.25,
            Direction::Right => 0.5,
            Direction::Up => 0.75,
        }
    }
}

#[derive(Resource)]
pub struct Map {
    width: usize,
    height: usize,
    map: Vec<Tile>,
}

impl Map {
    pub fn parse(map_text: &str) -> Self {
        let height = map_text.lines().next().unwrap().len();
        let width = map_text.lines().count();
        let map = map_text.lines().flat_map(|line| {
            assert_eq!(line.len(), height, "All lines must have the same length");
            line.chars().map(|c| match c {
                'W' => Tile::Wall,
                ' ' => Tile::Empty,
                'H' => Tile::GhostHouse,
                'D' => Tile::GhostHouseDoor,
                _ => panic!("Invalid character in map"),
            })
        }).collect();
        Self { width, height, map }
    }

    pub fn possible_directions(&self, location: Location) -> Vec<Direction> {
        if location.x.fract() == 0.5 || !self.x_is_in_map(location.x) {
            return vec![Direction::Left, Direction::Right];
        } else if location.y.fract() == 0.5 || !self.y_is_in_map(location.y) {
            return vec![Direction::Up, Direction::Down];
        }

        Direction::iter().filter(|direction| {
            let tile_to_check = location.next_tile(*direction);
            return !self.is_blocked(tile_to_check);
        }).collect()
    }

    pub fn is_blocked(&self, location: Location) -> bool {
        if let Some(Tile::Empty) | None = self.get(location) {
            false
        } else {
            true
        }
    }

    fn get(&self, location: Location) -> Option<&Tile> {
        let x = location.x.round();
        let y = location.y.round();
        if x < 0.0 || y < 0.0 {
            None
        } else {
            self.map.get((x as usize) * self.height + (y as usize))
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn is_in_map(&self, location: Location) -> bool {
        self.x_is_in_map(location.x) && self.y_is_in_map(location.y)
    }

    fn y_is_in_map(&self, y: f32) -> bool {
        y > 0.0 && y < (self.height - 1) as f32
    }

    fn x_is_in_map(&self, x: f32) -> bool {
        x > 0.0 && x < (self.width - 1) as f32
    }

    // for debugging
    pub fn print_7x7(&self, current_tile: Location, next_tile: Location ) {
        let possible_directions = self.possible_directions(next_tile);
        let possible_locations = possible_directions.iter().map(|direction| {
            next_tile.next_tile(*direction)
        }).collect::<Vec<_>>();

        let start_x = current_tile.x as i32 - 3;
        let start_y = current_tile.y as i32 - 3;
        let end_x = start_x + 7;
        let end_y = start_y + 7;

        let mut result = String::new();
        for y in start_y..end_y {
            for x in start_x..end_x {
                let vec = Vec2::new(x as f32, y as f32);
                if vec == *current_tile {
                    result.push('C');
                } else if vec == *next_tile {
                    result.push('N');
                } else if possible_locations.contains(&Location::new(x as f32, y as f32)) {
                    result.push('P');
                } else if self.is_blocked(Location::from_vec(vec)) {
                    result.push('W');
                } else {
                    result.push(' ');
                }
            }
            result.push('\n');
        }

        println!("{}", result);
    }
}
                
