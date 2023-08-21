use bracket_terminal::prelude::{render_draw_buffer, BTerm, DrawBatch, Point, *};
use specs::{Builder, Join, World, WorldExt};

use crate::{
    camera::get_bounding_box,
    components::{Breakable, HealthStats, Renderable, ToolType},
    map::render_map,
    Position, CL_INTERACTABLES, CL_WORLD,
};

pub fn draw_sprites(ecs: &World, draw_batch: &mut DrawBatch) {
    let positions = ecs.read_storage::<Position>();
    let renderables = ecs.read_storage::<Renderable>();

    let bounding_box = get_bounding_box(ecs);

    let data = (&positions, &renderables).join().collect::<Vec<_>>();
    for (pos, render) in data.iter() {
        if !(pos.x as i32 >= bounding_box.x1
            && bounding_box.x2 > pos.x as i32
            && pos.y as i32 >= bounding_box.y1
            && bounding_box.y2 > pos.y as i32)
        {
            continue;
        }
        draw_batch.set(
            Point::new(
                pos.x as i32 - bounding_box.x1,
                pos.y as i32 - bounding_box.y1,
            ),
            render.color_pair,
            render.atlas_index,
        );
    }
}

/// Draws the CL_INTERACTABLES and CL_WORLD sprites to the screen
pub fn draw_sprite_layers(ecs: &World, ctx: &mut BTerm) {
    let mut draw_batch = DrawBatch::new();

    draw_batch.target(CL_INTERACTABLES);
    draw_batch.cls();

    draw_sprites(&ecs, &mut draw_batch);
    draw_batch.submit(CL_INTERACTABLES).expect("Batch error??");

    draw_batch.target(CL_WORLD);
    draw_batch.cls();
    render_map(&ecs, &mut draw_batch);
    draw_batch.submit(CL_WORLD).expect("Batch error??");
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
                *COLORS[i],
                BLACK.into(),
                xy_to_idx(1, 4, 16),
            ))
            .with(Breakable::new(ToolType::Hand))
            .with(HealthStats::new(2, 0))
            .build();
    }
}

pub fn xy_to_idx(x: usize, y: usize, width: usize) -> usize {
    y * width + x
}
