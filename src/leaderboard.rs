use std::{fmt::Display, fs::OpenOptions, io::BufRead, io::BufReader};

use bevy::{input::keyboard::KeyboardInput, prelude::*};

use crate::{
    common::app_state::AppState,
    services::{map::Location, text::TextProvider},
};

#[derive(Component)]
struct Entry {
    index: usize,
    name: String,
    score: u32,
}

#[derive(Resource)]
struct LeaderboardState {
    top_entry_index: usize,
    entries: Vec<(String, u32)>,
}

#[derive(Component, Clone, Copy)]
enum EntryPart {
    Index,
    Name,
    Score,
}

pub struct LeaderboardPlugin;

impl Plugin for LeaderboardPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Leaderboard), setup);
        app.add_systems(Update, update.run_if(in_state(AppState::Leaderboard)));
        app.add_systems(OnExit(AppState::Leaderboard), despawn);
        app.insert_resource(LeaderboardState {
            top_entry_index: 0,
            entries: vec![],
        });
    }
}

fn setup(
    mut commands: Commands,
    mut text_provider: ResMut<TextProvider>,
    asset_server: Res<AssetServer>,
    mut leaderboard_state: ResMut<LeaderboardState>,
) {
    leaderboard_state.top_entry_index = 0;
    leaderboard_state.entries.clear();

    let scores = OpenOptions::new().read(true).open("scores");
    if let Ok(scores) = scores {
        let scores = BufReader::new(scores);

        leaderboard_state.entries.extend(scores.lines().map(|line| {
            line.expect("Failed to open scores file")
                .split_once(':')
                .map(|(name, score)| {
                    (
                        name.to_string(),
                        score.parse::<u32>().expect("Scores file is corrupted"),
                    )
                })
                .expect("Scores file is corrupted")
        }));
    }
    leaderboard_state.entries.sort_by(|(_, a), (_, b)| b.cmp(a));

    commands.spawn((
        Location::new(13.5, 27.0),
        SpriteBundle {
            texture: text_provider.get_image("LeaderBoard", Color::WHITE, &asset_server),
            sprite: Sprite {
                custom_size: Some(text_provider.get_size("LeaderBoard") * 1.5),
                ..default()
            },
            ..default()
        },
    ));

    commands
        .spawn((Location::new(13.5, 23.0), SpatialBundle::default()))
        .with_children(|parent| {
            parent.spawn(get_entry_part(
                EntryPart::Name,
                &"Name",
                &mut text_provider,
                &asset_server,
            ));

            parent.spawn(get_entry_part(
                EntryPart::Score,
                &"Score",
                &mut text_provider,
                &asset_server,
            ));
        });

    for i in 0..10 {
        commands
            .spawn((
                Location::new(13.5, 23.0 - (i + 1) as f32 * 2.0),
                SpatialBundle::default(),
                Entry {
                    index: i,
                    name: "1234567890".to_string(),
                    score: 1234567890,
                },
            ))
            .with_children(|parent| {
                parent.spawn(get_entry_part(
                    EntryPart::Index,
                    &".",
                    &mut text_provider,
                    &asset_server,
                ));

                parent.spawn(get_entry_part(
                    EntryPart::Name,
                    &".",
                    &mut text_provider,
                    &asset_server,
                ));

                parent.spawn(get_entry_part(
                    EntryPart::Score,
                    &".",
                    &mut text_provider,
                    &asset_server,
                ));
            });
    }
}

fn get_entry_part<T: Display>(
    entry_part: EntryPart,
    text: &T,
    text_provider: &mut TextProvider,
    assest_server: &AssetServer,
) -> (EntryPart, SpriteBundle) {
    let x = get_part_location(entry_part, text_provider, text);

    (
        entry_part,
        SpriteBundle {
            texture: text_provider.get_image(text, Color::WHITE, assest_server),
            transform: Transform::from_xyz(x, 0.0, 0.0),
            ..default()
        },
    )
}

fn get_part_location<T: Display>(
    entry_part: EntryPart,
    text_provider: &mut TextProvider,
    text: &T,
) -> f32 {
    const DISPLACEMENT: f32 = 10.0;

    let displacement = match entry_part {
        EntryPart::Index => 0.0,
        EntryPart::Name => 2.5,
        EntryPart::Score => 13.5,
    };
    text_provider.get_size(text).x / 2.0
        * if let EntryPart::Index = entry_part {
            -1.0
        } else {
            1.0
        }
        + (displacement - DISPLACEMENT) * 8.0
}

fn update(
    mut leaderboard_state: ResMut<LeaderboardState>,
    mut entry_query: Query<(&mut Entry, &mut Visibility, &Children), Without<EntryPart>>,
    mut entry_part_query: Query<(&EntryPart, &mut Transform, &mut Handle<Image>)>,
    mut keyboard_events: EventReader<KeyboardInput>,
    mut text_provider: ResMut<TextProvider>,
    asset_server: Res<AssetServer>,
) {
    for event in keyboard_events.read() {
        if !event.state.is_pressed() {
            continue;
        }

        match event.key_code {
            Some(KeyCode::Up) => {
                if leaderboard_state.top_entry_index > 0 {
                    leaderboard_state.top_entry_index -= 1;
                }
            }
            Some(KeyCode::Down) => {
                if leaderboard_state.top_entry_index < leaderboard_state.entries.len() - 1 {
                    leaderboard_state.top_entry_index += 1;
                }
            }
            _ => continue,
        }
    }

    for (entry, mut visibility, children) in entry_query.iter_mut() {
        let leaderboard_entry = leaderboard_state
            .entries
            .get(entry.index + leaderboard_state.top_entry_index);

        if leaderboard_entry.is_none() {
            *visibility = Visibility::Hidden;
            continue;
        } else {
            *visibility = Visibility::Inherited;
        }

        let (name, score) = leaderboard_entry.unwrap();

        for child in children.iter() {
            let (entry_part, mut transform, mut handle) = entry_part_query.get_mut(*child).unwrap();

            let text = match *entry_part {
                EntryPart::Index => {
                    format!("{}:", entry.index + 1 + leaderboard_state.top_entry_index)
                }
                EntryPart::Name => name.clone(),
                EntryPart::Score => score.to_string(),
            };
            *handle = text_provider.get_image(&text, Color::WHITE, &asset_server);

            let x = get_part_location(*entry_part, &mut text_provider, &text);
            transform.translation.x = x;
        }
    }
}

fn despawn(mut commands: Commands, query: Query<Entity, With<Location>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
