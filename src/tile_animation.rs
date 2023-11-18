use bracket_terminal::prelude::{ColorPair, RGB};
use specs::{Entities, Join, ReadStorage, System, Write, WriteStorage};

use crate::{
    components::{DeleteCondition, FinishedActivity, Position, Renderable},
    z_order::TILE_ANIM_Z,
};

#[derive(Default)]
pub struct TileAnimationBuilder {
    requests: Vec<TileAnimationRequest>,
}

impl TileAnimationBuilder {
    pub fn new() -> Self {
        Self {
            requests: Vec::new(),
        }
    }

    pub fn request(
        &mut self,
        atlas_index: u8,
        x: usize,
        y: usize,
        fg: RGB,
        bg: RGB,
        delete_condition: DeleteCondition,
    ) {
        self.requests.push(TileAnimationRequest {
            atlas_index,
            at: Position::new(x, y),
            fgbg: ColorPair::new(fg, bg),
            delete_condition,
        });
    }
}

struct TileAnimationRequest {
    atlas_index: u8,
    at: Position,
    fgbg: ColorPair,
    delete_condition: DeleteCondition,
}

pub struct TileAnimationSpawner;

impl<'a> System<'a> for TileAnimationSpawner {
    type SystemData = (
        Entities<'a>,
        Write<'a, TileAnimationBuilder>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, Renderable>,
        WriteStorage<'a, DeleteCondition>,
    );

    fn run(
        &mut self,
        (entities, mut anim_builder, mut positions, mut renderables, mut delete_conditions): Self::SystemData,
    ) {
        for TileAnimationRequest {
            atlas_index,
            at,
            fgbg,
            delete_condition,
        } in anim_builder.requests.iter()
        {
            let new_anim = entities.create();
            let _ = positions.insert(new_anim, *at);
            let _ = renderables.insert(
                new_anim,
                Renderable {
                    color_pair: *fgbg,
                    atlas_index: *atlas_index,
                    z_priority: TILE_ANIM_Z,
                },
            );
            let _ = delete_conditions.insert(new_anim, *delete_condition);
        }

        anim_builder.requests.clear(); // Clear all requests since we just cleared them ^
    }
}

pub struct TileAnimationCleanUpSystem;

impl<'a> System<'a> for TileAnimationCleanUpSystem {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, FinishedActivity>,
        ReadStorage<'a, DeleteCondition>,
    );

    fn run(&mut self, (entities, finished_activities, delete_conditions): Self::SystemData) {
        for (e, condition) in (&entities, &delete_conditions).join() {
            match condition {
                DeleteCondition::ActivityFinish(spawner) => {
                    if finished_activities.contains(*spawner) {
                        let _ = entities.delete(e);
                    }
                }
                DeleteCondition::_Timed(_) => {}
            }
        }
    }
}
