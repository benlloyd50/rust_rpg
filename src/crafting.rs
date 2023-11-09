use specs::{Entities, Entity, Join, ReadStorage, System, Write, WriteStorage};

use crate::{
    components::{InBag, Item, WantsToCraft},
    data_read::prelude::RECIPE_DB,
    items::{ItemSpawner, SpawnType},
    ui::message_log::MessageLog,
};

pub struct HandleCraftingSystem;

impl<'a> System<'a> for HandleCraftingSystem {
    type SystemData = (
        WriteStorage<'a, WantsToCraft>,
        Write<'a, ItemSpawner>,
        Write<'a, MessageLog>,
        ReadStorage<'a, Item>,
        ReadStorage<'a, InBag>,
        Entities<'a>,
    );

    /// TODO: check for item qty in recipes
    fn run(
        &mut self,
        (mut wants_to_craft, mut spawn_requests, mut log, items, in_bags, entities): Self::SystemData,
    ) {
        let rdb = &RECIPE_DB.lock().unwrap();
        for (crafter, craft_action) in (&entities, &wants_to_craft).join() {
            let crafter_inv: Vec<(Entity, &Item, &InBag)> = (&entities, &items, &in_bags)
                .join()
                .filter(|(_, _, bag)| bag.owner == crafter)
                .into_iter()
                .collect();
            let recipe_crafted = match rdb.use_with_recipes.iter().find(|r| {
                r.first.id.eq(&crafter_inv[craft_action.first_idx].1 .0)
                    && r.second.id.eq(&crafter_inv[craft_action.second_idx].1 .0)
            }) {
                Some(recipe) => recipe,
                None => {
                    log.log("No recipe matched for the items used to craft");
                    return;
                }
            };

            if recipe_crafted.first.consume {
                let _ = entities.delete(crafter_inv[craft_action.first_idx].0);
            }
            if recipe_crafted.second.consume {
                let _ = entities.delete(crafter_inv[craft_action.second_idx].0);
            }

            spawn_requests.request(recipe_crafted.output, SpawnType::InBag(crafter));
        }

        wants_to_craft.clear();
    }
}
