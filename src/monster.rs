use std::time::Duration;

use bracket_random::prelude::RandomNumberGenerator;
use specs::{Join, ReadStorage, System, World, WorldExt, WriteExpect, WriteStorage};

use crate::{
    components::{Monster, Name, Position, RandomWalkerAI, GoalMoverAI},
    data_read::ENTITY_DB,
    message_log::MessageLog,
    time::DeltaTime,
};

/// Mainly used for early testing but it's somewhat useful
/// Random Cardinal Directional Movement or RCDM for short
pub struct RandomMonsterMovementSystem;

impl<'a> System<'a> for RandomMonsterMovementSystem {
    type SystemData = (
        WriteStorage<'a, Position>,
        ReadStorage<'a, Monster>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, RandomWalkerAI>,
        WriteExpect<'a, MessageLog>,
    );

    fn run(&mut self, (mut positions, mons, names, randwalks, mut log): Self::SystemData) {
        let mut rng = RandomNumberGenerator::new();
        for (pos, _, name, _) in (&mut positions, &mons, &names, &randwalks).join() {
            match rng.range(0, 100) {
                0..=10 => {
                    pos.x += 1;
                }
                11..=20 => {
                    pos.y += 1;
                }
                21..=30 => {
                    pos.y = pos.y.saturating_sub(1);
                }
                31..=40 => {
                    pos.x = pos.x.saturating_sub(1);
                }
                79 => {
                    let edb = &ENTITY_DB.lock().unwrap();
                    if let Some(monster) = edb.beings.get_by_name(&name.0) {
                        monster
                            .quips
                            .as_ref()
                            .and_then(|quips| quips.first())
                            .map(|quip| log.enhance(quip));
                    }
                }
                _ => {}
            }
        }
    }
}

const MONSTER_ACTION_DELAY: Duration = Duration::from_secs(1);

/// Delays all monster entities from moving while player is activity bound
pub fn check_monster_delay(ecs: &World, monster_delay: &mut Duration) -> bool {
    let delta_time = ecs.read_resource::<DeltaTime>();
    *monster_delay = monster_delay.checked_add(delta_time.0).unwrap();

    if *monster_delay >= MONSTER_ACTION_DELAY {
        true
    } else {
        false
    }
}

pub struct UpdateGoalEntities;

impl<'a> System<'a> for UpdateGoalEntities {
    type SystemData = (WriteStorage<'a, GoalMoverAI>,
ReadStorage<'a, Position>,
ReadStorage<'a, Name>,
    );

    fn run(&mut self, (goal_movers, positions, names): Self::SystemData) {
        
    }
}