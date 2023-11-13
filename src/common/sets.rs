use bevy::prelude::*;

#[derive(SystemSet, Clone, Eq, PartialEq, Debug, Hash)]
pub enum GameLoop {
    Planning,
    Movement,
    Collisions,
}

