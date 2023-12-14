use log::warn;
use specs::{Entities, Entity, Join, ReadStorage, System, Write, WriteStorage};

use crate::{
    components::{CraftAction, InBag, Item},
    data_read::prelude::RECIPE_DB,
    items::{ItemID, ItemQty, ItemSpawner, SpawnType},
    ui::message_log::MessageLog,
};

pub struct UseWithRecipe {
    pub ingredients: Vec<Ingredient>,
    pub output: ItemID,
}

pub struct Ingredient {
    pub id: ItemID,
    pub consume: Option<ItemQty>,
}

pub struct HandleCraftingSystem;

impl<'a> System<'a> for HandleCraftingSystem {
    type SystemData = (
        WriteStorage<'a, CraftAction>,
        Write<'a, ItemSpawner>,
        Write<'a, MessageLog>,
        WriteStorage<'a, Item>,
        ReadStorage<'a, InBag>,
        Entities<'a>,
    );

    /// TODO: check for item qty in recipes
    fn run(
        &mut self,
        (mut craft_actions, mut spawn_requests, mut log, mut items, in_bags, entities): Self::SystemData,
    ) {
        let rdb = &RECIPE_DB.lock().unwrap();
        'outer: for (crafter, craft_action) in (&entities, &craft_actions).join() {
            let crafting_items: Vec<(Entity, &Item)> = (&entities, &items, &in_bags)
                .join()
                .filter(|(bagged_entity, _, bag)| {
                    bag.owner == crafter && (*bagged_entity == craft_action.first_item)
                        || (*bagged_entity == craft_action.second_item)
                })
                .map(|(item_entity, item, _)| (item_entity, item))
                .collect();

            #[rustfmt::skip] // not the prettiest way to check
            let recipe_crafted = match rdb.use_with_recipes.iter().find(|recipe| {
                crafting_items.iter().any(|(_, Item { id, ..})| recipe.ingredients[0].id == *id)
                && crafting_items.iter().any(|(_, Item { id, ..})| recipe.ingredients[1].id == *id)
            }) {
                Some(recipe) => recipe,
                None => {
                    log.log("No recipe matched for the items used to craft");
                    continue;
                }
            };

            let mut item_updates: Vec<(Entity, Item)> = vec![];
            // check there are enough of a consumable ingredient
            for ingredient in recipe_crafted.ingredients.iter().filter(|ingredient| ingredient.consume.is_some()) {
                match crafting_items.iter().find(|(_, Item { id, .. })| id.eq(&ingredient.id)) {
                    Some((e, bag_item)) => {
                        if bag_item.qty < ingredient.consume.unwrap() {
                            continue 'outer;
                        }
                        item_updates.push((*e, Item::new(bag_item.id, bag_item.qty - ingredient.consume.unwrap())));
                    }
                    None => {
                        warn!("Item entity was cleared before proper cleanup was conducted.")
                    }
                }
            }

            for (entity, new_item) in item_updates {
                let _ = items.insert(entity, new_item);
            }

            spawn_requests.request(recipe_crafted.output, SpawnType::InBag(crafter));
        }

        craft_actions.clear();
    }
}
