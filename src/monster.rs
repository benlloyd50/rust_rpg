use std::time::Duration;

use bracket_random::prelude::RandomNumberGenerator;
use specs::{Join, ReadStorage, System, World, WorldExt, WriteStorage};

use crate::{
    components::{Monster, Name, Position, RandomWalkerAI},
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
    );

    fn run(&mut self, (mut positions, mons, names, randwalks): Self::SystemData) {
        let mut rng = RandomNumberGenerator::new();
        for (pos, _, name, _) in (&mut positions, &mons, &names, &randwalks).join() {
            match rng.range(0, 5) {
                0 => {
                    pos.x += 1;
                }
                1 => {
                    pos.y += 1;
                }
                2 => {
                    pos.y = pos.y.saturating_sub(1);
                }
                3 => {
                    pos.x = pos.x.saturating_sub(1);
                }
                4 => {
                    println!("{} eats some grass from the ground.", name.0);
                }
                _ => unreachable!("the range is [0, 3]"),
            }
        }
    }
}

const MONSTER_ACTION_DELAY: Duration = Duration::from_secs(1);

/// Delays all monster entities from moving while player is activity bound
pub fn check_monster_delay(ecs: &World, monster_delay: &mut Duration) -> bool {
    let delta_time = ecs.read_resource::<DeltaTime>();
    *monster_delay = monster_delay.checked_add(delta_time.0).unwrap();
    println!("monster delay is {:?}", monster_delay);

    if *monster_delay >= MONSTER_ACTION_DELAY {
        true
    } else {
        false
    }
}
