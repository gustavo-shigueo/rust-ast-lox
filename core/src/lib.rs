mod error;
mod report;

pub use error::Error;
pub use report::report;

pub type Result<T, E> = core::result::Result<T, Error<E>>;
