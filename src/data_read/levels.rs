use crate::{
    components::{Blocking, Position, Water},
    indexing::idx_to_point,
    items::{ItemSpawner, SpawnType},
    map::{Map, WorldTile},
};
use ldtk_map::prelude::*;
use log::debug;
use specs::{Builder, World, WorldExt};

use super::{
    prelude::{build_being, build_obj},
    ENTITY_DB,
};

const LEVEL_ZERO: &str = "Level_0";

pub fn load_simple_ldtk_level(ecs: &mut World) -> Map {
    let ldtk_design = DesignMap::load("./resources/ldtk/rpg_world_v1.ldtk");
    let simple_level = &ldtk_design.levels()[LEVEL_ZERO];
    let mut map = Map::new(simple_level.width(), simple_level.height());

    debug!("going into loop");
    for (idx, tile) in simple_level.level().iter().enumerate() {
        map.tiles[idx] = WorldTile {
            atlas_index: tile.atlas_index(),
        };

        if let Some(name) = tile.entity_name() {
            if let Some(tag) = tile.entity_tag() {
                match tag {
                    "Item" => {
                        let edb = &ENTITY_DB.lock().unwrap();
                        let mut spawner = ecs.write_resource::<ItemSpawner>();
                        spawner.request(
                            edb.items.get_by_name(name).unwrap().identifier,
                            SpawnType::OnGround(idx_to_point(idx, map.width).into()),
                        );
                        // let _ = build_item(name, Some(idx_to_point(idx, map.width).into()), ecs);
                    }
                    "Interactable" => {
                        let _ = build_obj(name, idx_to_point(idx, map.width).into(), ecs);
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
                ecs.create_entity()
                    .with(Position::from_idx(idx, simple_level.width()))
                    .with(Blocking)
                    .build();
            }
            2 => {
                ecs.create_entity()
                    .with(Position::from_idx(idx, simple_level.width()))
                    .with(Water)
                    .with(Blocking)
                    .build();
            }
            _ => eprintln!(
                "Value not recognized at {:#?}",
                idx_to_point(idx, simple_level.width())
            ),
        };
    }

    map
}
