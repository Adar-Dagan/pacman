use std::f32::consts::TAU;

use bevy::prelude::*;

use crate::services::{map::{Direction, Map, Location}, events::PlayerAt};

#[derive(Component)]
struct Player {
    pub is_blocked: bool
}

#[derive(Component)]
struct Sprites([Handle<Image>; 3]);

pub struct CharacterPlugin;

impl Plugin for CharacterPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_characters);
        app.add_systems(FixedUpdate, (update_player, map_wrap.after(update_player)));
        app.add_systems(Update, update_pacman_sprite);
    }
}

fn spawn_characters(mut commands: Commands,
                    asset_server: Res<AssetServer>) {
    commands.spawn((
        Location::new(13.5, 7.0),
        Player { is_blocked: false },
        Direction::Right,
        Sprites([
            asset_server.load("pacman_closed.png"),
            asset_server.load("pacman_open_small.png"),
            asset_server.load("pacman_open_large.png"),
        ]),
        SpriteBundle {
            texture: asset_server.load("pacman_closed.png"),
            transform: Transform::from_xyz(0.0, 0.0, 20.0),
            ..default()
        }));
}

fn update_player(mut query: Query<(&mut Location, 
                                   &mut Direction, 
                                   &mut Player)>,
                 map: Res<Map>,
                 key: Res<Input<KeyCode>>,
                 mut player_at_events: EventWriter<PlayerAt>) {
    let mut query_iterator = query.iter_mut();
    let (mut location, mut direction, mut player)
        = query_iterator.next().expect("Query didn't find player");
    if query_iterator.next().is_some() {
        panic!("Query found more than one player");
    }

    let possible_directions = map.possible_directions(*location);

    let new_direction = possible_directions.iter().filter(|direction| {
        match **direction {
            Direction::Up => key.pressed(KeyCode::Up),
            Direction::Down => key.pressed(KeyCode::Down),
            Direction::Left => key.pressed(KeyCode::Left),
            Direction::Right => key.pressed(KeyCode::Right),
        }
    }).next();

    if let Some(d) = new_direction {
        if *d != *direction {
            *direction = *d;
        }
    }

    let advancement_delta = 1.0 / 8.0;
    let mut new_location = location.get() + direction.get_vec() * advancement_delta;

    if map.is_blocked(new_location + direction.get_vec() * 0.5) {
        player.is_blocked = true;
        new_location = new_location.round();
    } else {
        player.is_blocked = false;
        match *direction {
            Direction::Up | Direction::Down => {
                new_location.x = bring_to_center(new_location.x, advancement_delta);
            },
            Direction::Left | Direction::Right => {
                new_location.y = bring_to_center(new_location.y, advancement_delta);
            },
        };
    }
    location.set(new_location);

    player_at_events.send(PlayerAt { location: location.get_tile(*direction) });
}

fn bring_to_center(location: f32, advancement_delta: f32) -> f32 {
    if location.fract() == 0.0 {
        return location;
    }

    let dif_from_center = location.round() - location; 
    let dif_sign = dif_from_center.signum();
    let location = location + dif_sign * advancement_delta;
    
    location
}

fn map_wrap(mut query: Query<&mut Location>, map: Res<Map>) {
    query.par_iter_mut().for_each(|mut location| {
        let mut vec = location.get();
        const PIXELS: f32 = 1.0 / 8.0;
        if vec.x == -PIXELS * 14.0 {
            vec.x = map.width() as f32 - 1.0 + PIXELS * 14.0;
        } else if vec.x == (map.width() as f32 - 1.0 + PIXELS * 14.0) {
            vec.x = -PIXELS * 14.0;
        }

        if vec.y == -PIXELS * 14.0 {
            vec.y = map.height() as f32 - 1.0 + PIXELS * 14.0;
        } else if vec.y == (map.height() as f32 - 1.0 + PIXELS * 14.0) {
            vec.y = -PIXELS * 14.0;
        }

        location.set(vec);
    });
}

fn update_pacman_sprite(mut query: Query<(&Location, &mut Handle<Image>, &mut Transform, &Direction, &Sprites, &Player)>) {
    let mut query_iterator = query.iter_mut();
    let (location, mut sprite, mut transform, direction, sprites, player)
        = query_iterator.next().expect("Query didn't find player");
    if query_iterator.next().is_some() {
        panic!("Query found more than one player");
    }

    let sprite_index = if player.is_blocked {
        1
    } else {
        let direction_value = match location.get() * direction.get_vec() {
            Vec2 { x, y } if x == 0.0 => y,
            Vec2 { x, y } if y == 0.0 => x,
            _ => unreachable!(),
        }.fract().abs();

        let quarter = ((direction_value * 4.0).floor() as usize + 1) % 4;
        if quarter == 3 { 1 } else { quarter }
    };

    *sprite = sprites.0[sprite_index].clone();

    let rotation_multiplier = match *direction {
        Direction::Left => 0.0,
        Direction::Down => 1.0,
        Direction::Right => 2.0,
        Direction::Up => 3.0,
    };

    let rotation = Quat::from_rotation_z(rotation_multiplier * TAU / 4.0);
    if transform.rotation != rotation {
        transform.rotation = rotation;
    }
}

