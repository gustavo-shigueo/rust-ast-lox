#[derive(Debug)]
pub struct Token {
    pub line: u32,
    pub column: u32,
    pub token_type: TokenType,
}

#[derive(Debug, Clone)]
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
    BangEqual,
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
