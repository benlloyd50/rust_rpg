// In this file we are only concerned with the "backend" of the message logger, drawing is ui based
// therefore, drawing the message log is defined in user_interface
use std::fmt::Display;

use bracket_terminal::prelude::{ColorPair, DrawBatch, Point, Rect, TextAlign, RGBA, WHITESMOKE};
use specs::{World, WorldExt};

use crate::{
    colors::{INVENTORY_BACKGROUND, INVENTORY_OUTLINE},
    TurnCounter, CL_TEXT,
};

use super::drawing::AccentBox;

pub(crate) fn draw_message_log(draw_batch: &mut DrawBatch, ecs: &World) {
    let log = ecs.fetch::<MessageLog>();

    draw_batch.target(CL_TEXT);
    draw_batch.draw_accent_box(
        Rect::with_size(-1, 50, 70, 10),
        ColorPair::new(INVENTORY_OUTLINE, INVENTORY_BACKGROUND),
    );

    for (y_offset, message) in log.nth_recent(9).enumerate() {
        draw_batch.printer(
            Point::new(1, 51 + y_offset),
            message.colored(),
            TextAlign::Left,
            Some(RGBA::new()),
        );
    }
}

pub fn draw_turn_counter(draw_batch: &mut DrawBatch, ecs: &World) {
    let turn_counter = ecs.read_resource::<TurnCounter>();
    draw_batch.target(CL_TEXT);
    draw_batch.draw_accent_box(
        Rect::with_size(0, 1, 6 + turn_counter.0.to_string().len(), 2),
        ColorPair::new(INVENTORY_OUTLINE, INVENTORY_BACKGROUND),
    );
    draw_batch.print_color(
        Point { x: 1, y: 2 },
        format!("Turn:{}", turn_counter.0),
        ColorPair {
            fg: WHITESMOKE.into(),
            bg: INVENTORY_BACKGROUND.into(),
        },
    );
}

/// Resource used for logging to the message console on the screen to the player
pub struct MessageLog {
    pub messages: Vec<Message>,
}

impl MessageLog {
    pub fn new() -> Self {
        Self {
            messages: vec![Message::new(
                "Welcome to the world of rust_rpg!".to_string(),
                MessageType::Info,
            )],
        }
    }

    /// Adds info to the log
    pub fn log(&mut self, contents: impl ToString) {
        self.add_to_log(contents.to_string(), MessageType::Info);
    }

    /// Adds flavor to the log
    pub fn enhance(&mut self, contents: impl ToString) {
        self.add_to_log(contents.to_string(), MessageType::Flavor);
    }

    /// Adds debug info to the log
    pub fn debug(&mut self, contents: impl ToString) {
        self.add_to_log(contents.to_string(), MessageType::Debug);
    }

    /// Returns the nth most recent messages in the log
    pub fn nth_recent(&self, n: usize) -> impl Iterator<Item = &Message> {
        self.messages.iter().rev().take(n)
    }

    /// Adds a new message to the log. If the message is the same as it's predecessor then it will
    /// increment the `repeated` variable
    fn add_to_log(&mut self, contents: String, msg_type: MessageType) {
        if let Some(last_msg) = self.messages.last_mut() {
            if last_msg.contents.eq(&contents) && last_msg.kind.eq(&msg_type) {
                last_msg.repeated += 1;
                return;
            }
        };
        self.messages.push(Message::new(contents, msg_type));
    }
}

impl Default for MessageLog {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Message {
    pub kind: MessageType,
    pub contents: String,
    repeated: usize,
}

impl Message {
    fn new(contents: String, message_type: MessageType) -> Self {
        Self {
            kind: message_type,
            contents,
            repeated: 1,
        }
    }

    /// Returns a colored output of the message based on type and amt
    pub fn colored(&self) -> String {
        let color = match self.kind {
            MessageType::Info => "lightgray",
            MessageType::Debug => "orange",
            MessageType::Flavor => "white",
        };
        let suffix_amt = if self.repeated > 1 {
            format!(" x{}", self.repeated)
        } else {
            "".to_string()
        };
        format!("#[{}]{}#[]{}", color, &self.contents, suffix_amt)
    }
}

impl Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.contents)
    }
}

#[derive(PartialEq, Eq)]
pub enum MessageType {
    Flavor, // conversations, flavor text
    Info,   // game info ie Fishing attempts remaining
    Debug,  // only shown if debug is enabled
}
