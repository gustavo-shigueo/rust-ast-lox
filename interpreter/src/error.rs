use std::rc::Rc;

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

    #[error(r#"Undeclared variable "{0}""#)]
    UndeclaredVariable(Rc<str>),

    #[error(r#"Attempted to use variable "{0}" before it was assigned a value"#)]
    UnassignedVariable(Rc<str>),

    #[error("Unexpected break statement outside of loop")]
    Break,

    #[error("Unexpected continue statement outside of loop")]
    Continue,

    #[error(r#"Type "{0}" is not callable"#)]
    TypeIsNotCallable(&'static str),

    #[error("Function expected {expected} arguments but got {found}")]
    ImcorrectNumberOfArguments { expected: usize, found: usize },

    #[error("Unexpected return statement outside of function or method")]
    Return(Value),

    #[error(r#"Attempted to access property in value of type "{0}""#)]
    TypeIsNotInstance(&'static str),

    #[error(r#"Attempted to access undefined property "{0}""#)]
    UndefinedProperty(Rc<str>),

    #[error("A class can only inherit from another class")]
    SuperClassMustBeAClass,
}
