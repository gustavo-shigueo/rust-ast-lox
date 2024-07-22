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

impl TokenKind {
    pub fn len(&self) -> usize {
        match self {
            TokenKind::Identifier(ref x) => x.len(),
            TokenKind::String(ref x) => x.len() + 2,
            TokenKind::Number { ref lexeme, .. } => lexeme.len(),
            TokenKind::LeftParen
            | TokenKind::RightParen
            | TokenKind::LeftBracket
            | TokenKind::RightBracket
            | TokenKind::LeftCurly
            | TokenKind::RightCurly
            | TokenKind::Comma
            | TokenKind::Dot
            | TokenKind::Semicolon
            | TokenKind::QuestionMark
            | TokenKind::Colon
            | TokenKind::Plus
            | TokenKind::Minus
            | TokenKind::Slash
            | TokenKind::Star
            | TokenKind::Bang
            | TokenKind::GreaterThan
            | TokenKind::LessThan
            | TokenKind::Equals => 1,
            TokenKind::BangEqual
            | TokenKind::GreaterEqual
            | TokenKind::LessEqual
            | TokenKind::DoubleEquals
            | TokenKind::If
            | TokenKind::Or => 2,
            TokenKind::And | TokenKind::Fun | TokenKind::Nil | TokenKind::Var | TokenKind::For => 3,
            TokenKind::True | TokenKind::This | TokenKind::Else => 4,
            TokenKind::False
            | TokenKind::While
            | TokenKind::Class
            | TokenKind::Super
            | TokenKind::Print => 5,
            TokenKind::Return => 6,
            TokenKind::Eof => 0,
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
            Self::Fun => write!(f, "fun"),
            Self::Return => write!(f, "return"),
            Self::Class => write!(f, "class"),
            Self::Super => write!(f, "super"),
            Self::This => write!(f, "this"),
            Self::Print => write!(f, "print"),
            Self::Nil => write!(f, "nil"),
            Self::Var => write!(f, "var"),
            Self::Eof => write!(f, "EOF"),
        }
    }
}
