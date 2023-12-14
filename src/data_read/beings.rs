use std::fs;

use serde::Deserialize;
use serde_json::from_str;
use specs::{
    saveload::{MarkedBuilder, SimpleMarker},
    Builder, Entity, World, WorldExt,
};

use crate::{
    being::{AIDefinition, Being, BeingID},
    components::{Blocking, GoalMoverAI, Name, Position, RandomWalkerAI, Renderable},
    droptables::Drops,
    saveload::SerializeMe,
    stats::{EntityStatsBuilder, Stats},
    z_order::BEING_Z,
};

use super::{EntityBuildError, GameData, OptionalStats, ENTITY_DB};

pub struct BeingDatabase {
    data: Vec<Being>,
}

#[derive(Deserialize)]
pub struct RawBeing {
    pub(crate) identifier: BeingID,
    pub(crate) name: String,
    pub(crate) ai: Option<AIDefinition>,
    pub(crate) is_blocking: bool,
    pub(crate) atlas_index: u8,
    pub(crate) fg: (u8, u8, u8),
    pub(crate) quips: Option<Vec<String>>,
    pub(crate) stats: Option<OptionalStats>,
    pub(crate) loot: Option<RawDrops>,
}

#[derive(Deserialize)]
pub struct RawDrops {
    pub(crate) drop_chance: u32, // 1 - 100 indicates the chance there is a drop
    pub(crate) loot_table: Vec<RawLoot>,
}

#[derive(Deserialize)]
pub struct RawLoot {
    pub(crate) item: String,
    pub(crate) item_qty: String,
    pub(crate) weight: u32,
}

impl BeingDatabase {
    pub(crate) fn empty() -> Self {
        Self { data: Vec::new() }
    }

    // Uses GameData in order to transform string names into item ids
    pub fn load(game_db: &GameData) -> Self {
        let contents: String =
            fs::read_to_string("raws/beings.json").expect("Unable to find beings.json at `raws/beings.json`");
        let beings: Vec<RawBeing> = from_str(&contents).expect("Bad JSON in beings.json fix it");
        BeingDatabase {
            data: beings
                .iter()
                .map(|raw| Being {
                    identifier: raw.identifier,
                    name: raw.name.clone(),
                    ai: raw.ai.clone(),
                    is_blocking: raw.is_blocking,
                    atlas_index: raw.atlas_index,
                    fg: raw.fg,
                    quips: raw.quips.to_owned(),
                    stats: raw.stats.as_ref().map_or_else(Stats::zero, |stats| Stats::from_optional(&stats)),
                    loot: raw.loot.as_ref().and_then(|raw| Some(Drops::from_raw(raw, game_db))),
                })
                .collect(),
        }
    }

    pub fn get_by_name(&self, name: &String) -> Option<&Being> {
        self.data.iter().find(|i| i.name.eq(name))
    }

    #[allow(dead_code)]
    pub fn get_by_id(&self, id: u32) -> Option<&Being> {
        self.data.iter().find(|i| i.identifier.0 == id)
    }
}

/// Attempts to create the specified entity directly into the world
pub fn build_being(name: impl ToString, pos: Position, world: &mut World) -> Result<Entity, EntityBuildError> {
    let edb = &ENTITY_DB.lock().unwrap();

    let raw = match edb.beings.get_by_name(&name.to_string()) {
        Some(raw) => raw,
        None => {
            eprintln!("No being found named: {}", name.to_string());
            return Err(EntityBuildError);
        }
    };

    let mut builder = world
        .create_entity()
        .with(Name::new(&raw.name))
        .with(raw.identifier)
        .with(pos)
        .with(Renderable::clear_bg(raw.atlas_index, raw.fg, BEING_Z))
        .marked::<SimpleMarker<SerializeMe>>();

    if raw.is_blocking {
        builder = builder.with(Blocking {});
    }

    if let Some(ai) = &raw.ai {
        builder = match ai.start_mode.as_str() {
            "random_walk" => builder.with(RandomWalkerAI {}),
            "goal" => {
                let goals = match &ai.goals {
                    Some(goals) => goals.iter().map(|goal| Name(goal.to_string())).collect::<Vec<Name>>(),
                    None => panic!("{} has Goal ai type but no defined goals", &raw.name),
                };
                builder.with(GoalMoverAI::with_desires(&goals, ai.goal_range.unwrap()))
            }
            _ => builder,
        };
    }

    let esb = EntityStatsBuilder::new()
        .with_intelligence(raw.stats.intelligence)
        .with_strength(raw.stats.strength)
        .with_dexterity(raw.stats.dexterity)
        .with_vitality(raw.stats.vitality)
        .with_charisma(raw.stats.charisma)
        .with_precision(raw.stats.precision)
        .build();

    builder = builder.with(esb.0).with(esb.1);

    Ok(builder.build())
}
