use thiserror::Error as ThisError;

#[derive(Debug, ThisError)]
pub enum LexerError {
    #[error("Unterminated string")]
    UnterminatedString,

    #[error(r#"Unexpected character "{0}""#)]
    UnexpectedCharacter(char),
}
