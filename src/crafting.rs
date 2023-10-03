use crate::components::Backpack;
use crate::data_read::prelude::{ItemID, RECIPE_DB};

/// Crafts an item by removing items from a `crafter`'s inventory items.
/// TODO: check for item qty in recipes
pub fn craft_item(crafter_bag: &mut Backpack, first_item_used: ItemID, used_on: ItemID) {
    let rdb = &RECIPE_DB.lock().unwrap();
    let recipe_crafted = match rdb
        .recipes
        .iter()
        .find(|r| r.first.eq(&first_item_used) && r.second.eq(&used_on))
    {
        Some(recipe) => recipe,
        None => {
            print!("No recipe match the action performed. ||");
            println!(" First item: {} Second item: {}", first_item_used, used_on);
            return;
        }
    };

    if recipe_crafted.use_items {
        crafter_bag.remove_item(&first_item_used, 1);
        crafter_bag.remove_item(&used_on, 1);
    }

    crafter_bag.add_item(recipe_crafted.output, 1);
}
