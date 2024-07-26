#![deny(clippy::pedantic, clippy::nursery)]
#![allow(clippy::module_name_repetitions)]

mod callable;
mod environment;
mod error;
mod interpreter;
mod value;

pub use callable::{Callable, CallableKind};
pub use environment::Environment;
pub use error::RuntimeError;
pub use interpreter::Interpreter;
pub use value::Value;
