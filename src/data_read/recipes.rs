use crate::data_read::items::ItemID;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use serde_json::from_str;
use std::{fs, sync::Mutex};

use super::ENTITY_DB;

lazy_static! {
    pub static ref RECIPE_DB: Mutex<RecipeDatabase> = Mutex::new(RecipeDatabase::new());
}

pub struct RecipeDatabase {
    pub recipes: Vec<Recipe>,
}

impl RecipeDatabase {
    pub fn new() -> Self {
        Self {
            recipes: Vec::new(),
        }
    }

    pub fn load(&mut self) {
        let contents: String =
            fs::read_to_string("raws/recipes.json").expect("Unable to find recipes.json at `raws/recipes.json`");
        let recipes: Vec<RawRecipe> = from_str(&contents).expect("Bad JSON in recipes.json fix it");
        let edb = &ENTITY_DB.lock().unwrap();
        self.recipes = recipes
            .iter()
            .map(|r| Recipe {
                first: edb.items.get_by_name_unchecked(&r.first).identifier,
                second: edb.items.get_by_name_unchecked(&r.second).identifier,
                output: edb.items.get_by_name_unchecked(&r.output).identifier,
                use_items: r.use_items,
            })
            .collect();
    }
}

pub struct Recipe {
    pub first: ItemID,
    pub second: ItemID,
    pub output: ItemID,
    pub use_items: bool,
}

#[derive(Deserialize, Serialize)]
pub struct RawRecipe {
    pub first: String,
    pub second: String,
    pub output: String,
    pub use_items: bool,
}
