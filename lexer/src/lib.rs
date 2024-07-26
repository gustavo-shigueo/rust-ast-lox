#![deny(clippy::pedantic, clippy::nursery)]
#![allow(clippy::module_name_repetitions)]

mod error;
mod lexer;
mod token;

pub use error::LexerError;
pub use lexer::Lexer;
pub use token::{Token, TokenKind};
