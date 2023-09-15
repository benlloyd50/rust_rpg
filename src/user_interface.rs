use bracket_terminal::prelude::{
    to_cp437, ColorPair, DrawBatch, Point, Rect, TextAlign, RGB, RGBA, WHITESMOKE,
};
use specs::prelude::*;

use crate::{
    components::Backpack,
    data_read::ENTITY_DB,
    game_init::PlayerEntity,
    message_log::{MessageLog, MessageType},
    AppState, CL_TEXT,
};

pub fn draw_ui(ecs: &World, appstate: &AppState) {
    let mut draw_batch = DrawBatch::new();
    draw_batch.target(CL_TEXT).cls();

    draw_message_log(&mut draw_batch, &ecs);

    match *appstate {
        AppState::PlayerInInventory => {
            draw_inventory(&mut draw_batch, &ecs);
        }
        _ => {}
    }

    draw_batch.submit(CL_TEXT).expect("Batch error??");
}

fn draw_inventory(draw_batch: &mut DrawBatch, ecs: &World) {
    let player_entity = ecs.read_resource::<PlayerEntity>();
    let player_bag = ecs.read_storage::<Backpack>();

    let items_in_bag = match player_bag.get(player_entity.0) {
        Some(bag) => bag,
        None => {
            panic!("Player entity does not have a Backpack component.");
        }
    };

    draw_batch.draw_hollow_box(
        Rect::with_size(40, 2, 35, items_in_bag.len() + 1),
        ColorPair::new(WHITESMOKE, RGB::from_u8(61, 84, 107)),
    );
    draw_batch.fill_region(
        Rect::with_size(41, 3, 34, items_in_bag.len()),
        ColorPair::new(WHITESMOKE, RGB::from_u8(44, 57, 71)),
        to_cp437(' '),
    );

    let mut idx = 1;
    let edb = &ENTITY_DB.lock().unwrap();
    for (iid, qty) in items_in_bag.iter() {
        let name = match edb.items.get_by_id(iid.0) {
            Some(info) => &info.name,
            None => {
                eprintln!("ItemID: {:?} was not found in the Entity database", iid);
                "{} MISSING ITEM NAME"
            }
        };
        draw_batch.print(
            Point::new(41, 2 + idx),
            format!("{} |{} {}", idx, qty, name),
        );
        idx += 1;
    }
}

fn draw_message_log(draw_batch: &mut DrawBatch, ecs: &World) {
    let message_log = ecs.fetch::<MessageLog>();

    draw_batch.draw_hollow_box(
        Rect::with_size(-1, 50, 70, 10),
        ColorPair::new(WHITESMOKE, RGB::from_u8(61, 84, 107)),
    );
    draw_batch.fill_region(
        Rect::with_size(0, 51, 69, 9),
        ColorPair::new(WHITESMOKE, RGB::from_u8(44, 57, 71)),
        to_cp437(' '),
    );

    let mut y_offset = 0;
    for message in message_log.messages.iter().rev().take(9) {
        let color = match message.kind {
            MessageType::INFO => "lightgray",
            MessageType::DEBUG => "orange",
            MessageType::FLAVOR => "white",
        };
        draw_batch.printer(
            Point::new(1, 51 + y_offset),
            format!("#[{}]{}#[]", color, &message.contents),
            TextAlign::Left,
            Some(RGBA::new()),
        );
        y_offset += 1;
    }
}
