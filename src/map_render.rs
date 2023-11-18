use bevy::prelude::*;

use crate::common::app_state::{AppState, StateTimer};
use crate::services::map::{Map, Location}; 
use crate::common::sets::GameLoop;
use crate::common::layers::Layers;

#[derive(Component)]
struct MapComponent;

#[derive(Component)]
struct ReadySign;

pub struct MapRenderPlugin;

impl Plugin for MapRenderPlugin {
    fn build(&self, app: &mut App) {
        const MAP_TEXT: &str = include_str!("map");

        app.insert_resource(Map::parse(MAP_TEXT));
        app.add_systems(OnEnter(AppState::LevelStart), render_map);
        app.add_systems(OnExit(AppState::LevelStart), remove_ready);
        app.add_systems(FixedUpdate, map_wrap.after(GameLoop::Movement)
                                             .before(GameLoop::Collisions)
                                             .run_if(in_state(AppState::MainGame)));
        app.add_systems(Update, update_entities_location);

        app.add_systems(Update, flash_map.run_if(in_state(AppState::LevelComplete)));
        app.add_systems(OnExit(AppState::LevelComplete), despawn);
    }
}

fn render_map(mut commands: Commands,
              asset_server: Res<AssetServer>,
              mut texture_atlases: ResMut<Assets<TextureAtlas>>) {
    let map_texture = asset_server.load("map.png");
    let texture_atlas = TextureAtlas::from_grid(map_texture, Vec2::new(226.0, 248.0), 28, 36, None, None);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);
    
    commands.spawn((MapComponent,
                    SpriteSheetBundle{
                        texture_atlas: texture_atlas_handle,
                        sprite: TextureAtlasSprite::new(0),
                        transform: Transform::from_xyz(0.0, 0.0, Layers::Map.as_f32()),
                        ..default()
                    }));

    commands.spawn((MapComponent,
                    SpriteBundle {
                        texture: asset_server.load("map_outer_mask.png"),
                        transform: Transform::from_xyz(0.0, 0.0, Layers::Mask.as_f32()),
                        ..default()
                    }));

    commands.spawn((ReadySign,
                    Location::new(13.5, 13.0),
                    SpriteBundle {
                        texture: asset_server.load("ready.png"),
                        transform: Transform::from_xyz(0.0, 0.0, Layers::Map.as_f32() + 1.0),
                        ..default()
                    }));
}

fn remove_ready(mut commands: Commands, query: Query<Entity, With<ReadySign>>) {
    commands.entity(query.single()).despawn();
}

fn update_entities_location(mut query: Query<(&mut Transform, &Location), Changed<Location>>) {
    query.par_iter_mut().for_each(|(mut transform, location)| {
        transform.translation.x = (location.x - 13.5) * 8.0;
        transform.translation.y = (location.y - 15.0) * 8.0;
    });
}

fn map_wrap(mut query: Query<&mut Location>, map: Res<Map>) {
    query.par_iter_mut().for_each(|mut location| {
        if location.x <= -2.0 {
            let dif = location.x + 2.0;
            location.x = map.width() as f32 + 1.0 + dif;
        } else if location.x >= (map.width() as f32 + 1.0) {
            let dif = location.x - (map.width() as f32 + 1.0);
            location.x = -2.0 + dif;
        }

        if location.y <= -2.0 {
            let dif = location.y + 2.0;
            location.y = map.height() as f32 + 1.0 + dif;
        } else if location.y == (map.height() as f32 + 1.0) {
            let dif = location.y - (map.height() as f32 + 1.0);
            location.y = -2.0 + dif;
        }
    });
}

fn flash_map(timer: Res<StateTimer>,
             mut query: Query<&mut TextureAtlasSprite, With<MapComponent>>) {
    if timer.0.elapsed_secs() >= 3.0 {
        let first_half_of_second = timer.0.elapsed().as_secs_f32().fract() < 0.5;

        let mut sprite = query.single_mut();
        sprite.index = if first_half_of_second { 1 } else { 0 };
    }
}

fn despawn(mut commands: Commands, query: Query<Entity, With<MapComponent>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

