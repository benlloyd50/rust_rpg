use std::convert::Infallible;
use std::fs::{self, File};
use std::path::Path;

use log::{error, info};
use serde::{Deserialize, Serialize};
use specs::saveload::{DeserializeComponents, MarkedBuilder, SerializeComponents, SimpleMarker, SimpleMarkerAllocator};
#[allow(deprecated)] // must be imported so ConvertSaveload works
use specs::{
    error::NoError,
    saveload::{ConvertSaveload, Marker},
    Entity,
};
use specs::{Builder, Component, ConvertSaveload, Join, NullStorage, VecStorage, World, WorldExt};

use crate::being::BeingID;
use crate::components::{
    AttackBonus, Blocking, Breakable, Consumable, DeleteCondition, EntityStats, Equipable, EquipmentSlots, Equipped,
    Fishable, GoalMoverAI, Grass, HealthStats, InBag, Interactor, Item, LevelPersistent, Name, Position,
    RandomWalkerAI, Renderable, Water,
};
use crate::data_read::ENTITY_DB;
use crate::game_init::PlayerEntity;
use crate::map::Map;
use crate::player::Player;
use crate::ui::message_log::MessageLog;

// ripped right from https://bfnightly.bracketproductions.com/chapter_11.html
macro_rules! serialize_individually {
    ($ecs:expr, $ser:expr, $data:expr, $( $type:ty),*) => {
        $(
        SerializeComponents::<Infallible, SimpleMarker<SerializeMe>>::serialize(
            &( $ecs.read_storage::<$type>(), ),
            &$data.0,
            &$data.1,
            &mut $ser,
        )
        .unwrap();
        )*
    };
}

macro_rules! deserialize_individually {
    ($ecs:expr, $de:expr, $data:expr, $( $type:ty),*) => {
        $(
        DeserializeComponents::<Infallible, _>::deserialize(
            &mut ( &mut $ecs.write_storage::<$type>(), ),
            &mut $data.0, // entities
            &mut $data.1, // marker
            &mut $data.2, // allocater
            &mut $de,
        )
        .unwrap();
        )*
    };
}

#[derive(Component)]
#[storage(NullStorage)]
pub struct SerializeMe {}

/// Useful when trying to save things that aren't a normal component
#[derive(Component, Clone, ConvertSaveload)]
#[storage(VecStorage)]
pub struct SerializationHelper {
    map: Map,
    message_log: MessageLog,
}

pub enum SaveAction {
    Save,
    Cancel,
    Waiting,
    QuitWithoutSaving,
}

pub const SAVE_PATH: &str = "./saves/mysavegame.json";

pub fn cleanup_game(ecs: &mut World) {
    info!("Cleaning up game world.");
    ecs.delete_all();
    let mut message_log = ecs.write_resource::<MessageLog>();
    message_log.clear();
    info!("Cleaning Successful");
}

pub fn save_game_exists() -> bool {
    Path::new(SAVE_PATH).exists()
}

pub fn save_game(ecs: &mut World) {
    let map = ecs.get_mut::<Map>().unwrap().clone();
    let message_log = ecs.get_mut::<MessageLog>().unwrap().clone();
    let savehelper =
        ecs.create_entity().with(SerializationHelper { map, message_log }).marked::<SimpleMarker<SerializeMe>>().build();

    {
        let data = (ecs.entities(), ecs.read_storage::<SimpleMarker<SerializeMe>>());
        let writer = File::create(format!("{}", SAVE_PATH)).unwrap();
        let mut serializer = serde_json::Serializer::new(writer);
        #[rustfmt::skip]
        serialize_individually!(ecs, serializer, data, Position, Renderable, LevelPersistent, EntityStats, Blocking, Fishable,
                                Name, HealthStats, Breakable, DeleteCondition, Item, InBag, Consumable, Equipped, Equipable,
                                BeingID,
                                Player, EquipmentSlots, Water, Grass, Interactor, AttackBonus, SerializationHelper);
    }

    ecs.delete_entity(savehelper).expect("Crash in cleanup, hopefully we still saved.");
}
pub fn load_game(ecs: &mut World) {
    // make sure everything is wiped out
    cleanup_game(ecs);

    let save_data = match fs::read_to_string(SAVE_PATH) {
        Ok(data) => data,
        Err(e) => {
            error!("Save game file cannot be loaded from `{}`", SAVE_PATH);
            error!("Load game error: {}", e);
            return;
        }
    };

    let mut deserializer = serde_json::Deserializer::from_str(&save_data);

    {
        let mut d = (
            &mut ecs.entities(),
            &mut ecs.write_storage::<SimpleMarker<SerializeMe>>(),
            &mut ecs.write_resource::<SimpleMarkerAllocator<SerializeMe>>(),
        );
        #[rustfmt::skip]
        deserialize_individually!(ecs, deserializer, d, Position, Renderable, LevelPersistent, EntityStats, Blocking, Fishable,
                                Name, HealthStats, Breakable, DeleteCondition, Item, InBag, Consumable, Equipped, Equipable,
                                BeingID,
                                Player, EquipmentSlots, Water, Grass, Interactor, AttackBonus, SerializationHelper);
    }

    let mut delete_me = None;
    {
        let entities = ecs.entities();
        let helper = ecs.read_storage::<SerializationHelper>();
        let player = ecs.read_storage::<Player>();

        if let Some((helper_e, helper_data)) = (&entities, &helper).join().next() {
            let mut map = ecs.write_resource::<Map>();
            *map = helper_data.map.clone();
            map.tile_entities = vec![Vec::new(); map.width * map.height];

            let mut msg_log = ecs.write_resource::<MessageLog>();
            *msg_log = helper_data.message_log.clone();

            delete_me = Some(helper_e);
        } else {
            error!("No map found when loading the savegame.");
        }

        // Recreate AI for beings
        let beings = ecs.write_storage::<BeingID>();
        let edb = &ENTITY_DB.lock().unwrap();
        for (being_e, being_id) in (&entities, &beings).join() {
            match edb.beings.get_by_id(being_id.0) {
                Some(being_info) => {
                    if let Some(ai) = &being_info.ai {
                        match ai.start_mode.as_str() {
                            "random_walk" => {
                                let mut random_walk = ecs.write_storage::<RandomWalkerAI>();
                                let _ = random_walk.insert(being_e, RandomWalkerAI {});
                            }
                            "goal" => {
                                let goals = match &ai.goals {
                                    Some(goals) => {
                                        goals.iter().map(|goal| Name(goal.to_string())).collect::<Vec<Name>>()
                                    }
                                    None => {
                                        error!("{} has Goal ai type but no defined goals", &being_info.name);
                                        continue;
                                    }
                                };
                                let mut goal_movers = ecs.write_storage::<GoalMoverAI>();
                                let _ = goal_movers
                                    .insert(being_e, GoalMoverAI::with_desires(&goals, ai.goal_range.unwrap()));
                            }
                            _ => (),
                        }
                    }
                }
                None => continue,
            }
        }

        if let Some((player_e, _)) = (&entities, &player).join().next() {
            let mut player_e_res = ecs.write_resource::<PlayerEntity>();
            *player_e_res = PlayerEntity(player_e);
        } else {
            error!("No player found when loading the savegame.");
        }
    }

    ecs.delete_entity(delete_me.unwrap()).expect("Unable to delete helper after loading.");
}
