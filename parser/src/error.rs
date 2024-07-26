use thiserror::Error as ThisError;

pub const MAX_NUMBER_OF_ARGUMENTS: usize = 255;

#[derive(Debug, ThisError)]
pub enum ParserError {
    #[error("Expected expression")]
    ExpectedExpression,

    #[error(r#"Expected ":" in ternary expression"#)]
    UnterminatedTernary,

    #[error(r#"Expected ";" at the end of statement"#)]
    ExpectedSemicolon,

    #[error(r#"Expected identifier"#)]
    ExpectedIdentifier,

    #[error(r#"Expected ";" or initializer"#)]
    ExpectedSemicolonOrInitializer,

    #[error("Invalid assignment target")]
    InvalidAssignmentTarget,

    #[error(r#"Expected "{{""#)]
    ExpectedLeftCurly,

    #[error(r#"Expected "}}" after block"#)]
    ExpectedRightCurly,

    #[error(r#"Expected "(""#)]
    ExpectedLeftParen,

    #[error(r#"Expected ")" after expression"#)]
    ExpectedRightParen,

    #[error("Function cannot have more than {MAX_NUMBER_OF_ARGUMENTS} parameters")]
    ParameterLimitExceeded,

    #[error("Function cannot have more than {MAX_NUMBER_OF_ARGUMENTS} arguments")]
    ArgumentLimitExceeded,
}
