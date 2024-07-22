pub mod error;
pub mod report;

pub use error::{Error, ErrorType};
pub use report::report;

pub type Result<T, E = Error> = core::result::Result<T, E>;
