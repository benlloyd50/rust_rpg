use crate::map::{Map, WorldTile};
use bracket_terminal::prelude::Point;
use ldtk_map::prelude::*;

const LEVEL_ZERO: &'static str = "Level_0";

pub fn load_simple_ldtk_level() -> Map {
    let ldtk_design = DesignMap::load("./resources/ldtk/rpg_world_v1.ldtk");

    let simple_level = &ldtk_design.levels()[LEVEL_ZERO];
    let mut ldtk_level = Map::new(simple_level.width(), simple_level.height());

    let mut idx = 0;
    for tile in simple_level.level() {
        ldtk_level.tiles[idx] = WorldTile {atlas_index: tile.atlas_index() };
        idx += 1;
    }

    // println!("{:#?}", ldtk_design);
    ldtk_level
}

#[allow(dead_code)]
fn idx_to_xy(idx: usize, width: usize) -> Point {
    let x = idx / width;
    let y = idx % width;

    Point::new(x, y)
}
