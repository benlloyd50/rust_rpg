use bracket_terminal::prelude::{BLACK, WHITE};
use specs::{Builder, World, WorldExt, Entity};

use crate::{
    components::{Blocking, Name, Position, Renderable, Strength, Backpack},
    data_read::prelude::{build_being, load_simple_ldtk_level},
    player::Player,
    z_order::PLAYER_Z,
};

/// A convienent resource to access the entity associated with the player
pub struct PlayerEntity(pub Entity);

pub fn initialize_game_world(ecs: &mut World) {
    let map = load_simple_ldtk_level(ecs);
    ecs.insert(map);

    let player_entity = ecs.create_entity()
        .with(Position::new(17, 20))
        .with(Player)
        .with(Backpack::empty())
        .with(Strength { amt: 1 })
        .with(Renderable::new(WHITE, BLACK, 2, PLAYER_Z))
        .with(Blocking)
        .with(Name("Tester".to_string()))
        .build();
    ecs.insert(PlayerEntity(player_entity));

    build_being("Bahhhby", Position::new(5, 15), ecs).ok();
    build_being("Greg Goat", Position::new(12, 20), ecs).ok();
}
