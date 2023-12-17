use bracket_terminal::prelude::{to_char, ColorPair, DrawBatch, Point, WHITE};
use log::debug;
use specs::{World, WorldExt};

use crate::{
    components::FishingMinigame, debug::CLEAR, fishing::GoalBar, game_init::PlayerEntity, CL_EFFECTS, CL_EFFECTS2,
    DISPLAY_WIDTH,
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
            to_char(20).to_string().repeat(minigame.goal_bar.bar_width),
        );
        draw_batch.print(Point::new(left_bar_x, MINIGAME_HEIGHT), to_char(19));
        draw_batch.print(Point::new(left_bar_x + minigame.goal_bar.bar_width - 1, MINIGAME_HEIGHT), to_char(21));

        // draw the reel percent
        draw_batch.print(
            Point::new(left_bar_x, MINIGAME_HEIGHT - 2),
            to_char(26).to_string().repeat(minigame.goal_bar.bar_width),
        ); // middle part
        draw_batch.print(Point::new(left_bar_x, MINIGAME_HEIGHT - 2), to_char(25)); // left part
        draw_batch.print(Point::new(left_bar_x + minigame.goal_bar.bar_width - 1, MINIGAME_HEIGHT - 2), to_char(27)); // right part

        draw_goal_bar(&minigame.goal_bar, left_bar_x, draw_batch);

        draw_batch.target(CL_EFFECTS2);
        let reel_length = (minigame.reel.catch_percent / 100.0 * minigame.goal_bar.bar_width as f32).floor() as usize;
        debug!("{}", reel_length);
        draw_batch.print_color(
            Point::new(left_bar_x, MINIGAME_HEIGHT - 2),
            to_char(28).to_string().repeat(reel_length),
            ColorPair { fg: WHITE.into(), bg: CLEAR },
        ); // Draw the reel line
        draw_batch.print_color(
            Point::new(left_bar_x + reel_length, MINIGAME_HEIGHT - 2),
            to_char(29),
            ColorPair { fg: WHITE.into(), bg: CLEAR },
        ); // Draw the lil fish on the end

        let cursor_pos = minigame.cursor.bar_position();
        draw_batch.print_color(
            Point::new(left_bar_x + cursor_pos, MINIGAME_HEIGHT),
            format!("{}", to_char(16)),
            ColorPair { fg: WHITE.into(), bg: CLEAR },
        );
    }
}

fn draw_goal_bar(goal_bar: &GoalBar, left_bar_x: usize, draw_batch: &mut DrawBatch) {
    let left_goal_x = left_bar_x + goal_bar.goal;
    // single width goal
    if goal_bar.goal_width == 1 {
        draw_batch.print(Point::new(left_goal_x, MINIGAME_HEIGHT), to_char(18));
        return;
    }
    // middle goal section
    draw_batch.print(Point::new(left_goal_x, MINIGAME_HEIGHT), to_char(23).to_string().repeat(goal_bar.goal_width));
    // left goal bumper
    draw_batch.print(Point::new(left_goal_x, MINIGAME_HEIGHT), to_char(22));
    // right goal bumper
    draw_batch.print(Point::new(left_goal_x + goal_bar.goal_width - 1, MINIGAME_HEIGHT), to_char(24));
}
