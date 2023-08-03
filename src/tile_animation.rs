use bracket_terminal::prelude::{ColorPair, RGB};
use specs::{System, Write, WriteStorage, Entities, World};

use crate::components::{Position, Renderable, DeleteCondition};


#[derive(Default)]
pub struct TileAnimationBuilder {
    requests: Vec<TileAnimationRequest>
}

impl TileAnimationBuilder {
    pub fn new() -> Self {
        Self {
            requests: Vec::new()
        }
    }

    pub fn request(&mut self, atlas_index: usize, x: usize, y: usize, fg: RGB, bg: RGB, delete_condition: DeleteCondition) {
        self.requests.push(TileAnimationRequest { atlas_index , at: Position::new(x, y), fgbg: ColorPair::new(fg, bg), delete_condition });
    }
}

struct TileAnimationRequest {
    atlas_index: usize,
    at: Position,
    fgbg: ColorPair,
    delete_condition: DeleteCondition,
}

pub struct TileAnimationSpawner<'a> {
    pub world: &'a World,
}

impl<'a> System<'a> for TileAnimationSpawner<'a> {
    type SystemData = (
        Entities<'a>,
        Write<'a, TileAnimationBuilder>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, Renderable>,
        WriteStorage<'a, DeleteCondition>,
        );

    fn run(&mut self, (entities, mut anim_builder, mut positions, mut renderables, mut delete_conditions): Self::SystemData) {
        for TileAnimationRequest { atlas_index, at, fgbg, delete_condition } in anim_builder.requests.iter() {
            let new_anim = entities.create();
            let _ = positions.insert(new_anim, *at);
            let _ = renderables.insert(new_anim, Renderable { color_pair: *fgbg, atlas_index: *atlas_index });
            let _ = delete_conditions.insert(new_anim, *delete_condition);
        }

        anim_builder.requests.clear();  // Clear all requests since we just cleared them ^
    }
}
