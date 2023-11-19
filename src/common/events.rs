use bevy::prelude::*;

use crate::ghosts::{Ghost, GhostMode};
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
pub struct Collision {
    pub ghost: Ghost,
    pub mode: GhostMode,
}

#[derive(Resource)]
pub struct CollisionPauseTimer(pub Timer);
