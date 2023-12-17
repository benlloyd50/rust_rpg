use bracket_geometry::prelude::Point;
use log::info;
use std::{collections::HashMap, time::Duration};
use uuid::Uuid;

use bracket_color::{prelude::ColorPair, rgba::RGBA};
use bracket_terminal::{
    prelude::{to_char, to_cp437, DrawBatch},
    FontCharType,
};
use specs::{Read, System, World, WorldExt, Write};

use crate::{
    colors::{Color, MIDDLERED},
    components::CLEAR,
    data_read::prelude::ANIMATION_DB,
    indexing::idx_to_point,
    time::DeltaTime,
    CL_EFFECTS,
};

#[derive(Clone)]
pub struct Animation {
    frames: Vec<Frame>,
    _height: usize,
    width: usize,
    time_between_ms: usize,
}

impl Animation {
    pub fn from_vec(
        frames: &[Vec<String>],
        char_keys: &HashMap<char, u8>,
        key_colors: &HashMap<char, Color>,
        time_between_ms: usize,
    ) -> Self {
        let height = frames[0].len();
        let width = frames[0][0].len();
        let frames: Vec<Frame> = frames.iter().map(|frame| Frame::from_vec(frame, char_keys, key_colors)).collect();
        Self { frames, _height: height, width, time_between_ms }
    }
}

#[derive(Copy, Clone)]
pub struct AnimationPlay {
    top_left: Point,
    curr_frame: usize,
    timer: Duration,
    looping: bool,
    lasting: bool,
    id: Uuid,
}

impl AnimationPlay {
    #[allow(dead_code)]
    pub fn looping(top_left: Point) -> Self {
        Self { curr_frame: 0, timer: Duration::ZERO, looping: true, lasting: false, top_left, id: Uuid::new_v4() }
    }

    pub fn lasting(top_left: Point) -> Self {
        Self { curr_frame: 0, timer: Duration::ZERO, looping: false, lasting: true, top_left, id: Uuid::new_v4() }
    }
}

#[derive(Clone)]
pub struct Frame {
    cells: Vec<Cell>,
}

impl Frame {
    fn from_vec(frame: &[String], keys: &HashMap<char, u8>, colors: &HashMap<char, Color>) -> Frame {
        let mut cells = vec![];
        for row in frame {
            for char in row.chars() {
                let val = match keys.get(&char) {
                    Some(val) => to_cp437(to_char(*val)),
                    None => to_cp437(char),
                };
                let fg = match colors.get(&char) {
                    Some(fg) => *fg,
                    None => MIDDLERED,
                };

                cells.push(Cell::from(val, fg));
            }
        }
        Frame { cells }
    }
}

#[derive(Clone)]
pub struct Cell {
    glyph: FontCharType,
    fg: Color,
    bg: RGBA,
}

impl Cell {
    fn from(glyph: FontCharType, fg: Color) -> Self {
        Self { glyph, fg, bg: CLEAR }
    }
}

#[derive(Default)]
pub struct AnimationRenderer {
    running_anims: Vec<(Animation, AnimationPlay)>,
}

impl AnimationRenderer {
    pub fn new() -> Self {
        Self { running_anims: vec![] }
    }

    pub fn clear(&mut self) {
        self.running_anims.clear();
    }

    pub fn request(&mut self, name: &str, play: AnimationPlay) {
        let adb = &ANIMATION_DB.lock().unwrap();
        let anim = match adb.get_by_name(name) {
            Some(a) => a,
            None => return,
        };

        self.running_anims.push((anim.clone(), play));
    }
}

pub struct UpdateAnimationTimers;

impl<'a> System<'a> for UpdateAnimationTimers {
    type SystemData = (Write<'a, AnimationRenderer>, Read<'a, DeltaTime>);

    fn run(&mut self, (mut anim_render, dt): Self::SystemData) {
        let mut remove_me_uuid = vec![];

        for (anim, play) in anim_render.running_anims.iter_mut() {
            if play.lasting && play.curr_frame >= anim.frames.len() - 1 {
                continue; // get out so we dont increment the frame
            }
            play.timer = play.timer.saturating_add(dt.0);

            if !play.lasting && play.curr_frame >= anim.frames.len() {
                remove_me_uuid.push(play.id);
                continue; // get out so we dont increment the frame
            } else if play.looping && play.curr_frame >= anim.frames.len() {
                play.curr_frame = 0; // restart the animation
                continue;
            }

            if play.timer.as_millis() > anim.time_between_ms as u128 {
                play.curr_frame += 1;
                play.timer = Duration::ZERO;
                info!("Animation at {}", play.curr_frame);
            }
        }

        for remove in remove_me_uuid {
            if let Some(idx) = anim_render.running_anims.iter().position(|(_, play)| play.id == remove) {
                anim_render.running_anims.remove(idx);
            }
        }
    }
}

pub fn print_frame_animations(draw_batch: &mut DrawBatch, ecs: &World) {
    let anim_render = ecs.read_resource::<AnimationRenderer>();
    for (anim, play) in anim_render.running_anims.iter() {
        let curr_frame = match anim.frames.get(play.curr_frame) {
            Some(f) => f,
            None => continue,
        };

        let mut i = 0;
        // TODO: this should be part of anim defined in the json?
        draw_batch.target(CL_EFFECTS);
        for cell in &curr_frame.cells {
            let Point { x, y } = idx_to_point(i, anim.width);
            draw_batch.set(
                Point::new(play.top_left.x + x, play.top_left.y + y),
                ColorPair { fg: cell.fg.into(), bg: cell.bg },
                cell.glyph,
            );

            i += 1;
        }
    }
}
