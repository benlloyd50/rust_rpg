use bracket_terminal::prelude::{DrawBatch, Point};
use specs::{Join, World, WorldExt};

use crate::{components::Renderable, Position};

pub fn draw_sprites(ecs: &World, draw_batch: &mut DrawBatch) {
    let positions = ecs.read_storage::<Position>();
    let renderables = ecs.read_storage::<Renderable>();

    let data = (&positions, &renderables).join().collect::<Vec<_>>();
    // data.sort_by(|&a, &b| b.1.render_order.cmp(&a.1.render_order) );
    for (pos, render) in data.iter() {
        // let idx = map.xy_idx(pos.x, pos.y);
        // if map.visible_tiles[idx] { ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph) }
        draw_batch.set(
            Point::new(pos.x, pos.y),
            render.color_pair,
            render.atlas_index,
        );
    }
}

pub fn xy_to_idx(x: usize, y: usize, width: usize) -> usize {
    y * width + x
}
