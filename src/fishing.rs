use std::time::Duration;

use crate::{
    components::{
        DeleteCondition, FinishedActivity, FishAction, FishOnTheLine, Fishable, Name, Renderable,
        WaitingForFish, Water,
    },
    data_read::prelude::ItemID,
    items::{ItemSpawner, SpawnType},
    tile_animation::TileAnimationBuilder,
    time::DeltaTime,
    ui::message_log::MessageLog,
    z_order::EFFECT_Z,
};

pub const WHITE: (u8, u8, u8) = (255, 255, 255);

use bracket_random::prelude::*;
use bracket_terminal::prelude::BLACK;
use specs::{Entities, Join, Read, ReadStorage, System, Write, WriteExpect, WriteStorage};

pub struct SetupFishingActions;

impl<'a> System<'a> for SetupFishingActions {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, FishAction>,
        WriteStorage<'a, WaitingForFish>,
        Write<'a, TileAnimationBuilder>,
    );

    fn run(
        &mut self,
        (entities, mut fish_actions, mut fish_waiters, mut anim_builder): Self::SystemData,
    ) {
        for (fisher, fish_action) in (&entities, &mut fish_actions).join() {
            let mut rng = RandomNumberGenerator::new();
            anim_builder.request(
                112,
                fish_action.target.x,
                fish_action.target.y,
                WHITE.into(),
                BLACK.into(),
                DeleteCondition::ActivityFinish(fisher),
            );

            let attempts = rng.range(2, 6); // this could be affected by a fishing skill level?
            match fish_waiters.insert(fisher, WaitingForFish::new(attempts)) {
                Ok(fishy) => {
                    if fishy.is_some() {
                        eprintln!("ERROR: entity: {} was already waiting for fish, they should not have performed the action again", fisher.id());
                    }
                }
                Err(..) => {
                    eprintln!("ERROR: Failed to start fishing for entity: {}", fisher.id());
                    continue;
                }
            }
        }

        fish_actions.clear(); // All fish actions in the current frame should be dealt with, so long gay browsa
    }
}

pub struct WaitingForFishSystem;
const FISH_DELAY_TIME: Duration = Duration::new(1, 0);

impl<'a> System<'a> for WaitingForFishSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, WaitingForFish>,
        WriteStorage<'a, FishOnTheLine>,
        WriteStorage<'a, FinishedActivity>,
        Read<'a, DeltaTime>,
        WriteExpect<'a, MessageLog>,
        ReadStorage<'a, Name>,
    );

    fn run(
        &mut self,
        (entities, mut waiters, mut fishing_lines, mut finished_activities, dt, mut log, names): Self::SystemData,
    ) {
        let mut rng = RandomNumberGenerator::new();
        let mut finished_fishers = Vec::new();

        for (e, waiter, name) in (&entities, &mut waiters, &names).join() {
            if waiter.attempts == 0 {
                finished_fishers.push(e);
                log.enhance(format!("{} ran out of attempts to catch a fish", name));
                continue;
            }

            // Wait for FISH_DELAY_TIME before attempting to fish again
            waiter.time_since_last_attempt += dt.0;
            if waiter.time_since_last_attempt <= FISH_DELAY_TIME {
                continue;
            }
            waiter.time_since_last_attempt = Duration::ZERO;
            waiter.attempts -= 1;

            let roll = rng.range(1, 100);
            log.debug(format!(
                "Attempts left: {} | Rolled: {} ",
                waiter.attempts, roll
            ));

            if roll < 80 {
                continue;
            }

            finished_fishers.push(e);
            log.log(format!(
                "{} caught a fish wow with {} attempts remaining",
                name, waiter.attempts
            ));

            // To prevent a fisher who is already catching from potentially catching again without waiting properly
            if fishing_lines.contains(e) {
                log.debug(format!("ERROR: entity {} {} already had a fish on their line, cannot add a second fish ABORTING fish", name, e.id()));
                fishing_lines.remove(e);
                continue;
            }
            match fishing_lines.insert(e, FishOnTheLine) {
                Ok(_) => {}
                Err(err) => {
                    log.debug(format!(
                        "ERROR: entity: {} {} failed to add fish on the line: {}",
                        name,
                        e.id(),
                        err
                    ));
                }
            }
        }

        for finished in finished_fishers.iter() {
            waiters.remove(*finished);
            let _ = finished_activities.insert(*finished, FinishedActivity);
        }
    }
}

pub struct CatchFishSystem;

impl<'a> System<'a> for CatchFishSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, FishOnTheLine>,
        WriteStorage<'a, FinishedActivity>,
        WriteExpect<'a, ItemSpawner>,
        WriteExpect<'a, MessageLog>,
        ReadStorage<'a, Name>,
    );

    fn run(
        &mut self,
        (entities, mut hooks, mut finished_activities, mut item_spawner, mut log, names): Self::SystemData,
    ) {
        let mut remove_mes = Vec::new();
        for (e, _, name) in (&entities, &hooks, &names).join() {
            remove_mes.push((e, name));
            let _ = finished_activities.insert(e, FinishedActivity);
            log.enhance(format!("{} caught a really big fish!", name));
            item_spawner.request(ItemID(3), SpawnType::InBag(e));
        }
        for (entity, _) in remove_mes.iter() {
            hooks.remove(*entity);
        }
    }
}

pub struct UpdateFishingTiles;

pub const BUBBLE_SPAWN_RATE: usize = 9000;
pub const BUBBLE_LIFETIME_SECS: u64 = 10;
impl<'a> System<'a> for UpdateFishingTiles {
    type SystemData = (
        WriteStorage<'a, Fishable>,
        WriteStorage<'a, Renderable>,
        ReadStorage<'a, Water>,
        Entities<'a>,
    );

    fn run(&mut self, (mut fishables, mut renderables, waters, entities): Self::SystemData) {
        let mut rng = RandomNumberGenerator::new();
        let mut new_bubbles = Vec::new();
        for (_, _, entity) in (!(&fishables), &waters, &entities).join() {
            if rng.range(0, BUBBLE_SPAWN_RATE) < 3 {
                println!("Oh a new bubble");
                new_bubbles.push(entity);
            }
        }
        for bubble in new_bubbles {
            let _ = fishables.insert(
                bubble,
                Fishable {
                    time_left: Duration::from_secs(BUBBLE_LIFETIME_SECS),
                },
            );
            let _ = renderables.insert(bubble, Renderable::default_bg(47, WHITE, EFFECT_Z));
        }
    }
}

pub struct PollFishingTiles;

impl<'a> System<'a> for PollFishingTiles {
    type SystemData = (
        WriteStorage<'a, Fishable>,
        WriteStorage<'a, Renderable>,
        Read<'a, DeltaTime>,
        Entities<'a>,
    );

    fn run(&mut self, (mut fishables, mut renderables, delta_time, entities): Self::SystemData) {
        let mut remove_mes = Vec::new();

        for (e, fishable) in (&entities, &mut fishables).join() {
            fishable.time_left = fishable.time_left.saturating_sub(delta_time.0);
            if fishable.time_left.is_zero() {
                remove_mes.push(e);
            }
        }

        for me in remove_mes {
            renderables.remove(me);
            fishables.remove(me);
        }
    }
}
