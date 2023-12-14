use std::time::Duration;

use crate::{
    components::{
        DeleteCondition, FinishedActivity, FishAction, FishOnTheLine, Fishable, FishingMinigame, GameAction, Name,
        Renderable, WaitingForFish, Water,
    },
    game_init::PlayerEntity,
    items::{ItemID, ItemSpawner, SpawnType},
    tile_animation::TileAnimationBuilder,
    time::DeltaTime,
    ui::message_log::MessageLog,
    z_order::EFFECT_Z,
};

pub const WHITE: (u8, u8, u8) = (255, 255, 255);

use bracket_random::prelude::*;
use bracket_terminal::prelude::BLACK;
use log::info;
use specs::{Entities, Join, Read, ReadStorage, System, Write, WriteExpect, WriteStorage};

pub struct SetupFishingActions;

impl<'a> System<'a> for SetupFishingActions {
    type SystemData =
        (Entities<'a>, WriteStorage<'a, FishAction>, WriteStorage<'a, WaitingForFish>, Write<'a, TileAnimationBuilder>);

    fn run(&mut self, (entities, mut fish_actions, mut fish_waiters, mut anim_builder): Self::SystemData) {
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
        WriteStorage<'a, FishingMinigame>,
        WriteStorage<'a, FinishedActivity>,
        Read<'a, PlayerEntity>,
        Read<'a, DeltaTime>,
        WriteExpect<'a, MessageLog>,
        ReadStorage<'a, Name>,
    );

    fn run(
        &mut self,
        (
            entities,
            mut waiters,
            mut fishing_lines,
            mut minigames,
            mut finished_activities,
            p_entity,
            dt,
            mut log,
            names,
        ): Self::SystemData,
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
            log.debug(format!("Attempts left: {} | Rolled: {} ", waiter.attempts, roll));

            if roll < 80 {
                continue;
            }

            // Bite on the line
            finished_fishers.push(e);
            if e == p_entity.0 {
                let _ = minigames.insert(
                    e,
                    FishingMinigame {
                        cursor: Cursor::new(25.0),
                        goal_bar: GoalBar { goal: 5, bar_width: 18, goal_width: 3 },
                        attempts_left: 3,
                    },
                );
            } else {
                info!("{} caught a fish, o cool", name);
                log.log(format!("{} caught a fish wow with {} attempts remaining", name, waiter.attempts));
            }

            match fishing_lines.insert(e, FishOnTheLine {}) {
                Ok(existing_fish) => {
                    if let Some(fish) = existing_fish {
                        log.debug(format!("ERROR: entity {} {} already had a fish on their line, cannot add a second fish ABORTING fish", name, e.id()));
                        let _ = fishing_lines.insert(e, fish);
                    }
                }
                Err(err) => {
                    log.debug(format!("ERROR: entity: {} {} failed to add fish on the line: {}", name, e.id(), err));
                }
            }
        }

        for finished in finished_fishers.iter() {
            waiters.remove(*finished);
            if *finished == p_entity.0 && minigames.contains(p_entity.0) {
                info!("Player entering minigame state");
                continue;
            }
            let _ = finished_activities.insert(*finished, FinishedActivity {});
        }
    }
}

pub struct FishingMinigameUpdate;

impl<'a> System<'a> for FishingMinigameUpdate {
    type SystemData = (WriteStorage<'a, FishingMinigame>, Read<'a, DeltaTime>);

    fn run(&mut self, (mut minigames, dt): Self::SystemData) {
        for minigame in (&mut minigames).join() {
            let seconds_past = dt.0.as_millis() as f32 / 1000.0;
            minigame.cursor.position += minigame.cursor.speed * seconds_past;

            if minigame.cursor.position >= minigame.goal_bar.bar_width as f32 {
                minigame.cursor.position = 0.0;
            }
        }
    }
}

pub struct FishingMinigameCheck;

impl<'a> System<'a> for FishingMinigameCheck {
    type SystemData = (
        WriteStorage<'a, GameAction>,
        WriteStorage<'a, FinishedActivity>,
        WriteStorage<'a, FishingMinigame>,
        WriteStorage<'a, FishOnTheLine>,
        Write<'a, MessageLog>,
        Read<'a, PlayerEntity>,
        Entities<'a>,
    );

    fn run(
        &mut self,
        (
            mut game_actions,
            mut finished_activities,
            mut minigames,
            mut hooks,
            mut log,
            p_entity,
            entities,
        ): Self::SystemData,
    ) {
        if let Some((fisher, _, _, game, ())) =
            (&entities, &game_actions, &hooks, &mut minigames, !&finished_activities)
                .join()
                .find(|(e, _, _, _, _)| *e == p_entity.0)
        {
            info!("Game action read, checking if in_pos");
            let idx = game.cursor.bar_position();
            let goal = game.goal_bar.goal;
            let goals = (goal..(goal + game.goal_bar.goal_width)).collect::<Vec<usize>>();
            if goals.contains(&idx) {
                // button was hit on time
                log.log("#[bright_green]Success!#[]");
                let _ = finished_activities.insert(fisher, FinishedActivity {});
            } else {
                log.log("#[orange]Missed#[] the fish zone.");
                game.attempts_left = game.attempts_left.saturating_sub(1);
                if game.attempts_left == 0 {
                    log.log("#[red]Ahhh, the fish got away.#[]");
                    hooks.remove(fisher);
                    minigames.remove(fisher);
                    let _ = finished_activities.insert(fisher, FinishedActivity {});
                }
            }
        }

        game_actions.clear();
    }
}

pub struct CatchFishSystem;

impl<'a> System<'a> for CatchFishSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, FishOnTheLine>,
        WriteStorage<'a, FishingMinigame>,
        WriteExpect<'a, ItemSpawner>,
        WriteExpect<'a, MessageLog>,
        ReadStorage<'a, FinishedActivity>,
        ReadStorage<'a, Name>,
    );

    fn run(
        &mut self,
        (entities, mut hooks, mut minigames, mut item_spawner, mut log, finished_activities, names): Self::SystemData,
    ) {
        let mut remove_mes = Vec::new();
        for (e, _, name, _) in (&entities, &hooks, &names, &finished_activities).join() {
            remove_mes.push((e, name));
            log.enhance(format!("{} caught a really big fish!", name));
            item_spawner.request(ItemID(3), SpawnType::InBag(e));
        }
        for (entity, _) in remove_mes.iter() {
            hooks.remove(*entity);
            minigames.remove(*entity);
        }
    }
}

pub struct UpdateFishingTiles;

// pub const BUBBLE_SPAWN_RATE: usize = 9000;
pub const BUBBLE_SPAWN_RATE: usize = 1000;
pub const BUBBLE_LIFETIME_SECS: u64 = 10;
impl<'a> System<'a> for UpdateFishingTiles {
    type SystemData = (WriteStorage<'a, Fishable>, WriteStorage<'a, Renderable>, ReadStorage<'a, Water>, Entities<'a>);

    fn run(&mut self, (mut fishables, mut renderables, waters, entities): Self::SystemData) {
        let mut rng = RandomNumberGenerator::new();
        let mut new_bubbles = Vec::new();
        for (_, _, entity) in (!(&fishables), &waters, &entities).join() {
            if rng.range(0, BUBBLE_SPAWN_RATE) < 3 {
                new_bubbles.push(entity);
            }
        }
        for bubble in new_bubbles {
            let _ = fishables.insert(bubble, Fishable { time_left: Duration::from_secs(BUBBLE_LIFETIME_SECS) });
            let _ = renderables.insert(bubble, Renderable::clear_bg(47, WHITE, EFFECT_Z));
        }
    }
}

pub struct PollFishingTiles;

impl<'a> System<'a> for PollFishingTiles {
    type SystemData = (WriteStorage<'a, Fishable>, WriteStorage<'a, Renderable>, Read<'a, DeltaTime>, Entities<'a>);

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

pub struct Cursor {
    /// The precise location of the cursor in the world
    pub position: f32,
    /// Speed = blocks per sec
    pub speed: f32,
}

impl Cursor {
    pub fn new(speed: f32) -> Self {
        Self { position: 0.0, speed }
    }

    /// Where the cursor is on the bar
    pub fn bar_position(&self) -> usize {
        self.position.trunc() as usize
    }
}

pub struct GoalBar {
    /// Index at which the goal is located at
    pub goal: usize,
    /// Size of the goals
    pub goal_width: usize,
    /// The width of the goal bar
    pub bar_width: usize,
}
