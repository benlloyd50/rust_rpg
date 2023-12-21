use bracket_terminal::prelude::{DrawBatch, Point, *};
use specs::{Join, World, WorldExt};

use crate::{
    camera::get_camera_bounds,
    components::{Renderable, Transform, SizeFlexor},
    debug::CLEAR,
    map::render_map,
    time::DeltaTime,
    z_order::PLAYER_Z,
    Position, CL_INTERACTABLES, CL_WORLD,
};

pub const SPRITE_SPEED: f32 = 8.0;

pub fn update_fancy_positions(ecs: &World) {
    let mut transforms = ecs.write_storage::<Transform>();
    let positions = ecs.read_storage::<Position>();
    let flexors = ecs.read_storage::<SizeFlexor>();
    let dt = ecs.read_resource::<DeltaTime>();

    for (ftrans, pos, ()) in (&mut transforms, &positions, !(&flexors)).join() {
        ftrans.sprite_pos =
            lerp_point(&ftrans.sprite_pos, pos.x as f32, pos.y as f32, SPRITE_SPEED * dt.0.as_secs_f32());
    }
}

/// Draws the CL_INTERACTABLES and CL_WORLD sprites to the screen
pub fn draw_sprite_layers(ecs: &World) {
    let mut draw_batch = DrawBatch::new();

    draw_batch.target(CL_INTERACTABLES);
    draw_batch.cls();

    draw_sprites(ecs, &mut draw_batch);
    draw_fancy_sprites(ecs, &mut draw_batch);
    draw_batch.submit(CL_INTERACTABLES).expect("Batch error??");

    draw_batch.target(CL_WORLD);
    draw_batch.cls();
    render_map(ecs, &mut draw_batch);
    draw_batch.submit(CL_WORLD).expect("Batch error??");
}

fn draw_sprites(ecs: &World, draw_batch: &mut DrawBatch) {
    let positions = ecs.read_storage::<Position>();
    let renderables = ecs.read_storage::<Renderable>();
    let transforms = ecs.read_storage::<Transform>();

    let bounding_box = get_camera_bounds(ecs);

    let data = (&positions, &renderables, !&transforms)
        .join()
        .map(|(p, r, _)| (p, r))
        .filter(|(pos, _)| bounding_box.point_in_rect(pos.to_point()));
    for (pos, render) in data {
        draw_batch.set_with_z(
            Point::new(pos.x as i32 - bounding_box.x1, pos.y as i32 - bounding_box.y1),
            ColorPair { fg: render.color_pair.fg, bg: CLEAR },
            // render.color_pair,
            render.atlas_index,
            render.z_priority,
        );
    }
}

fn draw_fancy_sprites(ecs: &World, draw_batch: &mut DrawBatch) {
    let renderables = ecs.read_storage::<Renderable>();
    let transforms = ecs.read_storage::<Transform>();

    let bounding_box = get_camera_bounds(ecs);
    for (ftrans, render) in
        (&transforms, &renderables).join().filter(|(pos, _)| bounding_box.point_in_rect(pos.sprite_pos.into()))
    {
        let fx = ftrans.sprite_pos.x - bounding_box.x1 as f32;
        let fy = ftrans.sprite_pos.y - bounding_box.y1 as f32 + 1.0;
        let rendered_pos = PointF::new(fx, fy);
        draw_batch.set_fancy(
            rendered_pos,
            PLAYER_Z,
            ftrans.rotation,
            ftrans.scale,
            render.color_pair,
            render.atlas_index,
        );
    }
}

pub fn lerp_point(curr: &PointF, x: f32, y: f32, scalar: f32) -> PointF {
    let fx = curr.x + (x as f32 - curr.x) * scalar;
    let fy = curr.y + (y as f32 - curr.y) * scalar;
    PointF { x: fx, y: fy }
}

#[allow(dead_code)]
const COLORS: [&(u8, u8, u8); 11] = [
    &ROSYBROWN,
    &DARKSALMON,
    &BURLYWOOD,
    &CADETBLUE4,
    &ANTIQUEWHITE,
    &DARKGOLDENROD1,
    &CORNFLOWER_BLUE,
    &LIGHTPINK,
    &LIGHTCYAN4,
    &LIGHTGOLDENRODYELLOW,
    &LIGHTSALMON,
];
