use crate::{BinaryOperator, Literal, LogicalOperator, Statement, UnaryOperator};
use std::rc::Rc;

#[derive(Debug)]
pub enum Expression {
    Ternary {
        condition: Box<Expression>,
        truthy: Box<Expression>,
        falsey: Box<Expression>,
    },
    Binary {
        left: Box<Expression>,
        right: Box<Expression>,
        operator: BinaryOperator,
    },
    Logical {
        left: Box<Expression>,
        right: Box<Expression>,
        operator: LogicalOperator,
    },
    Unary {
        expression: Box<Expression>,
        operator: UnaryOperator,
    },
    GroupingExpression(Box<Expression>),
    Literal(Literal),
    Variable {
        line: usize,
        column: usize,
        identifier: Rc<str>,
    },
    Assignment {
        line: usize,
        column: usize,
        identifier: Rc<str>,
        value: Box<Expression>,
    },
    AnonymousFunction {
        parameters: Rc<[Rc<str>]>,
        body: Rc<[Statement]>,
    },
    Call {
        line: usize,
        column: usize,
        callee: Box<Expression>,
        args: Box<[Expression]>,
    },
}

impl std::fmt::Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ternary {
                condition,
                truthy,
                falsey,
            } => write!(f, "(ternary {condition} {truthy} {falsey})"),
            Self::Binary {
                left,
                right,
                operator,
            } => write!(f, "({} {left} {right})", operator.kind),
            Self::Logical {
                left,
                right,
                operator,
            } => write!(f, "({} {left} {right})", operator.kind),
            Self::Unary {
                expression,
                operator,
            } => write!(f, "({} {expression})", operator.kind),
            Self::GroupingExpression(expression) => write!(f, "(group {expression})"),
            Self::Literal(literal) => write!(f, "{literal}"),
            Self::Variable { identifier, .. } => write!(f, "(ident {identifier})"),
            Self::Assignment {
                identifier, value, ..
            } => write!(f, "(assign {identifier} {value})"),
            Self::Call { callee, args, .. } => {
                if args.is_empty() {
                    write!(f, "(call {callee})")
                } else {
                    write!(f, "(call {callee} (args ")?;

                    for arg in args.iter().take(args.len() - 1) {
                        write!(f, "{arg} ")?;
                    }

                    write!(f, "{}))", args.last().unwrap())
                }
            }
            Self::AnonymousFunction { .. } => write!(f, "<anonymous fn>"),
        }
    }
}
