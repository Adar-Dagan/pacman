use std::f32::consts::TAU;

use bevy::prelude::*;

use crate::common::layers::Layers;
use crate::common::sets::GameLoop;
use crate::services::map::{Direction, Map, Location};
use crate::common::events::PlayerAt;

#[derive(Component)]
pub struct Player {
    pub is_blocked: bool
}

#[derive(Bundle)]
struct PlayerBundle {
    location: Location,
    direction: Direction,
    player: Player,
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_characters);
        app.add_systems(FixedUpdate, (update_player.in_set(GameLoop::Planning),
                                      move_player.in_set(GameLoop::Movement)));
        app.add_systems(Update, update_pacman_sprite);
    }
}

fn spawn_characters(mut commands: Commands,
                    asset_server: Res<AssetServer>,
                    mut texture_atlases: ResMut<Assets<TextureAtlas>>) {
    let texture_handle = asset_server.load("pacman.png");
    let texture_atlas =
        TextureAtlas::from_grid(texture_handle, Vec2::new(15.0, 15.0), 3, 1, None, None);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);

    commands.spawn((
            PlayerBundle {
                location: Location::new(13.5, 7.0),
                player: Player { is_blocked: false },
                direction: Direction::Left,
            },
            SpriteSheetBundle {
                texture_atlas: texture_atlas_handle,
                sprite: TextureAtlasSprite::new(0), 
                transform: Transform::from_xyz(0.0, 0.0, Layers::Player.as_f32()),
                ..default()
            }));
}

fn update_player(mut query: Query<(&Location, 
                                   &mut Direction),
                                   With<Player>>,
                 map: Res<Map>,
                 key: Res<Input<KeyCode>>) {
    let (location, mut direction) = query.single_mut();

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
        *direction = *d;
    }
}

fn move_player(mut query: Query<(&mut Location, &Direction, &mut Player)>,
               map: Res<Map>,
               mut player_at_events: EventWriter<PlayerAt>) {
    let (mut location, direction, mut player) = query.single_mut();

    player.is_blocked =  *location == location.get_tile(*direction) && 
        map.is_blocked(location.next_tile(*direction));

    if player.is_blocked {
        return;
    }

    location.advance(*direction);

    match *direction {
        Direction::Up | Direction::Down => {
            location.x = bring_towards_center(location.x);
        },
        Direction::Left | Direction::Right => {
            location.y = bring_towards_center(location.y);
        },
    };

    player_at_events.send(PlayerAt { location: location.get_tile(*direction) });
}

fn bring_towards_center(location: f32) -> f32 {
    if location.fract() == 0.0 {
        return location;
    }

    let dif_from_center = location.round() - location; 
    let dif_sign = dif_from_center.signum();
    let location = location + dif_sign * Location::ADVANCEMENT_DELTA;

    location
}


fn update_pacman_sprite(mut query: Query<(&Location,
                                          &mut Transform,
                                          &Direction,
                                          &mut TextureAtlasSprite,
                                          &Player)>) {
    let (location, mut transform, direction, mut sprite, player) = 
        query.single_mut();

    let index = if player.is_blocked {
        1
    } else {
        let masked_location = *location * *direction.get_vec();
        let value_in_direction = if masked_location.x.fract() == 0.0 {
            masked_location.y
        } else {
            masked_location.x
        };
        let positive_fraction = value_in_direction.rem_euclid(1.0);

        let quarter = ((positive_fraction * 4.0).floor() as usize + 1) % 4;
        if quarter == 3 { 1 } else { quarter }
    };

    if sprite.index != index {
        sprite.index = index;
    }

    let rotation = Quat::from_rotation_z(TAU * direction.rotation());
    if transform.rotation != rotation {
        transform.rotation = rotation;
    }
}

