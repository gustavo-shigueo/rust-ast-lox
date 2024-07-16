mod error;
pub mod scanner;
mod token;

pub type Result<T, E = error::Error> = core::result::Result<T, E>;
pub use error::Error;
pub use scanner::Scanner;
