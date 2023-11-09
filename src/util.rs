use std::fmt;

/// Milliseconds since the UNIX epoch
pub fn timestamp() -> u128 {
    let now = std::time::SystemTime::now();
    let since_the_epoch = now.duration_since(std::time::UNIX_EPOCH).expect("Time went backwards");
    let millis = since_the_epoch.as_millis();
    millis
}

#[derive(Debug)]
pub enum GameError {
    InsufficientFunds,
    MaxLevelReached,
    OutOfBounds,
    AlreadyPlanted,
    AlreadyFarmed,
    NotYetReady,
    TooManyFields,
}

impl fmt::Display for GameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            GameError::InsufficientFunds => "Insufficient funds",
            GameError::MaxLevelReached => "Max level reached",
            GameError::OutOfBounds => "Out of bounds",
            GameError::AlreadyPlanted => "Already planted",
            GameError::AlreadyFarmed => "Already farmed",
            GameError::NotYetReady => "Not yet ready",
            GameError::TooManyFields => "Too many fields",
        };
        write!(f, "{s}")
    }
}

pub type Result<T> = core::result::Result<T, GameError>;

pub fn seconds_to_millis(seconds: u128) -> u128 {
    seconds * 1000
}
