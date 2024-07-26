#![deny(clippy::pedantic, clippy::nursery)]
#![allow(clippy::module_name_repetitions)]

mod error;
mod expression;
mod literal;
mod operator;
mod parser;
mod statement;

pub use error::{ParserError, MAX_NUMBER_OF_ARGUMENTS};
pub use expression::Expression;
pub use literal::Literal;
pub use operator::{
    binary_operator::{BinaryOperator, BinaryOperatorKind},
    logical_operator::{LogicalOperator, LogicalOperatorKind},
    unary_operator::{UnaryOperator, UnaryOperatorKind},
};
pub use parser::Parser;
pub use statement::Statement;
