use bracket_terminal::prelude::{DrawBatch, Rect, WHITESMOKE, ColorPair, PURPLE, to_cp437, YELLOW4, TextAlign, RGBA, Point};
use specs::World;

use crate::{CL_TEXT, message_log::{MessageLog, MessageType}};

pub fn draw_ui(ecs: &World) {
    let mut draw_batch = DrawBatch::new();
    draw_batch.target(CL_TEXT).cls();
    
    draw_message_log(&mut draw_batch, &ecs);

    draw_batch.submit(CL_TEXT).expect("Batch error??");
}

fn draw_message_log(draw_batch: &mut DrawBatch, ecs: &World) {
    let message_log = ecs.fetch::<MessageLog>();
    
    draw_batch.draw_hollow_box(Rect::with_size(2, 48, 70, 10), ColorPair::new(WHITESMOKE, PURPLE));
    draw_batch.fill_region(Rect::with_size(3, 49, 69, 9), ColorPair::new(WHITESMOKE, YELLOW4), to_cp437(' '));

    let mut y_offset = 0;
    for message in message_log.messages.iter().rev().take(9) {
        let color = match message.kind {
            MessageType::INFO => {"white"},
            MessageType::DEBUG => {"pink"},
            MessageType::FLAVOR => {"blue"},
        };
        draw_batch.printer(Point::new(3, 49 + y_offset), format!("#[{}]{}#[]", color, &message.contents), TextAlign::Left, Some(RGBA::new()));
        y_offset += 1;
    }
}

