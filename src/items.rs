/* Item/Inventory Workflows
 * Items go either on the floor or into an inventory
 * Destructible Rock - when broke -> Item on Floor - when picked up -> Item in Inventory
 * Enemy - when killed -> Item on Floor - ...
 * Quest - when finished -> Item in Inventory
 */

use std::{
    fmt::Display,
    ops::{Add, Sub},
};

use serde::{Deserialize, Serialize};
use specs::{
    Builder, Entities, Entity, Join, ReadStorage, System, World, WorldExt, Write, WriteStorage,
};

use crate::{
    components::{AttackBonus, Equipable, InBag, Item, Name, PickupAction, Position, Renderable},
    data_read::{prelude::*, EntityBuildError},
    ui::message_log::MessageLog,
    z_order::ITEM_Z,
};

#[derive(Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ItemQty(pub usize);

impl Display for ItemQty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Add for ItemQty {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl Sub for ItemQty {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
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

    pub fn request(&mut self, id: ItemID, spawn_type: SpawnType) {
        self.requests.push(ItemSpawnRequest {
            id,
            qty: ItemQty(1),
            spawn_type,
        });
    }

    #[allow(dead_code)]
    pub fn request_amt(&mut self, id: ItemID, spawn_type: SpawnType, qty: ItemQty) {
        self.requests.push(ItemSpawnRequest {
            id,
            qty,
            spawn_type,
        });
    }
}

pub struct ItemSpawnRequest {
    id: ItemID,
    qty: ItemQty,
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
        WriteStorage<'a, AttackBonus>,
    );

    fn run(
        &mut self,
        (
            entities,
            mut spawn_requests,
            mut items,
            mut positions,
            mut renderables,
            mut inbags,
            mut names,
            mut equipables,
            mut attack_bonus,
        ): Self::SystemData,
    ) {
        let edb = &ENTITY_DB.lock().unwrap();

        for spawn in spawn_requests.requests.iter() {
            let static_item = match edb.items.get_by_id(spawn.id) {
                Some(val) => val,
                None => {
                    eprintln!(
                        "Spawn request failed because {:?} item id does not exist in database",
                        spawn.id
                    );
                    continue;
                }
            };

            let new_item = entities.create();
            match spawn.spawn_type {
                SpawnType::OnGround(pos) => {
                    let _ = positions.insert(new_item, pos);
                    let _ = items.insert(new_item, Item::new(spawn.id, spawn.qty));
                }
                SpawnType::InBag(owner) => {
                    match (&entities, &items, &inbags)
                        .join()
                        .find(|(_, item, bag)| bag.owner == owner && item.id == spawn.id)
                    {
                        Some((bagged_entity, bagged_item, _)) => {
                            let _ = items.insert(
                                bagged_entity,
                                Item::new(bagged_item.id, bagged_item.qty + spawn.qty),
                            );
                        }
                        None => {
                            let _ = items.insert(new_item, Item::new(spawn.id, spawn.qty));
                            let _ = inbags.insert(new_item, InBag { owner });
                        }
                    }
                }
            }

            // TODO: duplicated in data_read/items.rs
            if let Some(equipable) = &static_item.equipable {
                let _ = equipables.insert(new_item, equipable.clone());
            }

            if let Some(attack) = &static_item.attack_bonus {
                let _ = attack_bonus.insert(new_item, attack.clone());
            }

            let _ = renderables.insert(
                new_item,
                Renderable::default_bg(static_item.atlas_index, static_item.fg, ITEM_Z),
            );
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
        WriteStorage<'a, Item>,
        Write<'a, MessageLog>,
        ReadStorage<'a, Name>,
        Entities<'a>,
    );

    fn run(
        &mut self,
        (mut positions, mut pickup_actions, mut inbags, mut items, mut log, names, entities): Self::SystemData,
    ) {
        for (picker, pickup, picker_name) in (&entities, &pickup_actions, &names).join() {
            let ground_entity = pickup.item;
            let item_name = match names.get(ground_entity) {
                Some(name) => name.clone(),
                None => Name::missing_item_name(),
            };
            let ground_item = match items.get(ground_entity) {
                Some(item) => item,
                None => {
                    eprintln!(
                        "{:?} was not an item, it's name was {}",
                        ground_entity, item_name
                    );
                    continue;
                }
            };

            let edb = &ENTITY_DB.lock().unwrap();
            // TODO: check inventory capacity
            match (&entities, &items, &inbags)
                .join()
                .find(|(_, item, bag)| bag.owner == picker && item.id == ground_item.id)
            {
                Some((bagged_entity, bagged_item, _)) => {
                    let _ = items.insert(
                        bagged_entity,
                        Item::new(bagged_item.id, bagged_item.qty + ground_item.qty),
                    );
                    let _ = entities.delete(ground_entity);
                }
                None => {
                    let _ = inbags.insert(ground_entity, InBag { owner: picker });
                    positions.remove(ground_entity);
                    if let Some(text) = &edb.items.get_by_name_unchecked(&item_name.0).pickup_text {
                        log.enhance(text);
                    }
                }
            }
            log.log(format!(
                "{} picked up a {}",
                picker_name,
                item_name.0.to_lowercase()
            ));
        }

        pickup_actions.clear();
    }
}

pub struct ZeroQtyItemCleanup;

impl<'a> System<'a> for ZeroQtyItemCleanup {
    type SystemData = (ReadStorage<'a, Item>, Entities<'a>);

    fn run(&mut self, (items, entities): Self::SystemData) {
        for (item_entity, Item { qty, .. }) in (&entities, &items).join() {
            if qty.0 == 0 {
                let _ = entities.delete(item_entity);
            }
        }
    }
}

/// Checks to see if there is atleast one `target` item on the `owner`
pub fn inventory_contains(target: &Name, owner: &Entity, ecs: &World) -> bool {
    // TODO: make this callable from systems
    // pt2: check for id instead of name
    let items = ecs.read_storage::<Item>();
    let names = ecs.read_storage::<Name>();
    let in_bags = ecs.read_storage::<InBag>();

    (&items, &names, &in_bags)
        .join()
        .filter(|(_, name, bag)| name.eq(&target) && bag.owner.eq(owner))
        .count()
        >= 1
}

pub struct ItemInfo {
    /// Unique id to find the item's static data
    pub identifier: ItemID,
    pub name: String,
    pub examine_text: String,
    pub atlas_index: u8,
    pub fg: (u8, u8, u8),
    pub pickup_text: Option<String>,
    pub equipable: Option<Equipable>,
    pub attack_bonus: Option<AttackBonus>,
}

#[derive(
    Serialize, Deserialize, Copy, Clone, Debug, Hash, Eq, PartialEq, Default, PartialOrd, Ord,
)]
pub struct ItemID(pub u32);

impl Display for ItemID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let edb = &ENTITY_DB.lock().unwrap();
        let missing = format!("Missing name {}", self.0);
        let name = match edb.items.get_by_id(*self) {
            Some(info) => &info.name,
            None => &missing,
        };
        write!(f, "id: {} | name: {}", self.0, name)
    }
}

// TODO: duplicated in items.rs, this one could probably be removed in favor the resource, ItemSpawner
pub fn build_item(
    name: &str,
    pos: Option<Position>,
    world: &mut World,
) -> Result<Entity, EntityBuildError> {
    let edb = &ENTITY_DB.lock().unwrap();
    let raw = match edb.items.get_by_name(name) {
        Some(raw) => raw,
        None => {
            eprintln!("No world object found named: {}", name.to_string());
            return Err(EntityBuildError);
        }
    };
    let mut builder = world
        .create_entity()
        .with(Item::new(raw.identifier, ItemQty(1)))
        .with(Name::new(&raw.name))
        .with(Renderable::default_bg(raw.atlas_index, raw.fg, ITEM_Z));

    if let Some(pos) = pos {
        builder = builder.with(pos);
    }

    if let Some(equipable) = &raw.equipable {
        builder = builder.with(equipable.clone());
    }
    if let Some(attack) = &raw.attack_bonus {
        builder = builder.with(attack.clone());
    }

    Ok(builder.build())
}
