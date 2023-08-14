use bracket_terminal::prelude::{
    render_draw_buffer, BTerm, ColorPair, DrawBatch, Point, Rect, TextAlign, BLACK, RGBA, WHITE,
};
use specs::World;

use crate::{message_log::MessageLog, CL_TEXT, DISPLAY_HEIGHT, DISPLAY_WIDTH};

// TODO: improve how writing to the screen occurs, how can we avoid doing many error prone offset
// calculations to print something on the screen?

pub fn draw_ui(ecs: &World, ctx: &mut BTerm) {
    let mut draw_batch = DrawBatch::new();
    draw_batch.target(CL_TEXT).cls();
    draw_batch.printer(
        Point::new(1, 2),
        &format!("#[pink]FPS: {}#[]", ctx.fps),
        TextAlign::Left,
        Some(RGBA::from_u8(0, 0, 0, 0)),
    );

    let message_box_pos = Rect {
        x1: 0,
        x2: DISPLAY_WIDTH as i32 * 2 - 1,
        y1: DISPLAY_HEIGHT as i32 * 2 - 6,
        y2: DISPLAY_HEIGHT as i32 * 2 - 1,
    };

    draw_message_log(ecs, &mut draw_batch, &message_box_pos);

    draw_batch.submit(CL_TEXT).expect("Batch error??");
    render_draw_buffer(ctx).expect("Render error??");
}

const DISPLAY_MESSAGES: usize = 5;

/// Draws the message log to the screen with the last `DISPLAY_MESSAGES` showing
pub fn draw_message_log(ecs: &World, draw_batch: &mut DrawBatch, bounds: &Rect) {
    draw_batch.draw_box(*bounds, ColorPair::new(WHITE, BLACK));

    let message_log = ecs.fetch::<MessageLog>();

    // Imagine a list where there is either one or five + items in them
    // 1. ["hi"]  <--\    2. ["hi","hi","hi","hi","hi",]
    // last_index: 1-/                       ^--5
    // we only want the last `DISPLAY_MESSAGES` in the vec so we iterate only that many times
    let bottom_of_log = DISPLAY_HEIGHT * 2;
    let mut msgs_left = DISPLAY_MESSAGES;
    for msg in message_log.messages.iter().enumerate().rev() {
        let mut output = format!("{}| {}", &msg.0, &msg.1);
        output.truncate(bounds.x2 as usize - 1);

        draw_batch.print(Point::new(1, bottom_of_log - msgs_left), output);
        msgs_left -= 1;
        if msgs_left <= 1 {
            return;
        }
    }
}
