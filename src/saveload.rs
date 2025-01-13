use std::convert::Infallible;
use std::fs::{self, create_dir, File};
use std::path::Path;

use log::{debug, error, info, warn};
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
    RandomWalkerAI, Renderable, Viewshed, Water,
};
use crate::data_read::ENTITY_DB;
use crate::game_init::PlayerEntity;
use crate::map::{Map, MapRes};
use crate::player::Player;
use crate::saveload_menu::LoadedWorld;
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
    QuickSave,
    Cancel,
    Waiting,
    QuitWithoutSaving,
}

pub const SAVE_PATH: &str = "./saves/";

pub fn cleanup_game(ecs: &mut World) {
    info!("Cleaning up game world.");
    ecs.delete_all();
    let mut message_log = ecs.write_resource::<MessageLog>();
    message_log.clear();
    info!("Cleaning Successful");
}

// TODO: save game check like this won't work anymore, should check existence of save game files
pub fn save_game_exists() -> bool {
    Path::new(SAVE_PATH).exists()
}

pub fn save_game(ecs: &mut World) {
    let MapRes(map) = ecs.get_mut::<MapRes>().unwrap().clone();
    let message_log = ecs.get_mut::<MessageLog>().unwrap().clone();
    let savehelper = ecs
        .create_entity()
        .with(SerializationHelper { map, message_log })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();

    let lw = ecs.get_mut::<LoadedWorld>().unwrap();
    let full_file_path = format!("{}{}", SAVE_PATH, lw.file_name.clone().unwrap_or("default.edo".to_string()));

    let writer = match File::create(&full_file_path) {
        Ok(w) => w,
        Err(e) => {
            warn!("Failed to create file trying to create directory and try again. Error: {}", e);
            let _ = create_dir(SAVE_PATH);
            match File::create(&full_file_path) {
                Ok(w) => w,
                Err(e) => {
                    error!("Could not save file successfully, {}", e);
                    return;
                }
            }
        }
    };
    {
        let data = (ecs.entities(), ecs.read_storage::<SimpleMarker<SerializeMe>>());

        let mut serializer = serde_json::Serializer::new(writer);
        #[rustfmt::skip]
        serialize_individually!(ecs, serializer, data, Position, Renderable, LevelPersistent, EntityStats, Blocking, Fishable,
                                Name, HealthStats, Breakable, DeleteCondition, Item, InBag, Consumable, Equipped, Equipable,
                                BeingID, Viewshed,
                                Player, EquipmentSlots, Water, Grass, Interactor, AttackBonus, SerializationHelper);
    }
    info!("Game was saved");

    ecs.delete_entity(savehelper).expect("Crash in cleanup, hopefully we still saved.");
}
pub fn load_game(ecs: &mut World, file_name: String) {
    // make sure everything is wiped out
    cleanup_game(ecs);

    // TODO: keep track of file_name for saving
    let save_game_path = format!("{}{}", SAVE_PATH, file_name);
    let save_data = match fs::read_to_string(&save_game_path) {
        Ok(data) => data,
        Err(e) => {
            error!("Save game file cannot be loaded from `{}`", &save_game_path);
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
                                BeingID, Viewshed,
                                Player, EquipmentSlots, Water, Grass, Interactor, AttackBonus, SerializationHelper);
    }

    // This is going to be replaced when it gets loaded below but it cannot be inserted in there
    // since some borrowing is going on
    let temp = ecs.create_entity().build();
    ecs.insert(PlayerEntity(temp));

    let mut delete_me = None;
    {
        let entities = ecs.entities();
        let helper = ecs.read_storage::<SerializationHelper>();
        let player = ecs.read_storage::<Player>();

        if let Some((helper_e, helper_data)) = (&entities, &helper).join().next() {
            let mut map = ecs.write_resource::<MapRes>();
            *map = MapRes(helper_data.map.clone());
            map.0.tile_entities = vec![Vec::new(); map.0.width * map.0.height];

            let mut msg_log = ecs.write_resource::<MessageLog>();
            *msg_log = helper_data.message_log.clone();
            debug!("Message and map loaded Successful");

            delete_me = Some(helper_e);
        } else {
            error!("No map found when loading the savegame.");
        }

        // Recreate AI components for beings
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
                                        warn!("{} has Goal ai type but no defined goals", &being_info.name);
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
            debug!("Player res set Successful");
        } else {
            error!("No player found when loading the savegame. Resulting to temp player variable.");
        }
    }

    ecs.insert(LoadedWorld { file_name: Some(file_name), ..Default::default() });
    debug!("Loading game complete");
    ecs.delete_entity(delete_me.unwrap()).expect("Unable to delete helper after loading.");
}
