use std::time::Duration;

use crate::{
    components::{DeleteCondition, FinishedActivity, FishAction, FishOnTheLine, WaitingForFish},
    message_log::MessageLog,
    tile_animation::TileAnimationBuilder,
    time::DeltaTime,
};
use bracket_random::prelude::*;
use bracket_terminal::prelude::{BLACK, WHITE};
use specs::{Entities, Join, Read, System, Write, WriteExpect, WriteStorage};

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
        for (e, fish_action) in (&entities, &mut fish_actions).join() {
            let mut rng = RandomNumberGenerator::new();
            anim_builder.request(
                112,
                fish_action.target.x,
                fish_action.target.y,
                WHITE.into(),
                BLACK.into(),
                DeleteCondition::ActivityFinish(e),
            );

            let attempts = rng.range(2, 6); // this could be affected by a fishing skill level?
            match fish_waiters.insert(e, WaitingForFish::new(attempts)) {
                Ok(fishy) => {
                    if let Some(_) = fishy {
                        // TODO: Add these to the message_log
                        // println!("INFO: entity: {} was already waiting for fish, they should not have performed the action again", e.id());
                    }
                }
                Err(..) => {
                    // println!("Failed to start fishing for entity: {}", e.id());
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
    );

    fn run(
        &mut self,
        (entities, mut waiters, mut fishing_lines, mut finished_activities, dt, mut log): Self::SystemData,
    ) {
        let mut rng = RandomNumberGenerator::new();
        let mut finished_fishers = Vec::new();

        for (e, waiter) in (&entities, &mut waiters).join() {
            if waiter.attempts == 0 {
                finished_fishers.push(e);
                log.enhance(format!(
                    "entity: {} ran out of attempts to catch a fish",
                    e.id()
                ));
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
            log.enhance(format!(
                "Attempts left: {} | Rolled: {} ",
                waiter.attempts, roll
            ));

            if roll > 80 {
                finished_fishers.push(e);
                log.log(format!(
                    "you caught a fish wow with {} attempts remaining",
                    waiter.attempts
                ));

                // To prevent a fisher who is already catching from potentially catching again without waiting properly
                if fishing_lines.contains(e) {
                    log.debug(format!("ERROR: entity {} already had a fish on their line, cannot add a second fish ABORTING fish", e.id()));
                    fishing_lines.remove(e);
                    continue;
                }
                match fishing_lines.insert(e, FishOnTheLine) {
                    Ok(_) => {}
                    Err(err) => {
                        log.debug(format!(
                            "ERROR: entity: {} failed to add fish on the line: {}",
                            e.id(),
                            err
                        ));
                    }
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
    );

    fn run(&mut self, (entities, mut hooks, mut finished_activities): Self::SystemData) {
        let mut remove_mes = Vec::new();
        for (e, _) in (&entities, &hooks).join() {
            remove_mes.push(e);
            let _ = finished_activities.insert(e, FinishedActivity);
        }
        for me in remove_mes.iter() {
            // TODO: convert to log
            // println!("entity {} caught a really big fish!", me.id());
            hooks.remove(*me);
        }
    }
}
