#![deny(clippy::pedantic, clippy::nursery)]
#![allow(clippy::module_name_repetitions)]

mod error;
pub mod lexer;
mod token;

pub type Result<T, E = error::Error> = core::result::Result<T, E>;
pub use error::{Error, ErrorType};
pub use lexer::Lexer;
