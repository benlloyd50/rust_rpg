use serde::Deserialize;
use specs::{Builder, Entity, World, WorldExt};

use crate::{
    components::{Blocking, GoalMoverAI, Monster, Name, Position, RandomWalkerAI, Renderable},
    z_order::BEING_Z,
};

use super::{EntityBuildError, ENTITY_DB};

#[derive(Deserialize)]
pub struct BeingDatabase {
    data: Vec<Being>,
}

#[derive(Deserialize, Debug)]
pub struct Being {
    pub(crate) identifier: BeingID,
    pub(crate) name: String,
    pub(crate) monster: Option<String>,
    pub(crate) is_blocking: bool,
    pub(crate) ai: Option<String>,
    pub(crate) atlas_index: usize,
    pub(crate) fg: (u8, u8, u8),
    pub(crate) quips: Option<Vec<String>>,
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

    if let Some(_) = &raw.monster {
        builder = builder.with(Monster);
    }

    if raw.is_blocking {
        builder = builder.with(Blocking);
    }

    if let Some(ai_type) = &raw.ai {
        builder = match ai_type.as_str() {
            "random_walk" => builder.with(RandomWalkerAI),
            "goal" => builder.with(GoalMoverAI::new()),
            _ => builder,
        };
    }

    Ok(builder.build())
}
