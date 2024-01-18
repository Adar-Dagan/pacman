use bevy::{
    input::{keyboard::KeyboardInput, ButtonState},
    prelude::*,
};
use std::{io::Write, time::Duration};

use crate::{
    common::{
        app_state::{AppState, DeadState},
        layers::Layers,
    },
    points::Points,
    services::{map::Location, text::TextProvider},
};

#[derive(Component)]
struct LetterIndex(usize);

#[derive(Component)]
struct PlayerName(String);

#[derive(Resource)]
struct FlashTimer(Timer);

#[derive(Component)]
struct GameOverSign;

#[derive(Resource, Default)]
struct GameOverTimer(Timer);

pub struct GameOverPlugin;

impl Plugin for GameOverPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(FlashTimer(Timer::from_seconds(0.5, TimerMode::Repeating)));
        app.insert_resource(GameOverTimer(Timer::from_seconds(3.0, TimerMode::Once)));
        app.add_systems(OnEnter(AppState::GameOver), (setup, despawn_game_over));
        app.add_systems(Update, update.run_if(in_state(AppState::GameOver)));
        app.add_systems(OnExit(AppState::GameOver), (save_score, despawn).chain());
        app.add_systems(
            OnEnter(DeadState::GameOver),
            (spawn_game_over, reset_game_over_timer),
        );
        app.add_systems(
            Update,
            goto_game_over_screen.run_if(in_state(DeadState::GameOver)),
        );
    }
}

fn setup(
    mut commands: Commands,
    mut text_provider: ResMut<TextProvider>,
    asset_server: Res<AssetServer>,
    points: Res<Points>,
    mut next_dead_state: ResMut<NextState<DeadState>>,
) {
    next_dead_state.set(DeadState::NotDead);

    commands.spawn((
        Location::new(13.5, 23.0),
        SpriteBundle {
            texture: text_provider.get_image("Game over", Color::RED, &asset_server),
            sprite: Sprite {
                custom_size: Some(text_provider.get_size("Game over") * 2.0),
                ..default()
            },
            ..default()
        },
    ));

    if points.score == points.high_score {
        commands.spawn((
            Location::new(13.5, 18.0),
            SpriteBundle {
                texture: text_provider.get_image("High Score!", Color::WHITE, &asset_server),
                ..default()
            },
        ));
    }

    commands.spawn((
        Location::new(13.5, 16.0),
        SpriteBundle {
            texture: text_provider.get_image(
                format!("Score: {}", points.score),
                Color::WHITE,
                &asset_server,
            ),
            ..default()
        },
    ));

    commands
        .spawn((
            Location::new(13.5, 14.0),
            PlayerName(String::with_capacity(10)),
            SpatialBundle::default(),
        ))
        .with_children(|parent| {
            parent.spawn(SpriteBundle {
                texture: text_provider.get_image("Name:", Color::WHITE, &asset_server),
                transform: Transform::from_translation(Vec3::new(-3.5 * 8.0, 0.0, 0.0)),
                ..default()
            });
            for i in 0..10 {
                parent.spawn((
                    LetterIndex(i),
                    SpriteBundle {
                        transform: Transform::from_translation(Vec3::new(i as f32 * 8.0, 0.0, 0.0)),
                        ..default()
                    },
                ));
            }
        });
}

fn update(
    mut player_name_query: Query<(&mut PlayerName, &Children)>,
    mut letter_query: Query<(
        &LetterIndex,
        &mut Transform,
        &mut Handle<Image>,
        &mut Visibility,
    )>,
    mut keyboard_events: EventReader<KeyboardInput>,
    mut text_provider: ResMut<TextProvider>,
    asset_server: Res<AssetServer>,
    mut flash_timer: ResMut<FlashTimer>,
    time: Res<Time>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    let (mut player_name, children) = player_name_query.single_mut();
    for event in keyboard_events.read() {
        if let KeyboardInput {
            state: ButtonState::Pressed,
            key_code: Some(key),
            ..
        } = event
        {
            let key_code = *key as u32;
            let new_char = match key_code {
                0..=8 => char::from_digit(key_code, 10),
                9 => Some('0'),
                10..=35 => char::from_digit(key_code, 36),
                76 => Some(' '),
                _ => None,
            };

            if let Some(c) = new_char {
                if player_name.0.len() < 10 {
                    player_name.0.push(c);
                }
            }

            if let KeyCode::Back = key {
                player_name.0.pop();
            }

            if let KeyCode::Return = key {
                next_state.set(AppState::MainMenu);
            }
        }
    }

    flash_timer.0.tick(time.delta());

    for child in children {
        let letter_result = letter_query.get_mut(*child);
        if letter_result.is_err() {
            continue;
        }

        let (letter_index, mut transform, mut texture, mut visibility) = letter_result.unwrap();
        let letter_index = letter_index.0;

        if player_name.0.len() < letter_index {
            *visibility = Visibility::Hidden;
            continue;
        } else if player_name.0.len() == letter_index {
            *visibility = match (*visibility, flash_timer.0.just_finished()) {
                (vis, false) => vis,
                (Visibility::Inherited, true) => Visibility::Hidden,
                (Visibility::Hidden, true) => Visibility::Inherited,
                _ => unreachable!(),
            };
            *texture = text_provider.get_image('_', Color::WHITE, &asset_server);
            transform.translation.y = -4.0;
        } else {
            let char = player_name.0.chars().nth(letter_index).unwrap();
            if char == ' ' {
                *visibility = Visibility::Hidden;
            } else {
                *visibility = Visibility::Inherited;
                *texture = text_provider.get_image(char, Color::WHITE, &asset_server);
                transform.translation.y = 0.0;
            }
        }
    }
}

fn despawn(mut commands: Commands, query: Query<Entity, With<Location>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn save_score(mut points: ResMut<Points>, player_name_query: Query<&PlayerName>) {
    let player_name = player_name_query.single();
    if player_name.0.is_empty() {
        return;
    }

    let mut scores_file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .append(true)
        .open("scores")
        .expect("Failed to open scores file");

    writeln!(scores_file, "{}:{}", player_name.0, points.score).expect("Failed to write score");

    points.score = 0;
}

fn spawn_game_over(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut text_provider: ResMut<TextProvider>,
) {
    commands.spawn((
        GameOverSign,
        Location::new(13.5, 13.0),
        SpriteBundle {
            texture: text_provider.get_image("Game over", Color::RED, &asset_server),
            transform: Transform::from_xyz(0.0, 0.0, Layers::OnMapText.as_f32()),
            ..default()
        },
    ));
}

fn despawn_game_over(mut commands: Commands, query: Query<Entity, With<GameOverSign>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn reset_game_over_timer(mut game_over_timer: ResMut<GameOverTimer>) {
    game_over_timer.0.set_duration(Duration::from_secs(3));
    game_over_timer.0.reset();
}

fn goto_game_over_screen(
    mut game_over_timer: ResMut<GameOverTimer>,
    time: Res<Time>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if game_over_timer.0.tick(time.delta()).just_finished() {
        next_state.set(AppState::GameOver);
    }
}
