use bevy::prelude::*;

use crate::services::map::Location;

#[derive(Event)]
pub struct PlayerAt {
    pub location: Location
}
