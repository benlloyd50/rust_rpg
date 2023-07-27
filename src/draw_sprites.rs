use bracket_terminal::prelude::{render_draw_buffer, BTerm, ColorPair, DrawBatch, Point, *};
use specs::{Builder, Join, World, WorldExt};

use crate::{
    components::{Breakable, Renderable},
    map::render_map,
    Position, CL_INTERACTABLES, CL_TEXT, CL_WORLD,
};

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

pub fn draw_all_layers(ecs: &World, ctx: &mut BTerm) {
    let mut draw_batch = DrawBatch::new();

    draw_batch.target(CL_INTERACTABLES);
    draw_batch.cls();
    draw_sprites(&ecs, &mut draw_batch);
    draw_batch.submit(CL_INTERACTABLES).expect("Batch error??");

    draw_batch.target(CL_TEXT).cls().print_color_with_z(
        Point::new(1, 2),
        &format!("FPS: {}", ctx.fps),
        ColorPair::new(PINK, BLACK),
        1000,
    );
    draw_batch.submit(CL_TEXT).expect("Batch error??");

    draw_batch.target(CL_WORLD);
    draw_batch.cls();
    render_map(&ecs, &mut draw_batch);
    draw_batch.submit(CL_WORLD).expect("Batch error??");

    render_draw_buffer(ctx).expect("Render error??");
}

const COLORS: [&'static (u8, u8, u8); 7] = [
    &ROSYBROWN,
    &DARKSALMON,
    &BURLYWOOD,
    &CADETBLUE4,
    &ANTIQUEWHITE,
    &DARKGOLDENROD1,
    &CORNFLOWER_BLUE,
];

pub fn debug_rocks(world: &mut World) {
    for i in 0..COLORS.len() {
        world
            .create_entity()
            .with(Position { x: 13 + i, y: 10 })
            .with(Renderable::new(
                ColorPair::new(*COLORS[i], BLACK),
                xy_to_idx(1, 4, 16),
            ))
            .with(Breakable::new(2, 0))
            .build();
    }
}

pub fn xy_to_idx(x: usize, y: usize, width: usize) -> usize {
    y * width + x
}