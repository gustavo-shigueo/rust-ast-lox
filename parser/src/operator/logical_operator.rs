#[derive(Debug)]
pub struct LogicalOperator {
    pub line: usize,
    pub column: usize,
    pub kind: LogicalOperatorKind,
}

impl std::ops::Deref for LogicalOperator {
    type Target = LogicalOperatorKind;

    fn deref(&self) -> &Self::Target {
        &self.kind
    }
}

#[derive(Debug)]
pub enum LogicalOperatorKind {
    And,
    Or,
}

impl std::fmt::Display for LogicalOperatorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::And => write!(f, "and"),
            Self::Or => write!(f, "or"),
        }
    }
}
