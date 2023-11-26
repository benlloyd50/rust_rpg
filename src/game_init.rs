use bracket_terminal::prelude::BLACK;
use log::debug;
use specs::{Builder, Entity, World, WorldExt};

pub const WHITE: (u8, u8, u8) = (255, 255, 255);

use crate::{
    components::{
        EquipmentSlots, Interactor, InteractorMode, ItemContainer, Name, Position, Renderable,
        Transform,
    },
    data_read::prelude::{build_being, load_simple_ldtk_level},
    items::{ItemID, ItemSpawner, SpawnType},
    player::Player,
    stats::get_random_stats,
    z_order::PLAYER_Z,
};

/// A convienent resource to access the entity associated with the player
pub struct PlayerEntity(pub Entity);

impl Default for PlayerEntity {
    fn default() -> Self {
        panic!("Dont call default on player_entity")
    }
}

pub fn initialize_game_world(ecs: &mut World) {
    debug!("startup: map loading");
    let map = load_simple_ldtk_level(ecs);
    ecs.insert(map);
    debug!("startup: map loaded");

    let player_stats = get_random_stats();
    let player_entity = ecs
        .create_entity()
        .with(Position::new(67, 30))
        // .with(Transform::new(13f32, 13f32, 0f32, 1.0, 1.0))
        .with(Interactor::new(InteractorMode::Reactive))
        .with(Player)
        .with(ItemContainer::new(10))
        .with(EquipmentSlots::human())
        .with(player_stats)
        .with(player_stats.set.get_health_stats())
        .with(Renderable::new(WHITE, BLACK, 2, PLAYER_Z))
        .with(Name("Player".to_string()))
        .build();
    ecs.insert(PlayerEntity(player_entity));
    debug!("startup: player loaded");

    {
        let mut item_spawner = ecs.write_resource::<ItemSpawner>();
        item_spawner.request(ItemID(201), SpawnType::InBag(player_entity));
    }

    build_being("Bahhhby", Position::new(5, 15), ecs).ok();
    let greg = build_being("Greg Goat", Position::new(12, 19), ecs).unwrap();
    let mut transforms = ecs.write_storage::<Transform>();
    let _ = transforms.insert(greg, Transform::new(12.0, 19.0, 0.0, 1.0, 1.0));
    debug!("startup: sample beings loaded");
}
