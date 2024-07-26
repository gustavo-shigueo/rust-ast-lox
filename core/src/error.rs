use std::error::Error as ErrorTrait;
use thiserror::Error as ThisError;

#[derive(Debug, ThisError)]
pub struct Error<E: ErrorTrait> {
    pub line: usize,
    pub column: usize,

    #[source]
    pub source: E,
}

impl<E: ErrorTrait> std::fmt::Display for Error<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.source)
    }
}
