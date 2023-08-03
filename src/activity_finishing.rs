use specs::{System, ReadStorage, Entities, WriteStorage, Join};

use crate::components::{FinishedActivity, DeleteCondition};


/// Handles all the logic that would be performed after an activity is finished. i.e. when an
/// entity finishes fishing. Refer to FinishedActivity enum for a comprehensive list
pub struct ActivityFinishSystem;

impl<'a> System<'a> for ActivityFinishSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, FinishedActivity>,
        ReadStorage<'a, DeleteCondition>,
        );

    fn run(&mut self, (entities, mut finished_activities, delete_conditions): Self::SystemData) {
        for (e, condition) in (&entities, &delete_conditions).join() {
            match condition {
                DeleteCondition::Event(spawner) => {
                    if finished_activities.contains(*spawner) {
                        let _ = entities.delete(e);
                        finished_activities.remove(*spawner);
                    }
                }
                DeleteCondition::Timed(_) => {}
            }
        }
    }
}
