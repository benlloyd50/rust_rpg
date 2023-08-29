/* Item/Inventory Workflows
 * Items go either on the floor or into an inventory
 * Destructible Rock - when broke -> Item on Floor - when picked up -> Item in Inventory
 * Enemy - when killed -> Item on Florr - ...
 * Quest - when finished -> Item in Inventory
 */

use specs::{Entities, Entity, Join, ReadStorage, System, World, WorldExt, Write, WriteStorage};

use crate::{
    components::{InBackpack, Item, Name, PickupAction, Position, Renderable},
    data_read::prelude::*,
    message_log::MessageLog,
    z_order::ITEM_Z,
};

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
        WriteStorage<'a, InBackpack>,
        WriteStorage<'a, PickupAction>,
        Write<'a, MessageLog>,
        Entities<'a>,
        ReadStorage<'a, Item>,
        ReadStorage<'a, Name>,
    );

    fn run(
        &mut self,
        (mut positions, mut backpacks, mut pickups, mut log, entities, items, names): Self::SystemData,
    ) {
        for (e, pickup, picker_name) in (&entities, &pickups, &names).join() {
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

            match backpacks.insert(item_target, InBackpack::of(e)) {
                Ok(maybe_prev_owner) => match maybe_prev_owner {
                    Some(prev_owner) => {
                        let prev_owner_name = names.get(prev_owner.owner).unwrap();
                        eprintln!(
                            "Item {} is already in {}'s backpack. How did {} pick it up?",
                            item_name, prev_owner_name, picker_name
                        );
                    }
                    None => {
                        let edb = &ENTITY_DB.lock().unwrap();
                        // Valid pickup from picker
                        positions.remove(item_target);
                        log.log(format!(
                            "{} picked up a {}",
                            picker_name,
                            item_name.0.to_lowercase()
                        ));
                        if let Some(text) =
                            &edb.items.get_by_name_unchecked(&item_name.0).pickup_text
                        {
                            log.enhance(text);
                        }
                    }
                },
                Err(err) => eprintln!("{}", err),
            }
        }

        pickups.clear();
    }
}

/// Checks to see if an item is held by an entity and will return the entity associated with the
/// item if there is one.
pub fn inventory_contains(name: &Name, inventory_of: &Entity, ecs: &World) -> Option<Entity> {
    let items = ecs.read_storage::<Item>();
    let in_pack = ecs.read_storage::<InBackpack>();
    let names = ecs.read_storage::<Name>();
    let entities = ecs.entities();

    for (item_entity, _, bag, item_name) in (&entities, &items, &in_pack, &names).join() {
        if bag.owner.eq(inventory_of) && item_name.eq(name) {
            return Some(item_entity);
        }
    }

    None
}
