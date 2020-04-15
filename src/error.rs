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
    #[error("dead creature")]
    DeadCreature,
    #[error("dead part")]
    DeadPart,
    #[error("not enough")]
    NotEnough,
    #[error("invalid action")]
    InvalidAction,
}

pub type Result<T> = std::result::Result<T, Error>;