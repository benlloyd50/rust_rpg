use bracket_terminal::prelude::{BLACK, WHITE};
use specs::{Builder, World, WorldExt};

use crate::{
    components::{Blocking, Monster, Name, Position, RandomWalkerAI, Renderable, Strength},
    data_read::prelude::load_simple_ldtk_level,
    draw_sprites::debug_rocks,
    player::Player,
};

pub fn initialize_game_world(ecs: &mut World) {
    let map = load_simple_ldtk_level(ecs);
    ecs.insert(map);

    ecs.create_entity()
        .with(Position::new(17, 20))
        .with(Player)
        .with(Strength { amt: 1 })
        .with(Renderable::new(WHITE, BLACK, 2))
        .with(Blocking)
        .with(Name("Tester".to_string()))
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
