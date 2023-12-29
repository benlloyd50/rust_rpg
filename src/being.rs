use bracket_lib::random::RandomNumberGenerator;
use bracket_lib::terminal::Point;
use log::{info, warn};
use pathfinding::prelude::astar;
use serde::{Deserialize, Serialize};
#[allow(deprecated)] // specs's saveload requires this import so it's not our choice
use specs::error::NoError;
use specs::{
    saveload::{ConvertSaveload, Marker},
    shred::PanicHandler,
    Component, ConvertSaveload, Entities, Entity, Join, ReadExpect, ReadStorage, System, VecStorage, Write,
    WriteExpect, WriteStorage,
};

use crate::{
    components::{AttackAction, BreakAction, GoalMoverAI, MoveAction, Name, Position, RandomWalkerAI},
    data_read::ENTITY_DB,
    droptables::Drops,
    map::{distance, is_goal, successors, Map, TileEntity},
    stats::Stats,
    ui::message_log::MessageLog,
};

pub struct Being {
    pub(crate) identifier: BeingID,
    pub(crate) name: String,
    pub(crate) ai: Option<AIDefinition>,
    pub(crate) is_blocking: bool,
    pub(crate) atlas_index: u8,
    pub(crate) fg: (u8, u8, u8),
    pub(crate) quips: Option<Vec<String>>,
    pub(crate) stats: Stats,
    pub(crate) loot: Option<Drops>,
}

#[derive(Deserialize, Clone)]
pub struct AIDefinition {
    pub(crate) start_mode: String,
    pub(crate) goals: Option<Vec<String>>,
    pub(crate) goal_range: Option<usize>,
}

#[derive(Debug, Copy, Clone, Deserialize, ConvertSaveload, Component)]
#[storage(VecStorage)]
pub struct BeingID(pub u32);

/// Mainly used for early testing but it's somewhat useful
/// Random Cardinal Directional Movement or RCDM for short
pub struct RandomMonsterMovementSystem;

impl<'a> System<'a> for RandomMonsterMovementSystem {
    type SystemData = (
        WriteStorage<'a, Position>,
        WriteStorage<'a, BreakAction>,
        WriteStorage<'a, MoveAction>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, RandomWalkerAI>,
        WriteExpect<'a, MessageLog>,
        ReadExpect<'a, Map>,
        Entities<'a>,
    );

    fn run(
        &mut self,
        (
            mut positions,
            mut break_actions,
            mut move_actions,
            names,
            randwalks,
            mut log,
            map,
            entities,
        ): Self::SystemData,
    ) {
        let mut rng = RandomNumberGenerator::new();
        for (entity, pos, name, _) in (&entities, &mut positions, &names, &randwalks).join() {
            let delta: Point = match rng.range::<u32>(0, 100) {
                0..=10 => Point::new(1, 0),
                11..=20 => Point::new(0, 1),
                21..=30 => Point::new(0, -1),
                31..=40 => Point::new(-1, 0),
                41..=97 => {
                    continue;
                }
                98..=99 => {
                    say_random_quip(&name, &mut log);
                    continue;
                }
                _ => unreachable!("rng.range(0, 100) should have range of "),
            };

            let target_pos = Point::new(pos.x as i32 + delta.x, pos.y as i32 + delta.y);
            if !map.in_bounds(target_pos) {
                return;
            }

            if let Some(tile) = map.first_entity_in_pos(&Position::from(target_pos)) {
                match tile {
                    TileEntity::Item(_) => {}
                    TileEntity::Breakable(target) => {
                        let _ = break_actions.insert(entity, BreakAction { target: *target });
                    }
                    TileEntity::Blocking(_blocker) => {
                        // maybe try attacking
                        continue;
                    }
                    _ => return,
                }
            }

            let _ = move_actions.insert(entity, MoveAction::new(target_pos.into()));
        }
    }
}

fn say_random_quip(name: &Name, log: &mut Write<MessageLog, PanicHandler>) {
    let edb = &ENTITY_DB.lock().unwrap();
    if let Some(monster) = edb.beings.get_by_name(&name.0) {
        if let Some(quip) = monster.quips.as_ref().and_then(|quips| quips.first()) {
            log.enhance(quip)
        }
    }
}

pub struct GoalFindEntities;

impl<'a> System<'a> for GoalFindEntities {
    type SystemData = (
        WriteStorage<'a, GoalMoverAI>,
        WriteStorage<'a, RandomWalkerAI>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, Name>,
        Entities<'a>,
    );

    fn run(&mut self, (mut goal_movers, mut randwalkers, positions, names, entities): Self::SystemData) {
        let mut remove_mes: Vec<Entity> = vec![];
        for (goal_entity, goal_mover, mover_pos, mover_name) in (&entities, &mut goal_movers, &positions, &names).join()
        {
            if goal_mover.current.is_some() {
                continue;
            }
            let mut closest_goal = (None, 1000000);
            let data: Vec<_> = (&entities, &names, &positions)
                .join()
                .filter(|(e, n, _)| goal_mover.desires.contains(n) && e.ne(&goal_entity))
                .collect();
            if data.len() == 0 {
                info!("No goals remain for {}, switching to randomwalk", mover_name);
                let _ = randwalkers.insert(goal_entity, RandomWalkerAI {});
                remove_mes.push(goal_entity);
            }

            for (entity, _, pos) in data {
                let dist_from_goal = distance(mover_pos, pos);
                let goal_within_range = (dist_from_goal as usize) < goal_mover.goal_range || goal_mover.goal_range == 0;
                if goal_within_range && dist_from_goal < closest_goal.1 {
                    closest_goal = (Some(entity), dist_from_goal);
                }
            }

            goal_mover.current = closest_goal.0;
        }
        for me in remove_mes.iter() {
            goal_movers.remove(*me);
        }
    }
}

pub struct GoalMoveToEntities;

impl<'a> System<'a> for GoalMoveToEntities {
    type SystemData = (
        WriteStorage<'a, MoveAction>,
        WriteStorage<'a, AttackAction>,
        WriteStorage<'a, GoalMoverAI>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, Name>,
        ReadExpect<'a, Map>,
        Entities<'a>,
    );

    fn run(
        &mut self,
        (mut move_actions, mut attack_actions, mut goal_movers, positions, names, map, entities): Self::SystemData,
    ) {
        for (entity, goal_mover, mover_pos, mover_name) in (&entities, &mut goal_movers, &positions, &names).join() {
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
                let _ = attack_actions
                    .insert(entity, AttackAction { target: goal_mover.current.unwrap() /* cant be none */ });
                let missing_name = Name::new("Missing");
                let target_name = names.get(goal_mover.current.unwrap()).unwrap_or(&missing_name);
                info!("{} tries to attack {}", mover_name, target_name);
                continue;
            }
            let path: (Vec<Position>, u32) =
                match astar(mover_pos, |p| successors(&map, p), |p| distance(p, goal_pos), |p| is_goal(p, goal_pos)) {
                    Some(path) => path,
                    None => {
                        continue;
                    }
                };
            if path.0.len() > 1 {
                let new_position = path.0[1];
                let _ = move_actions.insert(entity, MoveAction::new(new_position));
            }
        }
    }
}

// Performs the changes of a move for an entity, this should be called only if the move is valid.
pub struct HandleMoveActions;

impl<'a> System<'a> for HandleMoveActions {
    type SystemData = (WriteStorage<'a, MoveAction>, WriteStorage<'a, Position>, WriteExpect<'a, Map>, Entities<'a>);

    fn run(&mut self, (mut move_actions, mut positions, mut map, entities): Self::SystemData) {
        for (entity, want, mover_pos) in (&entities, &move_actions, &mut positions).join() {
            let idx = mover_pos.to_idx(map.width);
            match map.tile_entities[idx].iter().position(|tile| tile == &TileEntity::Blocking(entity)) {
                Some(remove_idx) => {
                    map.tile_entities[idx].remove(remove_idx);
                }
                None => {
                    // if ever a monster exists that doesn't block their own position, then change
                    // to an info maybe
                    warn!(
                        "Move action user was not blocking their previous position {}. Continuing move anyways.",
                        mover_pos
                    );
                }
            }

            *mover_pos = want.new_pos;

            let idx = mover_pos.to_idx(map.width);
            map.tile_entities[idx].push(TileEntity::Blocking(entity));
        }

        move_actions.clear();
    }
}
