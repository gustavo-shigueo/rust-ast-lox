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
    For {
        condition: Expression,
        increment: Option<Expression>,
        body: Box<Statement>,
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
    Function(Function),
    Return {
        line: usize,
        column: usize,
        expression: Option<Expression>,
    },
    Class {
        line: usize,
        column: usize,
        identifier: Rc<str>,
        super_class: Option<Expression>,
        methods: Rc<[Function]>,
    },
}

#[derive(Debug, Clone)]
pub struct Function {
    pub line: usize,
    pub column: usize,
    pub identifier: Rc<str>,
    pub parameters: Rc<[Rc<str>]>,
    pub body: Rc<[Statement]>,
}
