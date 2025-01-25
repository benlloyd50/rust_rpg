use bracket_lib::terminal::{to_char, ColorPair, DrawBatch, Point, WHITE};
use specs::{World, WorldExt};

use crate::{
    char_c::{
        CH_BAR_LEFT, CH_BAR_MID, CH_BAR_RIGHT, CH_CURSOR, CH_GOAL_LEFT, CH_GOAL_MID, CH_GOAL_RIGHT, CH_GOAL_SINGLE,
        CH_LILFISH, CH_REELBAR_LEFT, CH_REELBAR_MID, CH_REELBAR_RIGHT, CH_REELLINE,
    },
    components::FishingMinigame,
    debug::CLEAR,
    fishing::GoalBar,
    game_init::PlayerEntity,
    CL_EFFECTS, CL_EFFECTS2, DISPLAY_WIDTH,
};

pub const MINIGAME_HEIGHT: usize = 10;
pub fn draw_fishing_bar(draw_batch: &mut DrawBatch, ecs: &World) {
    let p_entity = ecs.read_resource::<PlayerEntity>();
    let minigames = ecs.read_storage::<FishingMinigame>();

    if let Some(minigame) = minigames.get(p_entity.0) {
        let left_bar_x = (DISPLAY_WIDTH / 2) - (minigame.goal_bar.bar_width / 2);
        draw_batch.target(CL_EFFECTS);
        draw_batch.print(
            Point::new(left_bar_x, MINIGAME_HEIGHT),
            to_char(CH_BAR_MID).to_string().repeat(minigame.goal_bar.bar_width),
        );
        draw_batch.print(Point::new(left_bar_x, MINIGAME_HEIGHT), to_char(CH_BAR_LEFT));
        draw_batch
            .print(Point::new(left_bar_x + minigame.goal_bar.bar_width - 1, MINIGAME_HEIGHT), to_char(CH_BAR_RIGHT));

        // draw the reel percent
        draw_batch.print(
            Point::new(left_bar_x, MINIGAME_HEIGHT - 2),
            to_char(CH_REELBAR_MID).to_string().repeat(minigame.goal_bar.bar_width),
        );
        draw_batch.print(Point::new(left_bar_x, MINIGAME_HEIGHT - 2), to_char(CH_REELBAR_LEFT)); // left part
        draw_batch.print(
            Point::new(left_bar_x + minigame.goal_bar.bar_width - 1, MINIGAME_HEIGHT - 2),
            to_char(CH_REELBAR_RIGHT),
        );

        draw_goal_bar(&minigame.goal_bar, left_bar_x, draw_batch);

        draw_batch.target(CL_EFFECTS2);
        let reel_length = (minigame.reel.catch_percent / 100.0 * minigame.goal_bar.bar_width as f32).floor() as usize;
        draw_batch.print_color(
            Point::new(left_bar_x, MINIGAME_HEIGHT - 2),
            to_char(CH_REELLINE).to_string().repeat(reel_length),
            ColorPair { fg: WHITE.into(), bg: CLEAR },
        );
        draw_batch.print_color(
            Point::new(left_bar_x + reel_length, MINIGAME_HEIGHT - 2),
            to_char(CH_LILFISH),
            ColorPair { fg: WHITE.into(), bg: CLEAR },
        );

        let cursor_pos = minigame.cursor.bar_position();
        draw_batch.print_color(
            Point::new(left_bar_x + cursor_pos, MINIGAME_HEIGHT),
            format!("{}", to_char(CH_CURSOR)),
            ColorPair { fg: WHITE.into(), bg: CLEAR },
        );
    }
}

fn draw_goal_bar(goal_bar: &GoalBar, left_bar_x: usize, draw_batch: &mut DrawBatch) {
    let left_goal_x = left_bar_x + goal_bar.goal;
    // single width goal
    if goal_bar.goal_width == 1 {
        draw_batch.print(Point::new(left_goal_x, MINIGAME_HEIGHT), to_char(CH_GOAL_SINGLE));
        return;
    }
    // middle goal section
    draw_batch
        .print(Point::new(left_goal_x, MINIGAME_HEIGHT), to_char(CH_GOAL_MID).to_string().repeat(goal_bar.goal_width));
    // left goal bumper
    draw_batch.print(Point::new(left_goal_x, MINIGAME_HEIGHT), to_char(CH_GOAL_LEFT));
    // right goal bumper
    draw_batch.print(Point::new(left_goal_x + goal_bar.goal_width - 1, MINIGAME_HEIGHT), to_char(CH_GOAL_RIGHT));
}
