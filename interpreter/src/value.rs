use std::rc::Rc;

use parser::Literal;

use crate::Callable;

#[derive(PartialEq, Debug, Clone)]
pub enum Value {
    String(Rc<str>),
    Number(f64),
    Boolean(bool),
    Nil,
    Callable(Callable),
}

impl From<Literal> for Value {
    fn from(literal: Literal) -> Self {
        match literal {
            Literal::String(string) => Self::String(string),
            Literal::Number(number) => Self::Number(number),
            Literal::Boolean(boolean) => Self::Boolean(boolean),
            Literal::Nil => Self::Nil,
        }
    }
}

impl Value {
    #[must_use]
    pub const fn type_name(&self) -> &'static str {
        match self {
            Self::String(_) => "string",
            Self::Number(_) => "number",
            Self::Boolean(_) => "boolean",
            Self::Nil => "nil",
            Self::Callable(_) => "function",
        }
    }

    /// Lox follows Ruby’s simple rule: `false` and `nil` are falsey,
    /// and everything else is truthy.
    #[must_use]
    pub const fn is_truthy(&self) -> bool {
        !matches!(self, Self::Nil | Self::Boolean(false))
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::String(string) => write!(f, "{string}"),
            Self::Number(num) => write!(f, "{num}"),
            Self::Boolean(true) => write!(f, "true"),
            Self::Boolean(false) => write!(f, "false"),
            Self::Nil => write!(f, "nil"),
            Self::Callable(function) => write!(f, "{function}"),
        }
    }
}
