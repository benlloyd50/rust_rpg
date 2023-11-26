use bracket_terminal::prelude::{to_char, ColorPair, DrawBatch, Point, ORANGE, PURPLE, WHITE};
use specs::{World, WorldExt};

use crate::{components::FishingMinigame, game_init::PlayerEntity, CL_TEXT, DISPLAY_WIDTH};

pub fn draw_fishing(draw_batch: &mut DrawBatch, ecs: &World) {
    let p_entity = ecs.read_resource::<PlayerEntity>();
    let minigames = ecs.read_storage::<FishingMinigame>();

    if let Some(minigame) = minigames.get(p_entity.0) {
        draw_batch.target(CL_TEXT);
        let left_bar_x = (DISPLAY_WIDTH / 2) - minigame.goal_bar.bar_width;
        draw_batch.print_color(
            Point::new(left_bar_x, 15),
            format!("{}", " ".repeat(minigame.goal_bar.bar_width)),
            ColorPair::new(WHITE, WHITE),
        );

        let left_goal_x = left_bar_x + minigame.goal_bar.goal;
        draw_batch.print_color(
            Point::new(left_goal_x, 15),
            " ",
            ColorPair::new(PURPLE, PURPLE),
        );

        let cursor_pos = minigame.cursor.bar_position();
        draw_batch.print_color(
            Point::new(left_bar_x + cursor_pos, 15),
            format!("{}", to_char(7)),
            ColorPair::new(ORANGE, WHITE),
        );
    }
}
