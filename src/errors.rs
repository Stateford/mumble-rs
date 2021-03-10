use std::error::Error;
use std::fmt::Display;

#[derive(Debug)]
pub struct MumbleError {
    message: String
}

impl MumbleError {
    pub fn new(message: &str) -> Self {
        Self {
            message: message.to_string()
        }
    }
}

unsafe impl Send for MumbleError {}

impl Display for MumbleError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Mumble Error: {}", self.message)
    }
}

impl Error for MumbleError {}