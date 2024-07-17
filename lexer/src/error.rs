use thiserror::Error as ErrorTrait;

#[derive(Debug, ErrorTrait)]
pub struct Error {
    pub line: u32,
    pub column: u32,

    #[source]
    pub source: ErrorType,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.source)
    }
}

#[derive(Debug, ErrorTrait)]
pub enum ErrorType {
    #[error("Unterminated string.")]
    UnterminatedString,
    #[error(r#"Unexpected character "{0}"."#)]
    UnexpectedCharacter(char),
}
