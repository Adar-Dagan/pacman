use bevy::prelude::*;

#[derive(States, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub enum AppState {
    #[default]
    MainMenu,
    LevelStart,
    MainGame,
    LevelComplete,
    GameOver,
}

#[derive(Resource)]
pub struct StateTimer(pub Timer);
