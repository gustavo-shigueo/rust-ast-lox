use std::rc::Rc;

use crate::Expression;

#[derive(Debug)]
pub enum Statement {
    Expression(Expression),
    Declaration {
        line: usize,
        column: usize,
        identifier: Rc<str>,
        initializer: Option<Expression>,
    },
    Block(Box<[Statement]>),
    If {
        condition: Expression,
        then_branch: Box<Statement>,
        else_branch: Option<Box<Statement>>,
    },
    While {
        condition: Expression,
        body: Box<Statement>,
    },
    Break {
        line: usize,
        column: usize,
    },
    Continue {
        line: usize,
        column: usize,
    },
    Function {
        identifier: Rc<str>,
        parameters: Rc<[Rc<str>]>,
        body: Rc<[Statement]>,
    },
    Return {
        line: usize,
        column: usize,
        expression: Option<Expression>,
    },
}
