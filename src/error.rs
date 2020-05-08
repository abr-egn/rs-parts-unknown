use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Obstructed")]
    Obstructed,
    #[error("Out of bounds")]
    OutOfBounds,
    #[error("Out of range")]
    OutOfRange,
    #[error("No such creature")]
    NoSuchCreature,
    #[error("No such part")]
    NoSuchPart,
    #[error("No such card")]
    NoSuchCard,
    #[error("No such stat")]
    NoSuchStat,
    #[error("Dead creature")]
    DeadCreature,
    #[error("Broken part")]
    BrokenPart,
    #[error("Not enough {0}")]
    NotEnough(String),
    #[error("Invalid action")]
    InvalidAction,
    #[error("Unhandled action")]
    UnhandledAction,  // TODO: include the action
}

pub type Result<T> = std::result::Result<T, Error>;