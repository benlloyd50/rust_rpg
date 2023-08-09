use bracket_terminal::prelude::{
    render_draw_buffer, BTerm, ColorPair, DrawBatch, Point, Rect, BLACK, PINK, WHITE,
};

use crate::{CL_TEXT, DISPLAY_HEIGHT, DISPLAY_WIDTH};

// idea: add this onto to draw batch to chain together ui calls
// pub fn draw_message_log(ctx: &mut BTerm) {}

pub fn draw_ui_layers(ctx: &mut BTerm) {
    let mut draw_batch = DrawBatch::new();
    draw_batch
        .target(CL_TEXT)
        .cls()
        .print_color_with_z(
            Point::new(1, 2),
            &format!("FPS: {}", ctx.fps),
            ColorPair::new(PINK, BLACK),
            1000,
        )
        .draw_box(
            Rect {
                x1: 0,
                x2: DISPLAY_WIDTH as i32 - 1,
                y1: DISPLAY_HEIGHT as i32 - 5,
                y2: DISPLAY_HEIGHT as i32 - 1,
            },
            ColorPair::new(WHITE, BLACK),
        )
        .submit(CL_TEXT)
        .expect("Batch error??");
    render_draw_buffer(ctx).expect("Render error??");
}
