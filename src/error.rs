use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("obstructed")]
    Obstructed,
    #[error("out of bounds")]
    OutOfBounds,
    #[error("no such creature")]
    NoSuchCreature,
}

pub type Result<T> = std::result::Result<T, Error>;