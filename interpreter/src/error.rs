use crate::Value;
use thiserror::Error as ErrorTrait;

#[derive(Debug, ErrorTrait)]
pub enum RuntimeError {
    #[error(r#"Expected expression of type "{expected}", found type "{found}""#)]
    TypeError {
        expected: &'static str,
        found: &'static str,
    },

    #[error("Attempted to divide by zero")]
    DivideByZero,

    #[error(r#"Undefined variable "{0}""#)]
    UndefinedVariable(String),

    #[error(r#"Attempted to use variable "{0}" before it was assigned a value"#)]
    UnassignedVariable(String),

    #[error("Unexpected break statement outside of loop")]
    UnexpectedBreakStatement,

    #[error("Unexpected continue statement outside of loop")]
    UnexpectedContinueStatement,

    #[error(r#"Type "{0}" is not callable"#)]
    TypeIsNotCallable(&'static str),

    #[error("Function expected {expected} arguments but got {found}")]
    ImcorrectNumberOfArguments { expected: usize, found: usize },

    #[error("Unexpected return statement outside of function or method")]
    UnexpectedReturnStatement(Value),
}
