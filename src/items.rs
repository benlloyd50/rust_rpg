/* Item/Inventory Workflows
 * Items go either on the floor or into an inventory
 * Destructible Rock - when broke -> Item on Floor - when picked up -> Item in Inventory 
 * Enemy - when killed -> Item on Florr - ...
 * Quest - when finished -> Item in Inventory
 */

use bracket_terminal::prelude::{ColorPair, WHITE, BLACK};
use specs::{System, WriteStorage, Write, Entities};

use crate::{components::{Renderable, Item, Position, Name}, data_read::{ItemID, ENTITY_DB}};


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

    pub fn request(&mut self, ItemID(item_id): ItemID, x: usize, y: usize) {
        self.requests.push(ItemSpawnRequest { item_id , position: Position::new(x, y) });
    }
}

pub struct ItemSpawnRequest {
    item_id: u32,
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

    fn run(&mut self, (entities, mut spawn_requests, mut items, mut positions, mut renderables, mut names): Self::SystemData) {
        let edb = &ENTITY_DB.lock().unwrap();

        for spawn in spawn_requests.requests.iter() {
            let static_item = match edb.items.get(spawn.item_id) {
                Some(val) => val,
                None => {
                    eprintln!("Spawn request failed because {} item id does not exist in database", spawn.item_id);
                    continue
                }
            };

            let new_item = entities.create();
            let _ = positions.insert(new_item, spawn.position);
            let _ = renderables.insert(new_item, Renderable { color_pair: ColorPair::new(WHITE, BLACK), atlas_index: static_item.atlas_index });
            let _ = items.insert(new_item, Item);
            let _ = names.insert(new_item, Name(static_item.name.clone()));
        }

        spawn_requests.requests.clear();
    }
}
