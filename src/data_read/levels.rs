use crate::{map::{Map, WorldTile}, components::{Position, Blocking}, indexing::idx_to_xy};
use ldtk_map::prelude::*;
use specs::{World, WorldExt, Builder};

const LEVEL_ZERO: &'static str = "Level_0";

pub fn load_simple_ldtk_level(ecs: &mut World) -> Map {
    let ldtk_design = DesignMap::load("./resources/ldtk/rpg_world_v1.ldtk");

    let simple_level = &ldtk_design.levels()[LEVEL_ZERO];
    let mut map = Map::new(simple_level.width(), simple_level.height());

    // let positions = ecs.write_storage::<Position>();
    // let blockers = ecs.write_storage::<Blocking>();

    let mut idx = 0;
    for tile in simple_level.level() {
        map.tiles[idx] = WorldTile { atlas_index: tile.atlas_index() };

        match tile.value() {
            0 | 2 => {}
            1 => { ecs.create_entity().with(Position::from_idx(idx, simple_level.width())).with(Blocking).build(); }
            _ => println!("Value not recognized at {:#?}", idx_to_xy(idx, simple_level.width())),
        };


        idx += 1;
    }

    map
}

