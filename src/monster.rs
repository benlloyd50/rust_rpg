use std::time::Duration;

use pathfinding::prelude::astar;
use bracket_random::prelude::RandomNumberGenerator;
use specs::{Join, ReadExpect, ReadStorage, System, World, WorldExt, WriteExpect, WriteStorage};

use crate::{
    components::{GoalMoverAI, Grass, Monster, Name, Position, RandomWalkerAI},
    data_read::ENTITY_DB,
    map::{Map, successors, distance, is_goal},
    message_log::MessageLog,
    time::DeltaTime, player::Player
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
    type SystemData = (
        WriteStorage<'a, Position>,
        ReadStorage<'a, GoalMoverAI>,
        ReadStorage<'a, Grass>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, Player>,
        ReadExpect<'a, Map>,
    );

    fn run(&mut self, (mut positions, goal_movers, grasses, names, players, map): Self::SystemData) {
        let goal_pos: Position;
        {
            match (&players, &positions).join().next() {
                Some(goal) => {
                    goal_pos = *goal.1;
                }
                None => {
                    return;
                }
            };
        }
        let goal_idx = map.xy_to_idx(goal_pos.x, goal_pos.y);
        println!("Frame Start <=======");
        println!("Found Goal at Idx: {}", goal_idx);

        for (_, mover_pos, name) in (&goal_movers, &mut positions, &names).join() {
            if distance(&mover_pos, &goal_pos) < 2 {
                println!("{} did not move since it was close to it's goal", name);
                continue;
            }
            let path: (Vec<Position>, u32) = match astar(mover_pos, |p| successors(&map, p), |p| distance(p, &goal_pos), |p| is_goal(p, &goal_pos)) {
                Some(path) => path,
                None => {
                    continue;
                }
            };
            println!("{} | {:?}", name, path);
            if path.0.len() > 1 {
                let new_position = path.0[1];
                *mover_pos = Position::from(new_position);
                println!("{} moved to {}", name, mover_pos);
            }
        }
        println!("Frame End =========>");
    }
}
