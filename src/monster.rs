use std::time::Duration;

use bracket_random::prelude::RandomNumberGenerator;
use bracket_terminal::prelude::Point;
use pathfinding::prelude::astar;
use specs::{
    Entities, Join, ReadExpect, ReadStorage, System, World, WorldExt, WriteExpect, WriteStorage,
};

use crate::{
    components::{BreakAction, GoalMoverAI, Monster, Name, Position, RandomWalkerAI, WantsToMove},
    data_read::ENTITY_DB,
    map::{distance, is_goal, successors, Map, TileEntity},
    time::DeltaTime,
    ui::message_log::MessageLog,
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
        ReadExpect<'a, Map>,
    );

    fn run(&mut self, (mut positions, mons, names, randwalks, mut log, map): Self::SystemData) {
        let mut rng = RandomNumberGenerator::new();
        for (pos, _, name, _) in (&mut positions, &mons, &names, &randwalks).join() {
            match rng.range(0, 100) {
                0..=10 => {
                    try_move_monster(1, 0, &map, pos);
                }
                11..=20 => {
                    try_move_monster(0, 1, &map, pos);
                }
                21..=30 => {
                    try_move_monster(0, -1, &map, pos);
                }
                31..=40 => {
                    try_move_monster(-1, 0, &map, pos);
                }
                79 => {
                    let edb = &ENTITY_DB.lock().unwrap();
                    if let Some(monster) = edb.beings.get_by_name(&name.0) {
                        if let Some(quip) = monster.quips.as_ref().and_then(|quips| quips.first()) {
                            log.enhance(quip)
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

fn try_move_monster(delta_x: i32, delta_y: i32, map: &Map, monster_pos: &mut Position) {
    let target_pos = Point::new(
        monster_pos.x as i32 + delta_x,
        monster_pos.y as i32 + delta_y,
    );
    if target_pos.x < 0
        || target_pos.y < 0
        || target_pos.x >= map.width as i32
        || target_pos.y >= map.height as i32
    {
        return;
    }

    if let Some(tile) = map.first_entity_in_pos(&Position::from(target_pos)) {
        match tile {
            TileEntity::Item(_) => {}
            _ => return,
        }
    }

    monster_pos.x = target_pos.x as usize;
    monster_pos.y = target_pos.y as usize;
}

const MONSTER_ACTION_DELAY: Duration = Duration::from_secs(1);

/// Delays all monster entities from moving while player is activity bound
pub fn check_monster_delay(ecs: &World, monster_delay: &mut Duration) -> bool {
    let delta_time = ecs.read_resource::<DeltaTime>();
    *monster_delay = monster_delay.checked_add(delta_time.0).unwrap();

    *monster_delay >= MONSTER_ACTION_DELAY
}

pub struct GoalFindEntities;

impl<'a> System<'a> for GoalFindEntities {
    type SystemData = (
        WriteStorage<'a, GoalMoverAI>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, Name>,
        Entities<'a>,
    );

    fn run(&mut self, (mut goal_movers, positions, names, entities): Self::SystemData) {
        for (goal_entity, goal_mover, mover_pos) in (&entities, &mut goal_movers, &positions).join()
        {
            if goal_mover.current.is_some() {
                continue;
            }
            let mut closest_goal = (None, 1000000);
            for (entity, _, pos) in (&entities, &names, &positions)
                .join()
                .filter(|(e, n, _)| goal_mover.desires.contains(n) && e.ne(&goal_entity))
            {
                let dist_from_goal = distance(mover_pos, pos);
                if dist_from_goal < closest_goal.1 {
                    closest_goal = (Some(entity), dist_from_goal);
                }
            }

            goal_mover.current = closest_goal.0;
        }
    }
}

pub struct GoalMoveToEntities;

impl<'a> System<'a> for GoalMoveToEntities {
    type SystemData = (
        WriteStorage<'a, WantsToMove>,
        WriteStorage<'a, BreakAction>,
        WriteStorage<'a, GoalMoverAI>,
        ReadStorage<'a, Position>,
        ReadExpect<'a, Map>,
        Entities<'a>,
    );

    fn run(
        &mut self,
        (mut wants_to_move, mut break_actions, mut goal_movers, positions, map, entities): Self::SystemData,
    ) {
        for (entity, goal_mover, mover_pos) in (&entities, &mut goal_movers, &positions).join() {
            if goal_mover.current.is_none() {
                continue;
            }
            let goal_pos = match positions.get(goal_mover.current.unwrap()) {
                Some(pos) => pos,
                None => {
                    goal_mover.current = None;
                    continue;
                }
            };

            if distance(mover_pos, goal_pos) < 2 {
                let _ = break_actions.insert(
                    entity,
                    BreakAction {
                        target: goal_mover.current.unwrap(), /* cant be none */
                    },
                );
                continue;
            }
            let path: (Vec<Position>, u32) = match astar(
                mover_pos,
                |p| successors(&map, p),
                |p| distance(p, goal_pos),
                |p| is_goal(p, goal_pos),
            ) {
                Some(path) => path,
                None => {
                    continue;
                }
            };
            if path.0.len() > 1 {
                let new_position = path.0[1];
                let _ = wants_to_move.insert(entity, WantsToMove::new(new_position));
            }
        }
    }
}

pub struct HandleMoveActions;

impl<'a> System<'a> for HandleMoveActions {
    type SystemData = (WriteStorage<'a, WantsToMove>, WriteStorage<'a, Position>);

    fn run(&mut self, (mut wants_to_move, mut positions): Self::SystemData) {
        for (want, pos) in (&wants_to_move, &mut positions).join() {
            *pos = want.new_pos;
        }

        wants_to_move.clear();
    }
}
