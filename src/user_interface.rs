use bracket_terminal::prelude::DrawBatch;
use specs::World;

use crate::CL_TEXT;

pub fn draw_ui(ecs: &World) {
    let mut draw_batch = DrawBatch::new();
    draw_batch.target(CL_TEXT).cls();

    draw_batch.submit(CL_TEXT).expect("Batch error??");
}

