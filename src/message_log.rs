// In this file we are only concerned with the "backend" of the message logger, drawing is ui based
// therefore, drawing the message log is defined in user_interface

use std::fmt::Display;

pub struct MessageLog {
    pub messages: Vec<Message>,
}

impl MessageLog {
    pub fn new() -> Self {
        Self {
            messages: vec![Message::new(
                "Welcome to the world of rust_rpg!".to_string(),
                MessageType::INFO,
            ),],
        }
    }

    pub fn log(&mut self, contents: impl ToString) {
        self.messages
            .push(Message::new(contents.to_string(), MessageType::INFO));
    }

    pub fn enhance(&mut self, contents: impl ToString) {
        self.messages
            .push(Message::new(contents.to_string(), MessageType::FLAVOR));
    }

    pub fn debug(&mut self, contents: impl ToString) {
        self.messages
            .push(Message::new(contents.to_string(), MessageType::DEBUG));
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
}

impl Message {
    fn new(contents: String, message_type: MessageType) -> Self {
        Self {
            kind: message_type,
            contents,
        }
    }
}

impl Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.contents)
    }
}

pub enum MessageType {
    FLAVOR, // conversations, flavor text
    INFO,   // game info ie Fishing attempts remaining
    DEBUG,  // only shown if debug is enabled
}
