use std::rc::Rc;

#[derive(Debug)]
pub enum Expression {
    TernaryExpression {
        condition: Box<Expression>,
        truthy: Box<Expression>,
        falsy: Box<Expression>,
    },
    BinaryExpression {
        left: Box<Expression>,
        right: Box<Expression>,
        operator: BinaryOperator,
    },
    UnaryExpression {
        expression: Box<Expression>,
        operator: UnaryOperator,
    },
    GroupingExpression(Box<Expression>),
    Literal(Literal),
}

#[derive(Debug)]
pub enum BinaryOperator {
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

impl std::fmt::Display for BinaryOperator {
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

#[derive(Debug)]
pub enum UnaryOperator {
    Minus,
    Bang,
}

impl std::fmt::Display for UnaryOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Minus => write!(f, "-"),
            Self::Bang => write!(f, "!"),
        }
    }
}

#[derive(Debug)]
pub enum Literal {
    LitStr(Rc<str>),
    LitNum(f64),
    LitBool(bool),
    LitNil,
}

impl std::fmt::Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LitStr(string) => write!(f, r#""{string}""#),
            Self::LitNum(num) => write!(f, "{num}"),
            Self::LitBool(true) => write!(f, "true"),
            Self::LitBool(false) => write!(f, "false"),
            Self::LitNil => write!(f, "nil"),
        }
    }
}

impl std::fmt::Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TernaryExpression {
                condition,
                truthy,
                falsy,
            } => write!(f, "(ternary {condition} {truthy} {falsy})"),
            Self::BinaryExpression {
                left,
                right,
                operator,
            } => write!(f, "({operator} {left} {right})"),
            Self::UnaryExpression {
                expression,
                operator,
            } => write!(f, "({operator} {expression})"),
            Self::GroupingExpression(expression) => write!(f, "(group {expression})"),
            Self::Literal(literal) => write!(f, "{literal}"),
        }
    }
}

#[test]
fn test() {
    let expr = Expression::BinaryExpression {
        left: Expression::Literal(Literal::LitNum(2.0)).into(),
        right: Expression::Literal(Literal::LitNum(5.0)).into(),
        operator: BinaryOperator::Plus,
    };

    assert_eq!(expr.to_string(), "(+ 2 5)")
}
