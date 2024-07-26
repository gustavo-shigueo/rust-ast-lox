#[derive(Debug)]
pub struct BinaryOperator {
    pub line: usize,
    pub column: usize,
    pub kind: BinaryOperatorKind,
}

impl std::ops::Deref for BinaryOperator {
    type Target = BinaryOperatorKind;

    fn deref(&self) -> &Self::Target {
        &self.kind
    }
}

#[derive(Debug)]
pub enum BinaryOperatorKind {
    Plus,
    Minus,
    Star,
    Slash,

    BangEqual,
    DoubleEquals,
    GreaterThan,
    GreaterEqual,
    LessThan,
    LessEqual,

    Comma,
}

impl std::fmt::Display for BinaryOperatorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Plus => write!(f, "+"),
            Self::Minus => write!(f, "-"),
            Self::Star => write!(f, "*"),
            Self::Slash => write!(f, "/"),
            Self::BangEqual => write!(f, "!="),
            Self::DoubleEquals => write!(f, "=="),
            Self::GreaterThan => write!(f, ">"),
            Self::GreaterEqual => write!(f, ">="),
            Self::LessThan => write!(f, "<"),
            Self::LessEqual => write!(f, "<="),
            Self::Comma => write!(f, ","),
        }
    }
}
