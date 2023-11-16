use specs::{Entities, Entity, Join, ReadStorage, System, Write, WriteStorage};

use crate::{
    components::{InBag, Item, WantsToCraft},
    data_read::prelude::{ItemID, RECIPE_DB},
    items::{ItemSpawner, SpawnType},
    ui::message_log::MessageLog,
};

pub struct UseWithRecipe {
    pub ingredients: Vec<Ingredient>,
    pub output: ItemID,
}

pub struct Ingredient {
    pub id: ItemID,
    pub consume: bool,
}

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
        (mut craft_actions, mut spawn_requests, mut log, items, in_bags, entities): Self::SystemData,
    ) {
        let rdb = &RECIPE_DB.lock().unwrap();
        for (crafter, craft_action) in (&entities, &craft_actions).join() {
            let crafting_items: Vec<(Entity, &ItemID)> = (&entities, &items, &in_bags)
                .join()
                .enumerate()
                .filter (|(idx, (_, _, bag))| {
                    bag.owner == crafter
                        && (*idx == craft_action.first_idx || *idx == craft_action.second_idx)
                })
                .map(|(_, (item_entity, Item(id), _))| (item_entity, id))
                .collect();

            #[rustfmt::skip] // not the prettiest way to check
            let recipe_crafted = match rdb.use_with_recipes.iter().find(|recipe| {
                crafting_items.iter().any(|(_, id)| recipe.ingredients[0].id == **id)
                && crafting_items.iter().any(|(_, id)| recipe.ingredients[1].id == **id)
            }) {
                Some(recipe) => recipe,
                None => {
                    log.log("No recipe matched for the items used to craft");
                    continue;
                }
            };

            for ingredient in &recipe_crafted.ingredients {
                if !ingredient.consume {
                    continue;
                }
                match crafting_items.iter().find(|(_, &id)| id.eq(&ingredient.id)) {
                    Some((e, _)) => {
                        let _ = entities.delete(*e);
                    }
                    None => eprintln!("Item entity was cleared before proper cleanup was conducted."),
                }
            }

            spawn_requests.request(recipe_crafted.output, SpawnType::InBag(crafter));
        }

        craft_actions.clear();
    }
}
