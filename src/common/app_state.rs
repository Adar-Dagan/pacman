use bevy::prelude::*;

#[derive(States, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub enum AppState {
    #[default]
    MainMenu,
    LevelStart,
    MainGame,
    LevelComplete,
    GameOver,
    Leaderboard,
}

#[derive(Resource)]
pub struct StateTimer(pub Timer);
