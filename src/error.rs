use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("obstructed")]
    Obstructed,
    #[error("out of bounds")]
    OutOfBounds,
    #[error("no such creature")]
    NoSuchCreature,
    #[error("no such part")]
    NoSuchPart,
    #[error("no such card")]
    NoSuchCard,
    #[error("dead creature")]
    DeadCreature,
    #[error("broken part")]
    BrokenPart,
    #[error("not enough")]
    NotEnough,
    #[error("invalid action")]
    InvalidAction,
}

pub type Result<T> = std::result::Result<T, Error>;