use bevy::prelude::*;

use super::map::Location;

#[derive(Event)]
pub struct PlayerAt {
    pub location: Location
}
