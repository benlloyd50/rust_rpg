use bracket_terminal::prelude::{ColorPair, DrawBatch, Point, BLACK, WHITE};
use specs::{Entity, World};

use crate::{camera::get_bounding_box, components::Position};

pub struct Map {
    pub tiles: Vec<WorldTile>,
    pub tile_entities: Vec<Vec<TileEntity>>,
    pub width: usize,
    pub height: usize,
}

#[derive(Clone)]
pub struct WorldTile {
    pub atlas_index: usize,
}

/// Defines the type of entity existing in a tile for quick lookup and action handling
/// Discrimnants are the priority for action handling, lower taking priority
#[repr(u8)]
#[derive(Clone, PartialEq, PartialOrd, Ord, Eq)]
pub enum TileEntity {
    Fishable(Entity) = 9,
    Breakable(Entity) = 15,
    Item(Entity) = 19,
    Blocking = 20,
}

impl TileEntity {
    /// Attempts to grab the inner entity if it is an item
    pub fn as_item_entity(&self) -> Option<&Entity> {
        match self {
            TileEntity::Item(item) => Some(item),
            _ => None,
        }
    }
}

impl WorldTile {
    pub fn default() -> Self {
        Self { atlas_index: 4 }
    }
}

impl Map {
    pub fn empty() -> Self {
        Self {
            tiles: vec![],
            tile_entities: vec![],
            width: 0,
            height: 0,
        }
    }

    pub fn new(width: usize, height: usize) -> Self {
        Map {
            tiles: vec![WorldTile::default(); width * height],
            tile_entities: vec![vec![]; width * height],
            width,
            height,
        }
    }

    pub fn xy_to_idx(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }

    /// Gets all the entities in the tile that are an item.
    /// It returns an iterator since often only the first value is used.
    pub fn all_items_at_pos(&self, pos: &Position) -> impl Iterator<Item = &TileEntity> {
        self.tile_entities[self.xy_to_idx(pos.x, pos.y)]
            .iter()
            .filter(|te| te.as_item_entity().is_some())
    }

    /// Attempts to get the first entity at the pos based on the contents of the tile
    /// Will return `None` if no entities are present in the tile
    pub fn first_entity_in_pos(&self, pos: &Position) -> Option<&TileEntity> {
        self.tile_entities[self.xy_to_idx(pos.x, pos.y)]
            .iter()
            .min_by_key(|&tile_entity| match tile_entity {
                TileEntity::Fishable(_) => 9,
                TileEntity::Breakable(_) => 15,
                TileEntity::Item(_) => 19,
                TileEntity::Blocking => 20,
            })
    }
}

/// Renders the current map resource to the current console layer
pub fn render_map(ecs: &World, batch: &mut DrawBatch) {
    let map = ecs.fetch::<Map>();

    let bounding_box = get_bounding_box(ecs);

    for x in bounding_box.x1..bounding_box.x2 {
        for y in bounding_box.y1..bounding_box.y2 {
            let atlas_index = if x < map.width as i32 && y < map.height as i32 && x >= 0 && y >= 0 {
                map.tiles[map.xy_to_idx(x as usize, y as usize)].atlas_index
            } else {
                xy_to_idx_given_width(0, 2, 16)
            };

            let batch_x = if x > 0 {
                x + (-bounding_box.x1)
            } else {
                x - bounding_box.x1
            };
            let batch_y = if y > 0 {
                y + (-bounding_box.y1)
            } else {
                y - bounding_box.y1
            };

            batch.set(
                Point::new(batch_x, batch_y),
                ColorPair::new(WHITE, BLACK),
                atlas_index,
            );
        }
    }
}

fn xy_to_idx_given_width(x: usize, y: usize, width: usize) -> usize {
    x + y * width
}
