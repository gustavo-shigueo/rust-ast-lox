#[derive(Debug)]
pub struct Token {
    line: u32,
    column: u32,
    token_type: TokenType,
}

#[derive(Debug)]
pub enum TokenType {
    Identifier(String),
    String(String),
    Number(f64),

    LeftParen,
    RightParen,
    LeftBracket,
    RightBracket,
    LeftCurly,
    RightCurly,

    Comma,
    Dot,
    Semicolon,

    Plus,
    Minus,
    Slash,
    Star,

    Bang,
    NotEquals,
    Equals,
    DoubleEquals,
    GreaterThan,
    GreaterEqual,
    LessThan,
    LessEqual,

    And,
    Or,
    True,
    False,

    If,
    Else,
    For,
    While,

    Fun,
    Return,

    Class,
    Super,
    This,

    Print,
    Nil,
    Var,

    Eof,
}
