use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct Levels {
    pub current: usize,
}

impl Levels {
    pub fn player_speed(&self) -> f32 {
        match self.current {
            1 => 0.8,
            5..=20 => 1.0,
            _ => 0.9,
        }
    }

    pub fn player_frite_speed(&self) -> f32 {
        match self.current {
            1 => 0.9,
            2..=4 => 0.95,
            _ => 1.0,
        }
    }

    pub fn ghost_normal_speed(&self) -> f32 {
        match self.current {
            1 => 0.75,
            2..=4 => 0.85,
            _ => 0.95,
        }
    }

    pub fn ghost_tunnel_speed(&self) -> f32 {
        match self.current {
            1 => 0.4,
            2..=4 => 0.45,
            _ => 0.5,
        }
    }

    pub fn elroy_1_dots(&self) -> usize {
        match self.current {
            1 => 20,
            2 => 30,
            3..=5 => 40,
            6..=8 => 50,
            9..=11 => 60,
            12..=14 => 80,
            15..=18 => 100,
            _ => 120,
        }
    }

    pub fn elroy_2_dots(&self) -> usize {
        match self.current {
            1 => 10,
            2 => 15,
            3..=5 => 20,
            6..=8 => 25,
            9..=11 => 30,
            12..=14 => 40,
            15..=18 => 50,
            _ => 60,
        }
    }

    pub fn elroy_1_speed(&self) -> f32 {
        match self.current {
            1 => 0.8,
            2..=4 => 0.9,
            _ => 1.0,
        }
    }

    pub fn elroy_2_speed(&self) -> f32 {
        match self.current {
            1 => 0.85,
            2..=4 => 0.95,
            _ => 1.05,
        }
    }

    pub fn ghost_frite_speed(&self) -> f32 {
        match self.current {
            1 => 0.5,
            2..=4 => 0.55,
            _ => 0.6,
        }
    }

    pub fn frite_duration(&self) -> u64 {
        match self.current {
            1 => 6,
            2 | 6 | 10 => 5,
            3 => 4,
            4 | 14 => 3,
            5 | 7 | 8 | 11 => 2,
            9 | 12 | 13 | 15 | 16 | 18 => 1,
            _ => 0,
        }
    }

    pub fn number_of_frite_flashes(&self) -> f32 {
        match self.current {
            1..=8 => 5.0,
            9 => 3.0,
            _ => 0.0,
        }
    }

    pub fn ghost_switch_global_mode(&self, index: usize) -> Option<f32> {
        match self.current {
            1 => [7.0, 20.0, 7.0, 20.0, 5.0, 20.0, 5.0],
            2..=4 => [7.0, 20.0, 7.0, 20.0, 5.0, 1033.0, 1.0 / 60.0],
            5.. => [5.0, 20.0, 5.0, 20.0, 5.0, 1037.0, 1.0 / 60.0],
            _ => unreachable!(),
        }.get(index).copied()
    }

    pub fn inky_home_exit_dots(&self) -> usize {
        match self.current {
            1 => 30,
            _ => 0,
        }
    }

    pub fn clyde_home_exit_dots(&self) -> usize {
        match self.current {
            1 => 90,
            2 => 50,
            _ => 0,
        }
    }
}
