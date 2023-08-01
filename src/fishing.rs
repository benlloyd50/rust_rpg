use specs::{System, WriteStorage, Entities, Join};

use crate::components::{FishAction, Position, Renderable};


pub struct SetupFishingActions;

impl<'a> System<'a> for SetupFishingActions {
    type SystemData = (
        WriteStorage<'a, FishAction>,
        Entities<'a>,
        WriteStorage<'a, Renderable>,
        WriteStorage<'a, Position>,
        );

    #[allow(dead_code)]
    fn run(&mut self, (mut fishactions, entities, renderables, positions): Self::SystemData) {
        for (_, fish_action) in (&entities, &mut fishactions).join() {
                //do stuff here
        }
    }
}


