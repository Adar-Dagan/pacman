use std::{fs::OpenOptions, io::BufRead, io::BufReader};

use bevy::prelude::*;

use crate::{
    advance_level,
    common::{
        app_state::AppState,
        events::{CollisionPauseTimer, GhostEaten, PelletEaten},
        layers::Layers,
        levels::Levels,
        sets::GameLoop,
    },
    map_render::NoMapWrap,
    player::Player,
    services::{map::Location, text::TextProvider},
};

#[derive(Component, Clone, Copy, Debug)]
pub enum BonusSymbol {
    Cherries,
    Strawberry,
    Peach,
    Apple,
    Grapes,
    Galaxian,
    Bell,
    Key,
}

impl BonusSymbol {
    fn points(&self) -> u32 {
        match self {
            BonusSymbol::Cherries => 100,
            BonusSymbol::Strawberry => 300,
            BonusSymbol::Peach => 500,
            BonusSymbol::Apple => 700,
            BonusSymbol::Grapes => 1000,
            BonusSymbol::Galaxian => 2000,
            BonusSymbol::Bell => 3000,
            BonusSymbol::Key => 5000,
        }
    }

    fn asset(&self) -> &'static str {
        match self {
            BonusSymbol::Cherries => "cherries.png",
            BonusSymbol::Strawberry => "strawberry.png",
            BonusSymbol::Peach => "peach.png",
            BonusSymbol::Apple => "apple.png",
            BonusSymbol::Grapes => "grapes.png",
            BonusSymbol::Galaxian => "galaxian.png",
            BonusSymbol::Bell => "bell.png",
            BonusSymbol::Key => "key.png",
        }
    }

    fn eaten_asset(&self) -> &'static str {
        match self {
            BonusSymbol::Cherries => "bonus_points_100.png",
            BonusSymbol::Strawberry => "bonus_points_300.png",
            BonusSymbol::Peach => "bonus_points_500.png",
            BonusSymbol::Apple => "bonus_points_700.png",
            BonusSymbol::Grapes => "bonus_points_1000.png",
            BonusSymbol::Galaxian => "bonus_points_2000.png",
            BonusSymbol::Bell => "bonus_points_3000.png",
            BonusSymbol::Key => "bonus_points_5000.png",
        }
    }
}

#[derive(Component)]
struct SymbolTimer(Timer);

#[derive(Resource)]
struct BonusTextTimer(Timer);

#[derive(Resource)]
pub struct Points {
    pub score: u32,
    pub high_score: u32,
}

#[derive(Component, Debug)]
enum PointsText {
    Still,
    Score,
    HighScore,
}

#[derive(Component)]
struct Digit {
    digit: u8,
}

#[derive(Resource)]
struct PelletEatenCounter(usize);

#[derive(Resource)]
struct GhostsEatenCounter([Option<u8>; 4], Option<usize>);

impl GhostsEatenCounter {
    fn ghost_eaten(&mut self) {
        assert!(self.1.is_some());
        assert!(self.0[self.1.unwrap()].is_some());

        let currently_eaten = self.0[self.1.unwrap()].unwrap();
        self.0[self.1.unwrap()] = Some(currently_eaten + 1);
    }

    fn power_pellet_eaten(&mut self) {
        self.1 = if let Some(i) = self.1 {
            Some(i + 1)
        } else {
            Some(0)
        };
        self.0[self.1.unwrap()] = Some(0);
    }
}

pub struct PointsPlugin;

impl Plugin for PointsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Points {
            score: 0,
            high_score: 0,
        });
        app.insert_resource(GhostsEatenCounter([None; 4], None));
        app.insert_resource(PelletEatenCounter(0));
        app.insert_resource(BonusTextTimer(Timer::from_seconds(3.0, TimerMode::Once)));
        app.add_systems(OnEnter(AppState::LevelStart), setup.after(advance_level));
        app.add_systems(OnExit(AppState::LevelComplete), despawn);
        app.add_systems(OnEnter(AppState::GameOver), despawn);
        app.add_systems(
            FixedUpdate,
            (
                generate_bonus_symbol,
                (bonus_symbol_collision, bonus_symbol_timer).run_if(symbol_exists),
                remove_bonus_text,
            )
                .chain()
                .in_set(GameLoop::Collisions)
                .run_if(in_state(AppState::MainGame)),
        );
        app.add_systems(
            FixedUpdate,
            update_points
                .run_if(in_state(AppState::MainGame))
                .after(GameLoop::Collisions),
        );
        app.add_systems(Update, draw_points.run_if(in_state(AppState::MainGame)));
    }
}

fn setup(
    mut commands: Commands,
    text_provider: ResMut<TextProvider>,
    asset_server: Res<AssetServer>,
    mut ghost_eaten_counter: ResMut<GhostsEatenCounter>,
    levels: Res<Levels>,
    mut pellets_eaten_counter: ResMut<PelletEatenCounter>,
    mut points: ResMut<Points>,
) {
    pellets_eaten_counter.0 = 0;
    *ghost_eaten_counter = GhostsEatenCounter([None; 4], None);

    spawn_points(&mut commands, text_provider.into_inner(), &asset_server);

    spawn_level_counter(&mut commands, &levels, &asset_server);

    let scores = OpenOptions::new().read(true).open("scores");

    if let Ok(scores) = scores {
        let reader = BufReader::new(scores);
        points.high_score = reader
            .lines()
            .map(|line| {
                line.expect("Error reading scores file")
                    .split_once(':')
                    .expect("Scores file is corrupt")
                    .1
                    .parse::<u32>()
                    .expect("Scores file is corrupt")
            })
            .max()
            .unwrap_or(0);
    } else {
        points.high_score = 0;
    }
}

fn despawn(
    mut commands: Commands,
    query: Query<
        Entity,
        Or<(
            With<LevelCounter>,
            With<PointsText>,
            With<BonusText>,
            With<BonusSymbol>,
        )>,
    >,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

#[derive(Component)]
struct LevelCounter;

fn spawn_level_counter(commands: &mut Commands, levels: &Levels, asset_server: &AssetServer) {
    for (i, symbol) in levels.level_counter_bonus_symbols().iter().enumerate() {
        commands.spawn((
            LevelCounter,
            NoMapWrap,
            Location::new(24.5 - (i * 2) as f32, -1.5),
            SpriteBundle {
                texture: asset_server.load(symbol.asset()),
                transform: Transform::from_xyz(0.0, 0.0, Layers::Text.as_f32()),
                ..default()
            },
        ));
    }
}

fn spawn_points(
    commands: &mut Commands,
    text_provider: &mut TextProvider,
    asset_server: &AssetServer,
) {
    commands.spawn((
        NoMapWrap,
        PointsText::Still,
        Location::new(13.5, 33.0),
        SpriteBundle {
            texture: text_provider.get_image("High score", Color::WHITE, asset_server),
            transform: Transform::from_xyz(0.0, 0.0, Layers::Text.as_f32()),
            ..default()
        },
    ));
    commands
        .spawn((
            NoMapWrap,
            PointsText::HighScore,
            Location::new(15.0, 32.0),
            SpatialBundle {
                transform: Transform::from_xyz(0.0, 0.0, Layers::Text.as_f32()),
                ..default()
            },
        ))
        .with_children(|parent| {
            for i in 0..=8 {
                parent.spawn((
                    Digit { digit: i },
                    SpriteBundle {
                        transform: Transform::from_xyz(
                            -((i * 8) as f32),
                            0.0,
                            Layers::Text.as_f32(),
                        ),
                        visibility: Visibility::Hidden,
                        ..default()
                    },
                ));
            }
        });
    commands
        .spawn((
            NoMapWrap,
            PointsText::Score,
            Location::new(6.0, 32.0),
            SpatialBundle {
                transform: Transform::from_xyz(0.0, 0.0, Layers::Text.as_f32()),
                ..default()
            },
        ))
        .with_children(|parent| {
            for i in 0..=8 {
                parent.spawn((
                    Digit { digit: i },
                    SpriteBundle {
                        texture: text_provider.get_image("0", Color::WHITE, asset_server),
                        transform: Transform::from_xyz(
                            -((i * 8) as f32),
                            0.0,
                            Layers::Text.as_f32(),
                        ),
                        visibility: if i < 2 {
                            Visibility::Inherited
                        } else {
                            Visibility::Hidden
                        },
                        ..default()
                    },
                ));
            }
        });
}

fn draw_points(
    query: Query<(&Children, &PointsText)>,
    mut decimal_query: Query<(&mut Handle<Image>, &mut Visibility, &Digit)>,
    points: Res<Points>,
    mut text_provider: ResMut<TextProvider>,
    asset_server: Res<AssetServer>,
) {
    for (children, points_text) in query.iter() {
        let text = match points_text {
            PointsText::Still => continue,
            PointsText::Score => points.score.to_string(),
            PointsText::HighScore => points.high_score.to_string(),
        };

        let chars = text.chars().rev().collect::<Vec<_>>();
        for child in children.iter() {
            let (mut image, mut visibility, digit) = decimal_query.get_mut(*child).unwrap();

            if let Some(c) = chars.get(digit.digit as usize) {
                *image = text_provider.get_image(c, Color::WHITE, &asset_server);
                *visibility = Visibility::Inherited;
            } else {
                *visibility = Visibility::Hidden;
            }
        }
    }
}

fn update_points(
    mut points: ResMut<Points>,
    mut pellet_eaten_events: EventReader<PelletEaten>,
    mut ghost_eaten_events: EventReader<GhostEaten>,
    mut ghosts_eaten_counter: ResMut<GhostsEatenCounter>,
) {
    for pellet_eaten in pellet_eaten_events.read() {
        if pellet_eaten.power {
            points.score += 50;
            ghosts_eaten_counter.power_pellet_eaten();
        } else {
            points.score += 10;
        }
    }

    for event in ghost_eaten_events.read() {
        let ghosts_eaten = event.eaten_ghosts;
        points.score += 100 * (2_u32.pow(ghosts_eaten as u32 + 1));

        ghosts_eaten_counter.ghost_eaten();
        let total_ghosts_eaten = ghosts_eaten_counter
            .0
            .iter()
            .fold(0, |acc, x| acc + x.unwrap_or(0));
        if total_ghosts_eaten == 4 * 4 {
            points.score += 12000;
        }
    }

    if points.score > points.high_score {
        points.high_score = points.score;
    }
}

fn generate_bonus_symbol(
    mut command: Commands,
    mut pellet_eaten_events: EventReader<PelletEaten>,
    mut pellets_eaten_counter: ResMut<PelletEatenCounter>,
    levels: Res<Levels>,
    asset_server: Res<AssetServer>,
) {
    for _ in pellet_eaten_events.read() {
        pellets_eaten_counter.0 += 1;

        if pellets_eaten_counter.0 == 70 || pellets_eaten_counter.0 == 170 {
            let bonus_symbol = levels.bonus_symbol();
            let symbol_timer = Timer::from_seconds(9.0 + fastrand::f32(), TimerMode::Once);

            command.spawn((
                bonus_symbol,
                SymbolTimer(symbol_timer),
                NoMapWrap,
                Location::new(13.5, 13.0),
                SpriteBundle {
                    texture: asset_server.load(bonus_symbol.asset()),
                    transform: Transform::from_xyz(0.0, 0.0, Layers::BonusSymbols.as_f32()),
                    ..default()
                },
            ));
        }
    }
}

#[derive(Component)]
struct BonusText;

fn bonus_symbol_collision(
    mut commands: Commands,
    mut query: Query<(Entity, &Location, &BonusSymbol)>,
    player_query: Query<&Location, With<Player>>,
    mut points: ResMut<Points>,
    asset_server: Res<AssetServer>,
    mut text_timer: ResMut<BonusTextTimer>,
) {
    let player_location = player_query.single();
    let (entity, location, bonus_symbol) = query.single_mut();

    if player_location == location {
        points.score += bonus_symbol.points();
        commands.entity(entity).despawn();

        commands.spawn((
            BonusText,
            NoMapWrap,
            location.clone(),
            SpriteBundle {
                texture: asset_server.load(bonus_symbol.eaten_asset()),
                transform: Transform::from_xyz(0.0, 0.0, Layers::OnMapText.as_f32()),
                ..default()
            },
        ));
        text_timer.0.reset();
    }
}

fn bonus_symbol_timer(
    mut commands: Commands,
    mut query: Query<(Entity, &mut SymbolTimer)>,
    pause_timer: Res<CollisionPauseTimer>,
    time: Res<Time>,
) {
    if !pause_timer.0.finished() {
        return;
    }
    let (entity, mut timer) = query.single_mut();
    if timer.0.tick(time.delta()).just_finished() {
        commands.entity(entity).despawn();
    }
}

fn symbol_exists(query: Query<&BonusSymbol>) -> bool {
    !query.is_empty()
}

fn remove_bonus_text(
    mut commands: Commands,
    query: Query<Entity, With<BonusText>>,
    mut timer: ResMut<BonusTextTimer>,
    time: Res<Time>,
) {
    if !timer.0.tick(time.delta()).just_finished() {
        return;
    }

    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}
