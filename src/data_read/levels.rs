use crate::{
    components::{Blocking, Fishable, Position},
    indexing::idx_to_xy,
    map::{Map, WorldTile},
};
use ldtk_map::prelude::*;
use specs::{Builder, World, WorldExt};

use super::{items::build_item, prelude::build_obj};

const LEVEL_ZERO: &'static str = "Level_0";

pub fn load_simple_ldtk_level(ecs: &mut World) -> Map {
    let ldtk_design = DesignMap::load("./resources/ldtk/rpg_world_v1.ldtk");

    let simple_level = &ldtk_design.levels()[LEVEL_ZERO];
    let mut map = Map::new(simple_level.width(), simple_level.height());

    let mut idx = 0;
    for tile in simple_level.level() {
        map.tiles[idx] = WorldTile {
            atlas_index: tile.atlas_index(),
        };

        if let Some(name) = tile.entity_name() {
            if let Some(tag) = tile.entity_tag() {
                match tag {
                    "Item" => {
                        let _ = build_item(name, idx_to_xy(idx, map.width).into(), ecs);
                    }
                    "Interactable" => {
                        let _ = build_obj(name, idx_to_xy(idx, map.width).into(), ecs);
                    }
                    _ => eprintln!("invalid tag on entity"),
                }
            }
        }

        match tile.value() {
            0 => {}
            1 => {
                ecs.create_entity()
                    .with(Position::from_idx(idx, simple_level.width()))
                    .with(Blocking)
                    .build();
            }
            2 => {
                ecs.create_entity()
                    .with(Position::from_idx(idx, simple_level.width()))
                    .with(Fishable)
                    .with(Blocking)
                    .build();
            }
            _ => println!(
                "Value not recognized at {:#?}",
                idx_to_xy(idx, simple_level.width())
            ),
        };

        idx += 1;
    }

    map
}
