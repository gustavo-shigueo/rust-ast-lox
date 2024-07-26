use std::rc::Rc;

#[derive(PartialEq, Debug, Clone)]
pub enum Literal {
    String(Rc<str>),
    Number(f64),
    Boolean(bool),
    Nil,
}

impl Literal {
    #[must_use]
    pub const fn type_name(&self) -> &'static str {
        match self {
            Self::String(_) => "string",
            Self::Number(_) => "number",
            Self::Boolean(_) => "boolean",
            Self::Nil => "nil",
        }
    }

    /// Lox follows Ruby’s simple rule: `false` and `nil` are falsey,
    /// and everything else is truthy.
    #[must_use]
    pub const fn is_truthy(&self) -> bool {
        !matches!(self, Self::Nil | Self::Boolean(false))
    }
}

impl std::fmt::Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::String(string) => write!(f, "{string}"),
            Self::Number(num) => write!(f, "{num}"),
            Self::Boolean(true) => write!(f, "true"),
            Self::Boolean(false) => write!(f, "false"),
            Self::Nil => write!(f, "nil"),
        }
    }
}
