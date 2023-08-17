use bracket_terminal::prelude::{BLACK, WHITE};
use specs::{Builder, World, WorldExt};

use crate::{
    components::{
        Blocking, Fishable, Monster, Name, Position, RandomWalkerAI, Renderable, Strength,
    },
    draw_sprites::debug_rocks,
    map::{Map, WorldTile},
    player::Player,
    DISPLAY_HEIGHT, DISPLAY_WIDTH,
};

pub fn initialize_game_world(ecs: &mut World) {
    // A very plain map
    let mut map = Map::new(DISPLAY_WIDTH, DISPLAY_HEIGHT - 3);
    let water_idx = map.xy_to_idx(10, 15);
    map.tiles[water_idx] = WorldTile { atlas_index: 80 };
    ecs.create_entity()
        .with(Position::new(10, 15))
        .with(Fishable)
        .with(Blocking)
        .build();

    ecs.insert(map);

    ecs.create_entity()
        .with(Position::new(17, 20))
        .with(Player)
        .with(Strength { amt: 1 })
        .with(Renderable::new(WHITE, BLACK, 2))
        .with(Blocking)
        .build();

    ecs.create_entity()
        .with(Position::new(5, 15))
        .with(Monster)
        .with(Name::new("Bahhhby"))
        .with(RandomWalkerAI)
        .with(Renderable::new(WHITE, BLACK, 16))
        .with(Blocking)
        .build();

    debug_rocks(ecs);
}
