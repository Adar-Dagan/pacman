use bevy::prelude::*;

#[derive(States, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub enum AppState {
    #[default]
    MainMenu,
    LevelStart,
    MainGame,
    LevelComplete,
    PlayerDied,
    GameOver,
    Leaderboard,
}

#[derive(States, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub enum DeadState {
    Pause,
    Animation,
    Restart,
    GameOver,
    #[default]
    NotDead,
}

#[derive(Resource)]
pub struct StateTimer(pub Timer);
