use crate::{
    components::{Blocking, Position, Water},
    indexing::idx_to_point,
    items::{ItemSpawner, SpawnType},
    map::{Map, WorldTile},
    FONT_TERRAIN_FOREST, FONT_TERRAIN_TOWN_FOREST,
};
use ldtk_map::prelude::*;
use log::{debug, info};
use specs::{Builder, World, WorldExt};

use super::prelude::{build_being, build_world_obj};

pub const LDTK_FILE: &str = "./resources/ldtk/rpg_world_v2.ldtk";

pub fn create_map(ecs: &mut World, level_name: &str) -> Map {
    let ldtk_design = DesignMap::load(LDTK_FILE); //note: loads all levels in file
    let new_level = &ldtk_design.levels()[level_name];
    let mut map = Map::new(new_level.width(), new_level.height(), new_level.world_xy());
    info!("{}", new_level.tileset_name());
    map.tile_atlas_index = match new_level.tileset_name().to_lowercase().as_str() {
        "terrain_town_forest" => FONT_TERRAIN_TOWN_FOREST,
        "terrain_forest" => FONT_TERRAIN_FOREST,
        _ => FONT_TERRAIN_FOREST,
    };

    for (idx, tile) in new_level.level().iter().enumerate() {
        map.tiles[idx] = WorldTile { atlas_index: tile.atlas_index() };

        if let Some(name) = tile.entity_name() {
            if let Some(tag) = tile.entity_tag() {
                debug!("spawning in a {}", name);
                match tag {
                    "Item" => {
                        let mut spawner = ecs.write_resource::<ItemSpawner>();
                        spawner.request_named(name, SpawnType::OnGround(idx_to_point(idx, map.width).into()));
                    }
                    "Interactable" => {
                        let _ = build_world_obj(name, idx_to_point(idx, map.width).into(), ecs);
                    }
                    "Being" => {
                        let _ = build_being(name, idx_to_point(idx, map.width).into(), ecs);
                    }
                    _ => eprintln!("invalid tag on entity"),
                }
            }
        }

        match tile.value() {
            0 => {}
            1 => {
                ecs.create_entity().with(Position::from_idx(idx, new_level.width())).with(Blocking).build();
            }
            2 => {
                ecs.create_entity().with(Position::from_idx(idx, new_level.width())).with(Water).with(Blocking).build();
            }
            _ => eprintln!("Value not recognized at {:#?}", idx_to_point(idx, new_level.width())),
        };
    }

    map
}
