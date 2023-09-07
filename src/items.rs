/* Item/Inventory Workflows
 * Items go either on the floor or into an inventory
 * Destructible Rock - when broke -> Item on Floor - when picked up -> Item in Inventory
 * Enemy - when killed -> Item on Florr - ...
 * Quest - when finished -> Item in Inventory
 */

use std::fmt::Display;

use specs::{Entities, Entity, Join, ReadStorage, System, World, WorldExt, Write, WriteStorage};

use crate::{
    components::{Backpack, Item, Name, PickupAction, Position, Renderable},
    data_read::prelude::*,
    message_log::MessageLog,
    player::PlayerResponse,
    z_order::ITEM_Z,
};

pub struct ItemQty(pub usize);

impl Display for ItemQty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl ItemQty {
    pub fn new(amt: usize) -> Self {
        Self(amt)
    }

    pub fn add(&mut self, amt: usize) {
        self.0 += amt;
    }
}

#[derive(Default)]
pub struct ItemSpawner {
    requests: Vec<ItemSpawnRequest>,
}

impl ItemSpawner {
    pub fn new() -> Self {
        Self {
            requests: Vec::new(),
        }
    }

    pub fn request(&mut self, item_id: ItemID, x: usize, y: usize) {
        self.requests.push(ItemSpawnRequest {
            item_id,
            position: Position::new(x, y),
        });
    }
}

pub struct ItemSpawnRequest {
    item_id: ItemID,
    position: Position,
}

pub struct ItemSpawnerSystem;

impl<'a> System<'a> for ItemSpawnerSystem {
    type SystemData = (
        Entities<'a>,
        Write<'a, ItemSpawner>,
        WriteStorage<'a, Item>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, Renderable>,
        WriteStorage<'a, Name>,
    );

    fn run(
        &mut self,
        (entities, mut spawn_requests, mut items, mut positions, mut renderables, mut names): Self::SystemData,
    ) {
        let edb = &ENTITY_DB.lock().unwrap();

        for spawn in spawn_requests.requests.iter() {
            let static_item = match edb.items.get_by_id(spawn.item_id.0) {
                Some(val) => val,
                None => {
                    eprintln!(
                        "Spawn request failed because {:?} item id does not exist in database",
                        spawn.item_id
                    );
                    continue;
                }
            };

            let new_item = entities.create();
            let _ = positions.insert(new_item, spawn.position);
            let _ = renderables.insert(
                new_item,
                Renderable::default_bg(static_item.atlas_index, static_item.fg, ITEM_Z),
            );
            let _ = items.insert(new_item, Item);
            let _ = names.insert(new_item, Name(static_item.name.clone()));
        }

        spawn_requests.requests.clear();
    }
}

pub struct ItemPickupHandler;

impl<'a> System<'a> for ItemPickupHandler {
    type SystemData = (
        WriteStorage<'a, Position>,
        WriteStorage<'a, Backpack>,
        WriteStorage<'a, PickupAction>,
        Write<'a, MessageLog>,
        ReadStorage<'a, Item>,
        ReadStorage<'a, Name>,
    );

    fn run(
        &mut self,
        (mut positions, mut backpacks, mut pickups, mut log, items, names): Self::SystemData,
    ) {
        for (pickup, picker_name, backpack) in (&pickups, &names, &mut backpacks).join() {
            let item_target = pickup.item;
            let item_name = match names.get(item_target) {
                Some(name) => name.clone(),
                None => Name::missing_item_name(),
            };

            if !items.contains(item_target) {
                eprintln!(
                    "{:?} was not an item, it's name was {}",
                    item_target, item_name
                );
                continue;
            }
            let edb = &ENTITY_DB.lock().unwrap();
            let item_info = edb.items.get_by_name_unchecked(&item_name.0);

            if backpack.add_into_backpack(item_info.identifier, 1) {
                positions.remove(item_target);
                log.log(format!(
                    "{} picked up a {}",
                    picker_name,
                    item_name.0.to_lowercase()
                ));
                if let Some(text) = &edb.items.get_by_name_unchecked(&item_name.0).pickup_text {
                    log.enhance(text);
                }
            }
        }

        pickups.clear();
    }
}

/// Checks to see if an item is held by an entity and will return the entity associated with the
/// item if there is one.
pub fn inventory_contains(looking_for: &Name, inventory_of: &Entity, ecs: &World) -> bool {
    let bags = ecs.read_storage::<Backpack>();
    let names = ecs.read_storage::<Name>();
    let checking_inventory = match bags.get(*inventory_of) {
        Some(bag) => bag,
        None => {
            let missing_being = &Name::missing_being_name();
            let inventory_owner = names.get(*inventory_of).unwrap_or(missing_being);
            eprintln!("{} does not have a backpack component", inventory_owner);
            return false;
        }
    };

    checking_inventory.contains_named(looking_for)
}

pub fn try_item(entity_trying: &Entity, desired_idx: usize, ecs: &mut World) -> PlayerResponse {
    let backpacks = ecs.read_storage::<Backpack>();
    let mut log = ecs.write_resource::<MessageLog>();
    let items_in_bag = match backpacks.get(*entity_trying) {
        Some(items) => items,
        None => panic!("Player entity does not have a Backpack component."),
    };

    let edb = &ENTITY_DB.lock().unwrap();

    let mut curr_index = 0;
    let desired_index = desired_idx - 1;
    for (iid, qty) in items_in_bag.iter() {
        if curr_index != desired_index {
            curr_index += 1;
            continue;
        }
        if let Some(info) = edb.items.get_by_id(iid.0) {
            log.debug(format!("It is {} units of {}", qty, info.name));
            break;
        }
    }

    PlayerResponse::Waiting
}
