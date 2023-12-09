use crate::{
    crafting::{Ingredient, UseWithRecipe},
    items::ItemQty,
};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use serde_json::from_str;
use std::{fs, sync::Mutex};

use super::ENTITY_DB;

lazy_static! {
    pub static ref RECIPE_DB: Mutex<RecipeDatabase> = Mutex::new(RecipeDatabase::new());
}

pub struct RecipeDatabase {
    pub use_with_recipes: Vec<UseWithRecipe>,
}

impl RecipeDatabase {
    pub fn new() -> Self {
        Self { use_with_recipes: Vec::new() }
    }

    pub fn load(&mut self) {
        let contents: String =
            fs::read_to_string("raws/recipes.json").expect("Unable to find recipes.json at `raws/recipes.json`");
        let recipes: Vec<RawRecipe> = from_str(&contents).expect("Bad JSON in recipes.json fix it");
        let edb = &ENTITY_DB.lock().unwrap();
        self.use_with_recipes = recipes
            .iter()
            .map(|r| UseWithRecipe {
                ingredients: vec![
                    Ingredient {
                        id: edb.items.get_by_name_unchecked(&r.first.name).identifier,
                        consume: r.first.consume.map(ItemQty),
                    },
                    Ingredient {
                        id: edb.items.get_by_name_unchecked(&r.second.name).identifier,
                        consume: r.second.consume.map(ItemQty),
                    },
                ],
                output: edb.items.get_by_name_unchecked(&r.output).identifier,
            })
            .collect();
    }
}

#[derive(Deserialize, Serialize)]
struct RawRecipe {
    first: RawIngredient,
    second: RawIngredient,
    output: String,
}

#[derive(Deserialize, Serialize)]
struct RawIngredient {
    name: String,
    consume: Option<usize>,
}
