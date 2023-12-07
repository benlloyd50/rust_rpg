use crate::{camera::get_camera_bounds, components::{Position, HealthStats}, droptables::Drops};
use bracket_terminal::prelude::{ColorPair, DrawBatch, Point, BLACK};
use specs::{Entity, World};

pub const WHITE: (u8, u8, u8) = (255, 255, 255);

pub struct Map {
    pub tiles: Vec<WorldTile>,
    pub tile_entities: Vec<Vec<TileEntity>>,
    pub width: usize,
    pub height: usize,
    pub world_coords: Position,
    pub tile_atlas_index: usize,
}

#[derive(Clone)]
pub struct WorldTile {
    pub atlas_index: usize,
}

impl WorldTile {
    pub fn default() -> Self {
        Self { atlas_index: 4 }
    }
}

#[derive(Debug)]
pub struct ObjectID(pub usize);

pub struct WorldObject {
    /// Unique id to find the world object's static data
    pub identifier: ObjectID,
    pub name: String,
    pub atlas_index: u8,
    pub is_blocking: bool,
    pub breakable: Option<String>,
    pub health_stats: Option<HealthStats>,
    pub grass: Option<String>,
    pub foreground: Option<(u8, u8, u8)>,
    pub loot: Option<Drops>,
}

/// Defines the type of entity existing in a tile for quick lookup and action handling
/// Discrimnants are the priority for action handling, lower taking priority
#[repr(u8)]
#[derive(Clone, PartialEq, PartialOrd, Ord, Eq, Debug)]
pub enum TileEntity {
    Fishable(Entity) = 9,
    Breakable(Entity) = 15,
    Item(Entity) = 19,
    Blocking(Entity) = 20,
}

impl TileEntity {
    /// Attempts to grab the inner entity if it is an item
    pub fn as_item_entity(&self) -> Option<&Entity> {
        match self {
            TileEntity::Item(item) => Some(item),
            _ => None,
        }
    }

    /// Tests if a `tile_entity` is Blocking variant
    pub fn is_blocker(&self) -> bool {
        matches!(self, TileEntity::Blocking(_))
    }
}

impl Map {
    pub fn empty() -> Self {
        Self {
            tiles: vec![],
            tile_entities: vec![],
            width: 0,
            height: 0,
            world_coords: Position::zero(),
            tile_atlas_index: 0,
        }
    }

    pub fn new(width: usize, height: usize, world_coords: (usize, usize)) -> Self {
        Map {
            tiles: vec![WorldTile::default(); width * height],
            tile_entities: vec![vec![]; width * height],
            width,
            height,
            world_coords: world_coords.into(),
            tile_atlas_index: 0,
        }
    }

    pub fn xy_to_idx(&self, x: usize, y: usize) -> usize {
        xy_to_idx_given_width(x, y, self.width)
    }
    
    pub fn world_x(&self) -> usize {
        self.world_coords.x
    }

    pub fn world_y(&self) -> usize {
        self.world_coords.y
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
                TileEntity::Blocking(_) => 20,
                TileEntity::Item(_) => 21,
            })
    }

    /// Checks a position on the map to see if it is blocked
    pub fn is_blocked(&self, pos: &Position) -> bool {
        self.tile_entities[self.xy_to_idx(pos.x, pos.y)]
            .iter()
            .any(|te| te.is_blocker())
    }

    pub fn in_bounds(&self, pos: Point) -> bool {
        pos.x >= 0 && pos.x < self.width as i32 && pos.y >= 0 && pos.y < self.height as i32
    }
}

pub fn successors(map: &Map, curr: &Position) -> Vec<(Position, u32)> {
    let (x, y) = (curr.x as i32, curr.y as i32);
    let mut successors = Vec::new();

    let valid_directions = [(0, 1), (1, 0), (0, -1), (-1, 0)];

    for &(dx, dy) in &valid_directions {
        let new_x = x + dx;
        let new_y = y + dy;

        // Check if the new position is within bounds and not blocked
        if map.in_bounds(Point::new(new_x, new_y)) {
            let new_pos = Position::new(new_x as usize, new_y as usize);
            if !map.is_blocked(&new_pos) {
                successors.push((new_pos, 1));
            }
        }
    }

    successors
}

pub fn distance(lhs: &Position, rhs: &Position) -> u32 {
    lhs.x.abs_diff(rhs.x) as u32 + lhs.y.abs_diff(rhs.y) as u32
}

pub fn is_goal(curr_pos: &Position, dest_pos: &Position) -> bool {
    curr_pos == dest_pos
}

/// Renders the current map resource to the current console layer
pub fn render_map(ecs: &World, batch: &mut DrawBatch) {
    let map = ecs.fetch::<Map>();

    let bounding_box = get_camera_bounds(ecs);

    for x in bounding_box.x1..bounding_box.x2 {
        for y in bounding_box.y1..bounding_box.y2 {
            let atlas_index = if x < map.width as i32 && y < map.height as i32 && x >= 0 && y >= 0 {
                map.tiles[map.xy_to_idx(x as usize, y as usize)].atlas_index
            } else {
                xy_to_idx_given_width(0, 2, 16)
            };

            let screen_x = x - bounding_box.x1;
            let screen_y = y - bounding_box.y1;

            batch.set(
                Point::new(screen_x, screen_y),
                ColorPair::new(WHITE, BLACK),
                atlas_index,
            );
        }
    }
}

/// If the struct requires the ability to convert xy to 1d idx make it call this
pub fn xy_to_idx_given_width(x: usize, y: usize, width: usize) -> usize {
    x + y * width
}
