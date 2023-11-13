/* Item/Inventory Workflows
 * Items go either on the floor or into an inventory
 * Destructible Rock - when broke -> Item on Floor - when picked up -> Item in Inventory
 * Enemy - when killed -> Item on Florr - ...
 * Quest - when finished -> Item in Inventory
 */

use std::fmt::Display;

use specs::{Entities, Entity, Join, ReadStorage, System, World, Write, WriteStorage};

use crate::{
    components::{Equipable, EquipmentSlot, InBag, Item, Name, PickupAction, Position, Renderable},
    data_read::prelude::*,
    ui::message_log::MessageLog,
    z_order::ITEM_Z,
};

pub struct ItemQty(pub usize);

impl Display for ItemQty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[allow(dead_code)]
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

    pub fn request(&mut self, item_id: ItemID, spawn_type: SpawnType) {
        self.requests.push(ItemSpawnRequest {
            item_id,
            spawn_type,
        });
    }
}

pub struct ItemSpawnRequest {
    item_id: ItemID,
    spawn_type: SpawnType,
}

pub enum SpawnType {
    OnGround(Position),
    InBag(Entity),
}

pub struct ItemSpawnerSystem;

impl<'a> System<'a> for ItemSpawnerSystem {
    type SystemData = (
        Entities<'a>,
        Write<'a, ItemSpawner>,
        WriteStorage<'a, Item>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, Renderable>,
        WriteStorage<'a, InBag>,
        WriteStorage<'a, Name>,
        WriteStorage<'a, Equipable>,
    );

    fn run(
        &mut self,
        (
            entities,
            mut spawn_requests,
            mut items,
            mut positions,
            mut renderables,
            mut in_bags,
            mut names,
            mut equipables,
        ): Self::SystemData,
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
            // TODO: duplicated in data_read/items.rs

            let new_item = entities.create();
            match spawn.spawn_type {
                SpawnType::OnGround(pos) => {
                    let _ = positions.insert(new_item, pos);
                }
                SpawnType::InBag(owner) => {
                    let _ = in_bags.insert(new_item, InBag { owner });
                }
            }

            if let Some(equipable) = &static_item.equipable {
                let slot = match equipable.as_str() {
                    "Hand" => EquipmentSlot::Hand,
                    "Torso" => EquipmentSlot::Torso,
                    "Head" => EquipmentSlot::Head,
                    "Legs" => EquipmentSlot::Legs,
                    "Feet" => EquipmentSlot::Feet,
                    "Tail" => EquipmentSlot::Tail,
                    _ => {
                        eprintln!(
                            "{} is not a valid name for an equipment slot, using Head instead",
                            equipable
                        );
                        EquipmentSlot::Head
                    }
                };
                let _ = equipables.insert(new_item, Equipable { slot });
            }

            let _ = renderables.insert(
                new_item,
                Renderable::default_bg(static_item.atlas_index, static_item.fg, ITEM_Z),
            );
            let _ = items.insert(new_item, Item(spawn.item_id));
            let _ = names.insert(new_item, Name(static_item.name.clone()));
        }

        spawn_requests.requests.clear();
    }
}

pub struct ItemPickupHandler;

impl<'a> System<'a> for ItemPickupHandler {
    type SystemData = (
        WriteStorage<'a, Position>,
        WriteStorage<'a, PickupAction>,
        WriteStorage<'a, InBag>,
        Write<'a, MessageLog>,
        ReadStorage<'a, Item>,
        ReadStorage<'a, Name>,
        Entities<'a>,
    );

    fn run(
        &mut self,
        (mut positions, mut pickups, mut inbags, mut log, items, names, entities): Self::SystemData,
    ) {
        for (picker, pickup, picker_name) in (&entities, &pickups, &names).join() {
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

            // TODO: check inventory capacity and handle this result better
            let _ = inbags.insert(item_target, InBag { owner: picker });
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

        pickups.clear();
    }
}

/// Checks to see if an item is held by an entity and will return the entity associated with the
/// item if there is one.
pub fn inventory_contains(_looking_for: &Name, _inventory_of: &Entity, _ecs: &World) -> bool {
    // TODO: check inv contains item
    true
}
