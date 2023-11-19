use bevy::prelude::*;

#[derive(Component)]
pub struct CharacterSpeed {
    speed: f32,
    advancement_counter: f32,
    missed_counter: f32,
    pub should_miss: bool,
}

impl CharacterSpeed {
    pub fn new(speed: f32) -> Self {
        assert!(speed >= 0.0 && speed <= 1.05);

        Self {
            speed,
            advancement_counter: 0.0,
            missed_counter: 0.0,
            should_miss: false,
        }
    }

    pub fn set_speed(&mut self, speed: f32) {
        assert!(speed >= 0.0 && speed <= 1.05);

        if speed != self.speed {
            self.speed = speed;
            self.advancement_counter = 0.0;
            self.missed_counter = 0.0;
            self.should_miss = false;
        }
    }

    pub fn tick(&mut self) {
        self.advancement_counter += 1.0;

        let precent_missed = self.missed_counter / self.advancement_counter;
        let precent_hit = (1.0 - precent_missed) * 1.05;

        if precent_hit > self.speed {
            self.missed_counter += 1.0;
            self.should_miss = true;
        } else {
            self.should_miss = false;
        }
    }
}
