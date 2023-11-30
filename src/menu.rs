use bevy::prelude::*;

use crate::{
    common::app_state::AppState,
    services::{map::Location, text::TextProvider},
};

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::MainMenu), setup_menu);
        app.add_systems(OnExit(AppState::MainMenu), despawn_menu);
        app.add_systems(FixedUpdate, update_menu);
    }
}

fn setup_menu(
    mut commands: Commands,
    text_provider: Res<TextProvider>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn((
        Location::new(13.5, 23.0),
        SpriteBundle {
            texture: text_provider.get_image("PACMAN", Color::YELLOW, &asset_server),
            sprite: Sprite {
                custom_size: Some(text_provider.get_size("PACMAN") * 2.0),
                ..default()
            },
            ..default()
        },
    ));

    spawn_option(&mut commands, &text_provider, &asset_server, "PLAY");
}

fn spawn_option(
    commands: &mut Commands,
    text_provider: &Res<TextProvider>,
    asset_server: &Res<AssetServer>,
    text: &str,
) {
    let arrow_location = Location::new(13.5 - (3.0 / 8.0) - (text.len() as f32 / 2.0), 15.0);
    commands.spawn((
        arrow_location,
        SpriteBundle {
            texture: asset_server.load("select_arrow.png"),
            ..default()
        },
    ));
    commands.spawn((
        Location::new(13.5, 15.0),
        SpriteBundle {
            texture: text_provider.get_image(text, Color::WHITE, asset_server),
            ..default()
        },
    ));
}

fn update_menu(mut state: ResMut<NextState<AppState>>, keyboard_input: ResMut<Input<KeyCode>>) {
    if keyboard_input.just_pressed(KeyCode::Return) {
        state.set(AppState::LevelStart);
    }
}

fn despawn_menu(mut commands: Commands, query: Query<Entity, With<Location>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
