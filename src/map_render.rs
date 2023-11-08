use bevy::prelude::*;

use crate::services::map::{Map, Location};

pub struct MapRenderPlugin;

impl Plugin for MapRenderPlugin {
    fn build(&self, app: &mut App) {
        const MAP_TEXT: &str = include_str!("map");

        app.insert_resource(Map::parse(MAP_TEXT));
        app.add_systems(Startup, render_map);
        app.add_systems(Update, update_entities_location);
    }
}

fn render_map(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(SpriteBundle {
        texture: asset_server.load("map.png"),
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        ..default()
    });

    commands.spawn(SpriteBundle {
        texture: asset_server.load("map_outer_mask.png"),
        transform: Transform::from_xyz(0.0, 0.0, 999.0),
        ..default()
    });
}

fn update_entities_location(mut query: Query<(&mut Transform, &Location)>) {
    query.par_iter_mut().for_each(|(mut transform, location)| {
        let vec = location.get();
        transform.translation.x = (vec.x - 13.5) * 8.0;
        transform.translation.y = (vec.y - 15.0) * 8.0;
    });
}
