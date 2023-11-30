use std::mem::discriminant;

use bevy::{
    input::{keyboard::KeyboardInput, ButtonState},
    prelude::*,
};
use strum::{Display, EnumCount, EnumIter, IntoEnumIterator};

use crate::{
    common::{app_state::AppState, levels::Levels},
    services::{map::Location, text::TextProvider},
};

#[derive(Component, Debug, EnumCount, EnumIter, Display, Clone, Copy)]
enum Menu {
    Play,
    Hard_Mode(bool),
}

#[derive(Resource)]
struct MenuState {
    current: usize,
    options: [Menu; Menu::COUNT],
}

impl MenuState {
    fn current(&self) -> Menu {
        self.options[self.current]
    }

    fn set_current(&mut self, item: Menu) {
        let current = &mut self.options[self.current];
        assert!(discriminant(current) == discriminant(&item));
        *current = item;
    }
}

#[derive(Component)]
struct Arrow;

#[derive(Component, Clone, Debug, Copy, Default, PartialEq)]
enum Toggle {
    On,
    #[default]
    Off,
}

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::MainMenu), setup_menu);
        app.add_systems(OnExit(AppState::MainMenu), despawn_menu);
        app.add_systems(Update, update_menu.run_if(in_state(AppState::MainMenu)));
        app.insert_resource(MenuState {
            current: 0,
            options: [Menu::Play, Menu::Hard_Mode(false)],
        });
    }
}

fn setup_menu(
    mut commands: Commands,
    text_provider: Res<TextProvider>,
    asset_server: Res<AssetServer>,
    mut selected_option: ResMut<MenuState>,
) {
    selected_option.current = 0;

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

    for (i, option) in Menu::iter().enumerate() {
        let option_name = option.to_string().replace("_", " ").to_uppercase();
        println!("{}", option_name);
        commands
            .spawn((
                Location::new(13.5, 17.0 - (2 * i) as f32),
                SpatialBundle::default(),
                option,
            ))
            .with_children(|parent| {
                let arrow_location = Vec2::new(-3.0 - 8.0 * (option_name.len() as f32 / 2.0), 0.0);
                parent.spawn((
                    Arrow,
                    SpriteBundle {
                        texture: asset_server.load("select_arrow.png"),
                        transform: Transform::from_translation(arrow_location.extend(0.0)),
                        ..default()
                    },
                ));
                parent.spawn(SpriteBundle {
                    texture: text_provider.get_image(&option_name, Color::WHITE, &asset_server),
                    ..default()
                });
                match option {
                    Menu::Play => {}
                    Menu::Hard_Mode(_) => {
                        let on_location =
                            Vec2::new(8.0 * ((option_name.len() + 4) as f32 / 2.0), 0.0);
                        parent.spawn((
                            Toggle::On,
                            SpriteBundle {
                                texture: text_provider.get_image("ON", Color::GREEN, &asset_server),
                                transform: Transform::from_translation(on_location.extend(0.0)),
                                ..default()
                            },
                        ));

                        let off_location =
                            Vec2::new(8.0 * ((option_name.len() + 5) as f32 / 2.0) - 0.5, 0.0);
                        parent.spawn((
                            Toggle::Off,
                            SpriteBundle {
                                texture: text_provider.get_image("OFF", Color::RED, &asset_server),
                                transform: Transform::from_translation(off_location.extend(0.0)),
                                ..default()
                            },
                        ));
                    }
                }
            });
    }
}

fn update_menu(
    mut menu_state: ResMut<MenuState>,
    mut next_state: ResMut<NextState<AppState>>,
    mut levels: ResMut<Levels>,
    mut key_event: EventReader<KeyboardInput>,
    query: Query<(&Menu, &Children)>,
    mut query_arrow: Query<&mut Visibility, With<Arrow>>,
    mut query_toggle: Query<(&Toggle, &mut Visibility), Without<Arrow>>,
) {
    for event in key_event.read() {
        if event.state != ButtonState::Pressed {
            continue;
        }

        match event.key_code {
            Some(KeyCode::Up) => {
                menu_state.current = (menu_state.current as i32 - 1)
                    .rem_euclid(menu_state.options.len() as i32)
                    as usize;
            }
            Some(KeyCode::Down) => {
                menu_state.current = (menu_state.current as i32 + 1)
                    .rem_euclid(menu_state.options.len() as i32)
                    as usize;
            }
            Some(KeyCode::Return) => match menu_state.current() {
                Menu::Play => {
                    next_state.set(AppState::LevelStart);
                }
                Menu::Hard_Mode(state) => {
                    menu_state.set_current(Menu::Hard_Mode(!state));
                    levels.hard_mode = !state;
                }
            },
            _ => {}
        }
    }

    for (i, option) in menu_state.options.iter().enumerate() {
        let (_, children) = query
            .iter()
            .find(|(menu, _)| discriminant(*menu) == discriminant(option))
            .expect("Menu item not found");

        for child in children.iter() {
            if let Ok(mut visibility) = query_arrow.get_mut(*child) {
                *visibility = if i == menu_state.current {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                };
            } else if let Ok((toggle, mut visibility)) = query_toggle.get_mut(*child) {
                let item_state = if let Menu::Hard_Mode(state) = option {
                    state
                } else {
                    unreachable!();
                };

                *visibility = match (toggle, item_state) {
                    (Toggle::On, true) | (Toggle::Off, false) => Visibility::Visible,
                    _ => Visibility::Hidden,
                };
            }
        }
    }
}

fn despawn_menu(mut commands: Commands, query: Query<Entity, With<Location>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
