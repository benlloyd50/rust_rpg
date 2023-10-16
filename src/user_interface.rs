use bracket_terminal::prelude::{to_cp437, DrawBatch, Point, Rect, TextAlign, RGBA};
use specs::prelude::*;

use crate::{
    colors::{to_rgb, white_fg, INVENTORY_BACKGROUND, INVENTORY_OUTLINE},
    components::{Backpack, SelectedInventoryIdx},
    data_read::ENTITY_DB,
    game_init::PlayerEntity,
    inventory::{check_player_selection, SelectionStatus},
    message_log::MessageLog,
    AppState, CL_TEXT,
};

pub fn draw_ui(ecs: &World, appstate: &AppState) {
    let mut draw_batch = DrawBatch::new();
    draw_batch.target(CL_TEXT).cls();

    draw_message_log(&mut draw_batch, &ecs);

    match *appstate {
        AppState::PlayerInInventory => {
            draw_inventory(&mut draw_batch, &ecs);
            if check_player_selection(ecs) == SelectionStatus::SelectionWithoutAction {
                draw_use_menu(&mut draw_batch);
            }
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
        white_fg(to_rgb(INVENTORY_OUTLINE)),
    );
    draw_batch.fill_region(
        Rect::with_size(41, 3, 34, items_in_bag.len()),
        white_fg(to_rgb(INVENTORY_BACKGROUND)),
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
        draw_batch.print_color(
            Point::new(42, 2 + idx),
            format!("{:X}| {:03} {}", idx, qty.0, name),
            white_fg(to_rgb(INVENTORY_BACKGROUND)),
        );
        idx += 1;
    }

    let selected_indices = ecs.read_storage::<SelectedInventoryIdx>();
    if let Some(selection) = selected_indices.get(player_entity.0) {
        if selection.first_idx < items_in_bag.len() {
            draw_batch.print(Point::new(41, 3 + selection.first_idx), ">");
        }
    }
}

const POSSIBLE_ACTIONS: [&str; 4] = [
    "#[orange]U#[]se with",
    "#[orange]E#[]xamine",
    "#[orange]D#[]rop",
    "#[white]Cancel#[]",
];

fn draw_use_menu(draw_batch: &mut DrawBatch) {
    draw_batch.draw_hollow_box(
        Rect::with_size(28, 6, 10, POSSIBLE_ACTIONS.len() + 1),
        white_fg(to_rgb(INVENTORY_OUTLINE)),
    );
    draw_batch.fill_region(
        Rect::with_size(29, 7, 9, POSSIBLE_ACTIONS.len()),
        white_fg(to_rgb(INVENTORY_BACKGROUND)),
        to_cp437(' '),
    );

    // NOTE: we would probably want to keep track of what actions are possible for a specific item
    for (idx, action) in POSSIBLE_ACTIONS.iter().enumerate() {
        draw_batch.printer(
            Point::new(29, 7 + idx),
            action,
            TextAlign::Left,
            Some(to_rgb(INVENTORY_BACKGROUND).into()),
        );
    }
}

fn draw_message_log(draw_batch: &mut DrawBatch, ecs: &World) {
    let log = ecs.fetch::<MessageLog>();

    draw_batch.draw_hollow_box(
        Rect::with_size(-1, 50, 70, 10),
        white_fg(to_rgb(INVENTORY_OUTLINE)),
    );
    draw_batch.fill_region(
        Rect::with_size(0, 51, 69, 9),
        white_fg(to_rgb(INVENTORY_BACKGROUND)),
        to_cp437(' '),
    );

    let mut y_offset = 0;
    for message in log.nth_recent(9) {
        draw_batch.printer(
            Point::new(1, 51 + y_offset),
            message.colored(),
            TextAlign::Left,
            Some(RGBA::new()),
        );
        y_offset += 1;
    }
}
