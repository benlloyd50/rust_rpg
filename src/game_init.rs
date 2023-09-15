use bracket_terminal::prelude::BLACK;
use specs::{Builder, Entity, World, WorldExt};

pub const WHITE: (u8, u8, u8) = (255, 255, 255);

use crate::{
    components::{
        Backpack, Interactor, InteractorMode, Name, Position, Renderable, Strength, Transform,
    },
    data_read::prelude::{build_being, load_simple_ldtk_level},
    player::Player,
    z_order::PLAYER_Z,
};

/// A convienent resource to access the entity associated with the player
pub struct PlayerEntity(pub Entity);

pub fn initialize_game_world(ecs: &mut World) {
    let map = load_simple_ldtk_level(ecs);
    ecs.insert(map);

    let player_entity = ecs
        .create_entity()
        .with(Position::new(67, 30))
        // .with(Transform::new(13f32, 13f32, 0f32, 1.0, 1.0))
        .with(Interactor::new(InteractorMode::Reactive))
        .with(Player)
        .with(Backpack::empty())
        .with(Strength { amt: 1 })
        .with(Renderable::new(WHITE, BLACK, 2, PLAYER_Z))
        .with(Name("Tester".to_string()))
        .build();
    ecs.insert(PlayerEntity(player_entity));

    build_being("Bahhhby", Position::new(5, 15), ecs).ok();
    let greg = build_being("Greg Goat", Position::new(12, 19), ecs).unwrap();
    let mut transforms = ecs.write_storage::<Transform>();
    let _ = transforms.insert(greg, Transform::new(12.0, 19.0, 0.0, 1.0, 1.0));
}
