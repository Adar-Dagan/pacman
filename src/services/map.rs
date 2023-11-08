use std::f32::EPSILON;

use bevy::prelude::*;

use strum::{EnumIter, IntoEnumIterator};

enum Tile {
    Wall,
    Empty,
    GhostHouse,
    GhostHouseDoor,
}

#[derive(Component, Copy, Clone, Debug, PartialEq)]
pub struct Location {
    vec: Vec2,
}

impl Location {
    pub fn set(&mut self, vec: Vec2) {
        const ONE_EIGHTH: f32 = 1.0 / 8.0;
        assert!(vec.x.fract() % ONE_EIGHTH == 0.0);
        assert!(vec.y.fract() % ONE_EIGHTH == 0.0);
        self.vec = vec;
    }

    pub fn new(x: f32, y: f32) -> Self {
        let mut new = Self { vec: Vec2::default() };
        new.set(Vec2::new(x, y));
        return new;
    }

    pub fn get(&self) -> Vec2 {
        self.vec
    }

    pub fn get_tile(&self, direction: Direction) -> Location {
        let mut new = *self;
        let new_vec = new.get() + direction.get_vec() * EPSILON;
        new.set(new_vec.round());
        new
    }
}

#[derive(Component, EnumIter, Copy, Clone, PartialEq, Debug)]
pub enum Direction {
    Left,
    Up,
    Right,
    Down,
}

impl Direction {
    pub fn get_vec(&self) -> Vec2 {
        match self {
            Direction::Up => Vec2::new(0.0, 1.0),
            Direction::Down => Vec2::new(0.0, -1.0),
            Direction::Left => Vec2::new(-1.0, 0.0),
            Direction::Right => Vec2::new(1.0, 0.0),
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
        let vec = location.get();

        if vec.x.fract() == 0.5 {
            return vec![Direction::Left, Direction::Right];
        } else if vec.y.fract() == 0.5 {
            return vec![Direction::Up, Direction::Down];
        }

        Direction::iter().filter(|direction| {
            let vec = vec + direction.get_vec();
            return !self.is_blocked(vec);
        }).collect()
    }

    pub fn is_blocked(&self, vec: Vec2) -> bool {
        if let Some(Tile::Empty) | None = self.get(vec) {
            false
        } else {
            true
        }
    }

    fn get(&self, vec: Vec2) -> Option<&Tile> {
        let x = vec.x.round();
        let y = vec.y.round();
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
}
                
