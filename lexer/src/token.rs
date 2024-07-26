use std::rc::Rc;
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub line: usize,
    pub column: usize,
    pub kind: TokenKind,
}

impl std::ops::Deref for Token {
    type Target = TokenKind;

    fn deref(&self) -> &Self::Target {
        &self.kind
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Identifier(Rc<str>),
    String(Rc<str>),
    Number {
        /// The actual floating point value
        value: f64,

        /// The number lexeme as written in the code,
        /// used to know precisely the length of the
        /// lexeme, as it is possible it doesn't match
        /// `value.to_string().len()`
        lexeme: Rc<str>,
    },

    LeftParen,
    RightParen,
    LeftBracket,
    RightBracket,
    LeftCurly,
    RightCurly,

    Comma,
    Dot,
    Semicolon,
    QuestionMark,
    Colon,

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
    Break,
    Continue,

    Fun,
    Return,

    Class,
    Super,
    This,

    Nil,
    Var,

    Eof,
}

impl TokenKind {
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        matches!(self, Self::Eof)
    }

    #[must_use]
    pub fn len(&self) -> usize {
        match self {
            Self::Identifier(ref x) => x.len(),
            Self::String(ref x) => x.len() + 2,
            Self::Number { ref lexeme, .. } => lexeme.len(),
            Self::LeftParen
            | Self::RightParen
            | Self::LeftBracket
            | Self::RightBracket
            | Self::LeftCurly
            | Self::RightCurly
            | Self::Comma
            | Self::Dot
            | Self::Semicolon
            | Self::QuestionMark
            | Self::Colon
            | Self::Plus
            | Self::Minus
            | Self::Slash
            | Self::Star
            | Self::Bang
            | Self::GreaterThan
            | Self::LessThan
            | Self::Equals => 1,
            Self::BangEqual
            | Self::GreaterEqual
            | Self::LessEqual
            | Self::DoubleEquals
            | Self::If
            | Self::Or => 2,
            Self::And | Self::Fun | Self::Nil | Self::Var | Self::For => 3,
            Self::True | Self::This | Self::Else => 4,
            Self::Break | Self::False | Self::While | Self::Class | Self::Super => 5,
            Self::Return => 6,
            Self::Continue => 8,
            Self::Eof => 0,
        }
    }
}

impl std::fmt::Display for TokenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Identifier(name) => write!(f, "{name}"),
            Self::String(value) => write!(f, r#""{value}""#),
            Self::Number { value, .. } => write!(f, "{value}"),
            Self::LeftParen => write!(f, "("),
            Self::RightParen => write!(f, ")"),
            Self::LeftBracket => write!(f, "["),
            Self::RightBracket => write!(f, "]"),
            Self::LeftCurly => write!(f, "{{"),
            Self::RightCurly => write!(f, "}}"),
            Self::Comma => write!(f, ","),
            Self::Dot => write!(f, "."),
            Self::Semicolon => write!(f, ";"),
            Self::Colon => write!(f, ":"),
            Self::QuestionMark => write!(f, "?"),
            Self::Plus => write!(f, "+"),
            Self::Minus => write!(f, "-"),
            Self::Slash => write!(f, "/"),
            Self::Star => write!(f, "*"),
            Self::Bang => write!(f, "!"),
            Self::BangEqual => write!(f, "!="),
            Self::Equals => write!(f, "="),
            Self::DoubleEquals => write!(f, "=="),
            Self::GreaterThan => write!(f, ">"),
            Self::GreaterEqual => write!(f, ">="),
            Self::LessThan => write!(f, "<"),
            Self::LessEqual => write!(f, "<="),
            Self::And => write!(f, "and"),
            Self::Or => write!(f, "or"),
            Self::True => write!(f, "true"),
            Self::False => write!(f, "false"),
            Self::If => write!(f, "if"),
            Self::Else => write!(f, "else"),
            Self::For => write!(f, "for"),
            Self::While => write!(f, "while"),
            Self::Break => write!(f, "break"),
            Self::Continue => write!(f, "continue"),
            Self::Fun => write!(f, "fun"),
            Self::Return => write!(f, "return"),
            Self::Class => write!(f, "class"),
            Self::Super => write!(f, "super"),
            Self::This => write!(f, "this"),
            Self::Nil => write!(f, "nil"),
            Self::Var => write!(f, "var"),
            Self::Eof => write!(f, "EOF"),
        }
    }
}
