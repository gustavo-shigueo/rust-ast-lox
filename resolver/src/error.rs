use std::rc::Rc;

use thiserror::Error as ErrorTrait;

#[derive(Debug, ErrorTrait)]
pub enum ResolverError {
    #[error("You cannot access a variable in its own initializer")]
    AttemptedToAccessVariableInItsOwnInitializer,

    #[error(r#"There is already a variable named "{0}" in the current scope"#)]
    AttemptedToRedeclareVariable(Rc<str>),

    #[error("Unexpected return statement outside of function")]
    UnexpectedReturnStatement,

    #[error("Unexpected break statement outside of loop")]
    UnexpectedBreakStatement,

    #[error("Unexpected continue statement outside of loop")]
    UnexpectedContinueStatement,

    #[error(r#"Unexpected "this" keyword outside of class"#)]
    UnexpectedThisKeyword,

    #[error("You cannot return a value from an initializer")]
    CannotReturnFromInitializer,

    #[error("A class cannot inherit from itself")]
    ClassCannotInheritFromItself,

    #[error(r#"Unexpected "this" keyword outside of subclass"#)]
    UnexpectedSuperKeyword,
}
