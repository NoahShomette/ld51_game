use bevy_ecs_tilemap::prelude::*;
use bevy::{prelude::*, render::texture::ImageSettings};

pub fn setup_map(mut commands: Commands, asset_server: Res<AssetServer>) {
    
    let texture_handle: Handle<Image> = asset_server.load("grass_tile.png");

    let tilemap_size = TilemapSize { x: 32, y: 32 };
    let tilemap_entity = commands.spawn().id();
    let mut tile_storage = TileStorage::empty(tilemap_size);

    for x in 0..tilemap_size.x {
        for y in 0..tilemap_size.y {
            let tile_pos = TilePos { x, y };
            let tile_entity = commands
                .spawn()
                .insert_bundle(TileBundle {
                    position: tile_pos,
                    tilemap_id: TilemapId(tilemap_entity),
                    ..Default::default()
                })
                .id();
            tile_storage.set(&tile_pos, Option::from(tile_entity));
        }
    }
    
    let tile_size = TilemapTileSize { x: 64.0, y: 64.0 };
    let grid_size = tile_size.into();

    commands
        .entity(tilemap_entity)
        .insert_bundle(TilemapBundle {
            grid_size,
            size: tilemap_size,
            storage: tile_storage,
            texture: TilemapTexture(texture_handle),
            tile_size,
            //transform: get_tilemap_center_transform(&tilemap_size, &grid_size, 0.0),
            ..Default::default()
        });
}
