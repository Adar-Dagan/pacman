use bevy::prelude::*;

use crate::ghosts::Ghost;
use crate::services::map::Location;

#[derive(Event)]
pub struct PlayerAt {
    pub location: Location,
}

#[derive(Event)]
pub struct PelletEaten {
    pub power: bool,
}

#[derive(Event)]
pub struct GhostEaten {
    pub ghost: Ghost,
    pub eaten_ghosts: usize,
}

#[derive(Resource)]
pub struct CollisionPauseTimer(pub Timer);
