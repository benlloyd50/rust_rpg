use serde::Deserialize;
use specs::{Builder, Entity, World, WorldExt};

use crate::{
    components::{Blocking, GoalMoverAI, Monster, Name, Position, RandomWalkerAI, Renderable},
    stats::EntityStatsBuilder,
    z_order::BEING_Z,
};

use super::{EntityBuildError, OptionalStats, ENTITY_DB};

#[derive(Deserialize)]
pub struct BeingDatabase {
    data: Vec<Being>,
}

#[derive(Deserialize)]
pub struct Being {
    pub(crate) identifier: BeingID,
    pub(crate) name: String,
    pub(crate) monster: Option<String>,
    pub(crate) ai: Option<AIDefinition>,
    pub(crate) is_blocking: bool,
    pub(crate) atlas_index: u8,
    pub(crate) fg: (u8, u8, u8),
    pub(crate) quips: Option<Vec<String>>,
    pub(crate) stats: Option<OptionalStats>,
}

#[derive(Deserialize)]
pub struct AIDefinition {
    pub(crate) start_mode: String,
    pub(crate) goals: Option<Vec<String>>,
    pub(crate) goal_range: Option<usize>,
}

#[derive(Deserialize, Debug)]
pub struct BeingID(pub u32);

impl BeingDatabase {
    pub(crate) fn empty() -> Self {
        Self { data: Vec::new() }
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
pub fn build_being(
    name: impl ToString,
    pos: Position,
    world: &mut World,
) -> Result<Entity, EntityBuildError> {
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
        .with(pos)
        .with(Renderable::default_bg(raw.atlas_index, raw.fg, BEING_Z));

    if raw.monster.is_some() {
        builder = builder.with(Monster);
    }

    if raw.is_blocking {
        builder = builder.with(Blocking);
    }

    if let Some(ai_def) = &raw.ai {
        builder = match ai_def.start_mode.as_str() {
            "random_walk" => builder.with(RandomWalkerAI),
            "goal" => {
                let goals = match &ai_def.goals {
                    Some(goals) => goals
                        .iter()
                        .map(|goal| Name(goal.to_string()))
                        .collect::<Vec<Name>>(),
                    None => panic!("{} has Goal ai type but no defined goals", &raw.name),
                };
                builder.with(GoalMoverAI::with_desires(&goals, ai_def.goal_range.unwrap()))
            }
            _ => builder,
        };
    }

    if let Some(stats) = &raw.stats {
        let mut esb = EntityStatsBuilder::new();

        if let Some(intelligence) = stats.intelligence {
            esb.with_intelligence(intelligence);
        }
        if let Some(strength) = stats.strength {
            esb.with_strength(strength);
        }
        if let Some(dexterity) = stats.dexterity {
            esb.with_dexterity(dexterity);
        }
        if let Some(vitality) = stats.vitality {
            esb.with_vitality(vitality);
        }
        if let Some(charisma) = stats.charisma {
            esb.with_charisma(charisma);
        }
        if let Some(precision) = stats.precision {
            esb.with_precision(precision);
        }

        builder = builder.with(esb.build()).with(esb.build_health_stats());
    }

    Ok(builder.build())
}
