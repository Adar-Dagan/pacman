use bevy::prelude::*;

use crate::services::map::{Map, Location}; 
use crate::common::sets::GameLoop;
use crate::common::layers::Layers;

pub struct MapRenderPlugin;

impl Plugin for MapRenderPlugin {
    fn build(&self, app: &mut App) {
        const MAP_TEXT: &str = include_str!("map");

        app.insert_resource(Map::parse(MAP_TEXT));
        app.add_systems(Startup, render_map);
        app.add_systems(FixedUpdate, map_wrap.after(GameLoop::Movement)
                                             .before(GameLoop::Collisions));
        app.add_systems(Update, update_entities_location);
    }
}

fn render_map(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(SpriteBundle {
        texture: asset_server.load("map.png"),
        transform: Transform::from_xyz(0.0, 0.0, Layers::Map.as_f32()),
        ..default()
    });

    commands.spawn(SpriteBundle {
        texture: asset_server.load("map_outer_mask.png"),
        transform: Transform::from_xyz(0.0, 0.0, Layers::Mask.as_f32()),
        ..default()
    });
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
