#[derive(Debug)]
pub struct UnaryOperator {
    pub line: usize,
    pub column: usize,
    pub kind: UnaryOperatorKind,
}

#[derive(Debug)]
pub enum UnaryOperatorKind {
    Minus,
    Bang,
}

impl std::fmt::Display for UnaryOperatorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Minus => write!(f, "-"),
            Self::Bang => write!(f, "!"),
        }
    }
}
