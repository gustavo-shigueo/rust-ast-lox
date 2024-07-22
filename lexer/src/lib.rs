#![deny(clippy::pedantic, clippy::nursery)]
#![allow(clippy::module_name_repetitions)]

mod lexer;
mod token;

pub use lexer::Lexer;
pub use token::{Token, TokenKind};
