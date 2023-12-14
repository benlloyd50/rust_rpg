use bracket_terminal::prelude::BTerm;
use ldtk_map::prelude::DesignMap;
use log::debug;
use specs::{
    saveload::{MarkedBuilder, SimpleMarker},
    Builder, Entity, Join, World, WorldExt,
};

pub const WHITE: (u8, u8, u8) = (255, 255, 255);

use crate::{
    components::{EquipmentSlots, Interactor, InteractorMode, LevelPersistent, Name, Position, Renderable, Transform},
    data_read::prelude::{build_being, create_map, LDTK_FILE},
    items::{ItemID, ItemSpawner, SpawnType},
    map::Map,
    player::Player,
    saveload::SerializeMe,
    stats::get_random_stats,
    z_order::PLAYER_Z,
    CL_WORLD,
};

/// A convenient resource to access the entity associated with the player
pub struct PlayerEntity(pub Entity);

impl Default for PlayerEntity {
    fn default() -> Self {
        panic!("Dont call default on player_entity")
    }
}

const LEVEL_ZERO: &str = "Level_0";

pub fn initialize_new_game_world(ecs: &mut World, ctx: &mut BTerm) {
    debug!("startup: map loading");
    load_map(LEVEL_ZERO, ecs, ctx);
    debug!("startup: map loaded");

    let mut player_stats = get_random_stats();
    player_stats.set.vitality = 25;
    player_stats.set.strength = 75;
    let player_entity = ecs
        .create_entity()
        .with(Position::new(67, 30))
        .with(Interactor::new(InteractorMode::Reactive))
        .with(Player {})
        .with(EquipmentSlots::human())
        .with(player_stats)
        .with(player_stats.set.get_health_stats())
        .with(Renderable::clear_bg(2, WHITE, PLAYER_Z))
        .with(Name("Player".to_string()))
        .with(LevelPersistent {})
        .marked::<SimpleMarker<SerializeMe>>()
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

pub fn cleanup_old_map(ecs: &mut World) {
    let mut remove_me = Vec::new();
    {
        let persistent_objs = ecs.read_storage::<LevelPersistent>();
        let entities = ecs.entities();

        for (e, _) in (&entities, !&persistent_objs).join() {
            remove_me.push(e);
        }
    }

    let _ = ecs.delete_entities(&remove_me);
}

pub fn move_player_to(world_pos: &Position, ecs: &mut World) {
    let mut positions = ecs.write_storage::<Position>();
    let player_e = ecs.read_resource::<PlayerEntity>();
    let map = ecs.read_resource::<Map>();
    let local_pos = Position::new(world_pos.x - map.world_x(), world_pos.y - map.world_y());
    let _ = positions.insert(player_e.0, local_pos);
}

pub fn load_map(level: &str, ecs: &mut World, ctx: &mut BTerm) {
    let map = create_map(ecs, level);
    ctx.set_active_console(CL_WORLD);
    ctx.set_active_font(map.tile_atlas_index, false);
    ecs.insert(map);
}

pub fn find_next_map(pos: &Position) -> Option<String> {
    let ldtk_design = DesignMap::load(LDTK_FILE); //note: loads all levels in file
    ldtk_design
        .levels()
        .values()
        .find(|level| {
            (level.world_tile_x()..level.world_tile_x() + level.width()).contains(&pos.x)
                && (level.world_tile_y()..level.world_tile_y() + level.height()).contains(&pos.y)
        })
        .and_then(|level| Some(level.name().to_string()))
}
