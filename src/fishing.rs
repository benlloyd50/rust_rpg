use std::time::Duration;

use specs::{System, WriteStorage, Entities, Join, Read};
use bracket_random::prelude::*;
use crate::{components::{FishAction, Position, WaitingForFish, FishOnTheLine}, time::DeltaTime};


pub struct SetupFishingActions;

impl<'a> System<'a> for SetupFishingActions {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, FishAction>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, WaitingForFish>,
        );

    fn run(&mut self, (entities, mut fishactions, positions, mut fish_waiters): Self::SystemData) {
        for (e, _fish_action, _pos) in (&entities, &mut fishactions, &positions).join() {
            let mut rng = RandomNumberGenerator::new();
            // TODO: Create request for a tile animation 

            let attempts = rng.range(2, 6); // this could be affected by a fishing skill level?
            match fish_waiters.insert(e, WaitingForFish::new(attempts)) {
                Ok(fishy) => {
                    if let Some(_) = fishy {
                        println!("INFO: entity: {} was already waiting for fish, they should not have performed the action again", e.id());
                    }
                }
                Err(..) => {
                    println!("Failed to start fishing for entity: {}", e.id());
                    continue;
                }
            }
        }

        fishactions.clear();  // All fish actions in the current frame should be dealt with
    }
}

pub struct WaitingForFishSystem; 
const FISH_DELAY_TIME: Duration = Duration::new(1, 0);

impl <'a> System<'a> for WaitingForFishSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, WaitingForFish>,
        WriteStorage<'a, FishOnTheLine>,
        Read<'a, DeltaTime>,
        );

    fn run(&mut self, (entities, mut waiters, mut fishing_lines, dt): Self::SystemData) {
        let mut rng = RandomNumberGenerator::new();
        let mut finished_fishers = Vec::new();

        for (e, waiter) in (&entities, &mut waiters).join() {
            if waiter.attempts == 0 {
                finished_fishers.push(e);
                println!("entity: {} ran out of attempts to catch a fish", e.id());
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
            println!("Attempts left: {} | Rolled: {} ", waiter.attempts, roll);

            if roll > 80 {
                finished_fishers.push(e);
                println!("you caught a fish wow with {} attempts remaining", waiter.attempts);

                // To prevent a fisher who is already catching from potentially catching again without waiting properly
                if fishing_lines.contains(e) {
                    println!("ERROR: entity {} already had a fish on their line, cannot add a second fish ABORTING fish", e.id());
                    fishing_lines.remove(e); 
                    continue;
                }
                match fishing_lines.insert(e, FishOnTheLine) {
                    Ok(_) => {}
                    Err(err) => {
                        println!("ERROR: entity: {} failed to add fish on the line: {}", e.id(), err);
                    }
                }
            } 
        } 

        for finished in finished_fishers.iter() {
            waiters.remove(*finished);
        }
    }
}

pub struct CatchFishSystem;

impl<'a> System<'a> for CatchFishSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, FishOnTheLine>,
        );

    fn run(&mut self, (entities, mut hooks): Self::SystemData) {
        let mut remove_mes = Vec::new();
        for (e, _) in (&entities, &hooks).join() {
            remove_mes.push(e);
        }
        for me in remove_mes.iter() {
            println!("entity {} caught a really big fish!", me.id());
            hooks.remove(*me);
        }
        //clean up tile anim
    }
}
