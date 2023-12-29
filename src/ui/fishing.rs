use bracket_lib::terminal::{to_char, ColorPair, DrawBatch, Point, WHITE};
use specs::{World, WorldExt};

use crate::{
    components::FishingMinigame, debug::CLEAR, fishing::GoalBar, game_init::PlayerEntity, CL_EFFECTS, CL_EFFECTS2,
    DISPLAY_WIDTH,
};

const CH_LILFISH: u8 = 29;
const CH_REELLINE: u8 = 28;
const CH_CURSOR: u8 = 16;

const CH_REELBAR_MID: u8 = 26;
const CH_REELBAR_LEFT: u8 = 25;
const CH_REELBAR_RIGHT: u8 = 27;

const CH_BAR_MID: u8 = 20;
const CH_BAR_LEFT: u8 = 19;
const CH_BAR_RIGHT: u8 = 21;

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

const CH_GOAL_SINGLE: u8 = 18;
const CH_GOAL_MID: u8 = 23;
const CH_GOAL_LEFT: u8 = 22;
const CH_GOAL_RIGHT: u8 = 24;

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
