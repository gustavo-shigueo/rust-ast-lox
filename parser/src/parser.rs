use std::{ops::Not, rc::Rc};

use lexer::{Token, TokenKind};
use lox_core::{report, Error, Result};

use crate::{
    BinaryOperator, BinaryOperatorKind, Expression, Function, Literal, LogicalOperator,
    LogicalOperatorKind, ParserError, Reference, Statement, UnaryOperator, UnaryOperatorKind,
    MAX_NUMBER_OF_ARGUMENTS,
};

macro_rules! match_token {
    ($self: ident, $($kinds: pat),+ $(,)?) => {{
        match $self.peek().kind {
            $($kinds)|+ => {
                $self.next();
                true
            }
            _ => false
        }
    }};
    (peek: $self: ident, $($kinds: pat),+ $(,)?) => {{
        match $self.peek().kind {
            $($kinds)|+ => true,
            _ => false,
        }
    }};
}

macro_rules! binary_operators {
    (
        $self: ident;
        $(
            $(#[doc = $doc: literal])?
            ($step: ident, $next: ident) {
                $($tokens: pat => $operators: expr),+ $(,)?
            }
        ),+
        $(,)?
    ) => {
        $(
            $(#[doc = $doc])?
            fn $step(&mut $self) -> Result<Expression, ParserError> {
                if match_token!($self, $($tokens),+) {
                    let token = $self.previous();
                    return Err(Error {
                        line: token.line,
                        column: token.column.saturating_sub(token.len()),
                        source: ParserError::ExpectedExpression,
                    });
                }

                let mut expression = $self.$next()?;

                while match_token!($self, $($tokens),+) {
                    let token = $self.previous().clone();
                    let right = $self.$next()?.into();

                    expression = Expression::Binary {
                        left: expression.into(),
                        right,
                        operator: match token.kind {
                            $(
                                $tokens => BinaryOperator {
                                    line: token.line,
                                    column: token.column,
                                    kind: $operators
                                },
                            )+
                            _ => unreachable!(),
                        }
                    }
                }

                Ok(expression)
            }
        )+
    }
}

macro_rules! logical_operators {
    (
        $self: ident;
        $(
            $(#[doc = $doc: literal])?
            ($step: ident, $next: ident) {
                $($tokens: pat => $operators: expr),+ $(,)?
            }
        ),+
        $(,)?
    ) => {
        $(
            $(#[doc = $doc])?
            fn $step(&mut $self) -> Result<Expression, ParserError> {
                if match_token!($self, $($tokens),+) {
                    let token = $self.previous();
                    return Err(Error {
                        line: token.line,
                        column: token.column.saturating_sub(token.len()),
                        source: ParserError::ExpectedExpression,
                    });
                }

                let mut expression = $self.$next()?;

                while match_token!($self, $($tokens),+) {
                    let token = $self.previous().clone();
                    let right = $self.$next()?.into();

                    expression = Expression::Logical {
                        left: expression.into(),
                        right,
                        operator: match token.kind {
                            $(
                                $tokens => LogicalOperator {
                                    line: token.line,
                                    column: token.column,
                                    kind: $operators
                                },
                            )+
                            _ => unreachable!(),
                        }
                    }
                }

                Ok(expression)
            }
        )+
    }
}

macro_rules! error {
    ($self: ident, $source: expr) => {{
        let token = $self.previous();

        return Err(Error {
            line: token.line,
            column: token.column + token.len(),
            source: $source,
        });
    }};
}

pub struct Parser<'a> {
    current: usize,
    source: &'a str,
    tokens: &'a [Token],
}

impl<'a> Parser<'a> {
    #[must_use]
    pub const fn new(source: &'a str, tokens: &'a [Token]) -> Self {
        Self {
            current: 0,
            source,
            tokens,
        }
    }

    pub fn parse(&mut self) -> Vec<Statement> {
        self.program()
    }

    /// `program` -> `statement`* `EOF`
    fn program(&mut self) -> Vec<Statement> {
        let mut statements = vec![];
        let mut had_error = false;
        while !self.is_done() {
            match self.declaration() {
                Ok(stmt) if !had_error => statements.push(stmt),
                Ok(_) => (),
                Err(err) => {
                    had_error = true;
                    statements.clear();
                    report(self.source, &err);
                    self.sinchronyze();
                }
            }
        }

        statements
    }

    /// `declaration` ->
    ///     | `var_declaration`
    ///     | `function_declaration`
    ///     | `statement`
    ///     | `class_declaration`
    fn declaration(&mut self) -> Result<Statement, ParserError> {
        if match_token!(self, TokenKind::Var) {
            self.var_declaration()
        } else if match_token!(self, TokenKind::Fun) {
            self.function_declaration()
        } else if match_token!(self, TokenKind::Class) {
            self.class_declaration()
        } else {
            self.statement()
        }
    }

    /// `var_declaration` -> "var" `IDENTIFIER` ("=" `expression`)? ";"
    fn var_declaration(&mut self) -> Result<Statement, ParserError> {
        let var = self.previous().clone();
        if !match_token!(self, TokenKind::Identifier(_)) {
            error!(self, ParserError::ExpectedIdentifier);
        }

        let identifier = self.previous().clone();
        let name = match identifier.kind {
            TokenKind::Identifier(ref ident) => Rc::clone(ident),
            _ => unreachable!(),
        };

        let declaration = Statement::Declaration {
            line: var.line,
            column: var.column,
            identifier: name,
            initializer: match self.peek().kind {
                TokenKind::Equals => {
                    self.next();
                    Some(self.expression()?)
                }
                TokenKind::Semicolon => None,
                _ => {
                    return Err(Error {
                        line: identifier.line,
                        column: identifier.line + identifier.len(),
                        source: ParserError::ExpectedSemicolonOrInitializer,
                    })
                }
            },
        };

        if !match_token!(self, TokenKind::Semicolon) {
            error!(self, ParserError::ExpectedSemicolon);
        }

        Ok(declaration)
    }

    /// `function_declaration` -> "fun" `named_function`
    fn function_declaration(&mut self) -> Result<Statement, ParserError> {
        self.named_function(false)
    }

    /// `class_declaration` -> "class" `IDENTIFIER` ( "<" `IDENTIFIER` )? "{" function* "}"
    fn class_declaration(&mut self) -> Result<Statement, ParserError> {
        let token = self.previous().clone();

        let TokenKind::Identifier(identifier) = self.peek().kind.clone() else {
            error!(self, ParserError::ExpectedIdentifier);
        };

        self.next();

        let super_class = match_token!(self, TokenKind::LessThan)
            .then(|| {
                let TokenKind::Identifier(identifier) = self.peek().kind.clone() else {
                    error!(self, ParserError::ExpectedIdentifier);
                };

                self.next();

                let token = self.previous().clone();

                Ok(Expression::Variable(Reference {
                    line: token.line,
                    column: token.column,
                    identifier,
                }))
            })
            .transpose()?;

        if !match_token!(self, TokenKind::LeftCurly) {
            error!(self, ParserError::ExpectedLeftCurly);
        }

        let mut methods = vec![];
        while !self.is_done() && !match_token!(peek: self, TokenKind::RightCurly) {
            methods.push(match self.named_function(true)? {
                Statement::Function(function) => function,
                _ => unreachable!(),
            });
        }

        if !match_token!(self, TokenKind::RightCurly) {
            error!(self, ParserError::ExpectedRightCurly);
        }

        Ok(Statement::Class {
            line: token.line,
            column: token.column,
            identifier,
            super_class,
            methods: methods.into(),
        })
    }

    /// `named_function` -> `IDENTIFIER` `anonymous_function`
    fn named_function(&mut self, is_method: bool) -> Result<Statement, ParserError> {
        let token = if is_method {
            self.peek().clone()
        } else {
            self.previous().clone()
        };

        let TokenKind::Identifier(identifier) = self.peek().kind.clone() else {
            error!(self, ParserError::ExpectedIdentifier);
        };

        self.next();

        let Expression::AnonymousFunction { parameters, body } = self.anonymous_function()? else {
            unreachable!()
        };

        Ok(Statement::Function(Function {
            line: token.line,
            column: token.column,
            identifier,
            parameters,
            body,
        }))
    }

    /// `anonymous_function` -> "("  `parameters`? ")" `block`
    fn anonymous_function(&mut self) -> Result<Expression, ParserError> {
        if !match_token!(self, TokenKind::LeftParen) {
            error!(self, ParserError::ExpectedLeftParen);
        }

        let parameters = self.parameters()?;

        if !match_token!(self, TokenKind::RightParen) {
            error!(self, ParserError::ExpectedRightParen);
        }

        if !match_token!(self, TokenKind::LeftCurly) {
            error!(self, ParserError::ExpectedLeftCurly);
        }

        Ok(Expression::AnonymousFunction {
            parameters,
            body: match self.block()? {
                Statement::Block(statements) => statements.into(),
                _ => unreachable!(),
            },
        })
    }

    /// `parameters` -> (
    ///     `IDENTIFIER`
    ///     ("," `IDENTIFIER`){0, `MAX_NUMBER_OF_ARGUMENTS - 1`}
    ///     ","?
    /// )
    fn parameters(&mut self) -> Result<Rc<[Rc<str>]>, ParserError> {
        let mut parameters = Vec::with_capacity(MAX_NUMBER_OF_ARGUMENTS);

        loop {
            // This allows a trailing comma
            if match_token!(peek: self, TokenKind::RightParen) {
                break;
            }

            if parameters.len() == MAX_NUMBER_OF_ARGUMENTS {
                let token = self.peek().clone();

                // Report the error, but don't return it,
                // as the parser is still in a valid state
                report(
                    self.source,
                    &Error {
                        line: token.line,
                        column: token.column,
                        source: ParserError::ParameterLimitExceeded,
                    },
                );
            }

            if let TokenKind::Identifier(ident) = self.peek().kind.clone() {
                self.next();
                parameters.push(ident);
            } else {
                error!(self, ParserError::ExpectedIdentifier);
            }

            if !match_token!(self, TokenKind::Comma) {
                break;
            }
        }

        Ok(parameters.into())
    }

    /// `statement` ->
    ///     | `expression_statement`
    ///     | `block`
    ///     | `if_statement`
    ///     | `while_statement`
    ///     | `for_statement`
    ///     | `break_statement`
    ///     | `continue_statement`
    ///     | `return_statement`
    fn statement(&mut self) -> Result<Statement, ParserError> {
        let stmt = match self.peek().kind {
            TokenKind::LeftCurly => {
                self.next();
                self.block()
            }
            TokenKind::If => {
                self.next();
                self.if_statement()
            }
            TokenKind::While => {
                self.next();
                self.while_statement()
            }
            TokenKind::For => {
                self.next();
                self.for_statement()
            }
            TokenKind::Break => {
                self.next();
                self.break_statement()
            }
            TokenKind::Continue => {
                self.next();
                self.continue_statement()
            }
            TokenKind::Return => {
                self.next();
                self.return_statement()
            }
            _ => self.expression_statement(),
        };

        stmt
    }

    /// `if_statement` -> "if" "(" expression ")" statement ("else" statement)?
    fn if_statement(&mut self) -> Result<Statement, ParserError> {
        if !match_token!(self, TokenKind::LeftParen) {
            error!(self, ParserError::ExpectedLeftParen);
        }

        let condition = self.expression()?;

        if !match_token!(self, TokenKind::RightParen) {
            error!(self, ParserError::ExpectedRightParen);
        }

        let then_branch = self.statement()?.into();

        Ok(Statement::If {
            condition,
            then_branch,
            else_branch: match_token!(self, TokenKind::Else)
                .then(|| self.statement().map(Box::new))
                .transpose()?,
        })
    }

    /// `while_statement` -> "if" "(" expression ")" statement
    fn while_statement(&mut self) -> Result<Statement, ParserError> {
        if !match_token!(self, TokenKind::LeftParen) {
            error!(self, ParserError::ExpectedLeftParen);
        }

        let condition = self.expression()?;

        if !match_token!(self, TokenKind::RightParen) {
            error!(self, ParserError::ExpectedRightParen);
        }

        Ok(Statement::While {
            condition,
            body: self.statement()?.into(),
        })
    }

    /// `for_statement` ->
    ///     "for" "("
    ///         (`var_declaration` | `expression_statement` | ";")
    ///         expression? ";"
    ///         expression? ";"
    ///     ")" statement
    fn for_statement(&mut self) -> Result<Statement, ParserError> {
        if !match_token!(self, TokenKind::LeftParen) {
            error!(self, ParserError::ExpectedLeftParen);
        }

        let initializer = if match_token!(self, TokenKind::Semicolon) {
            None
        } else if match_token!(self, TokenKind::Var) {
            Some(self.var_declaration()?)
        } else {
            Some(self.expression_statement()?)
        };

        let condition = match_token!(peek: self, TokenKind::Semicolon)
            .not()
            .then(|| self.expression())
            .transpose()?
            .unwrap_or(Expression::Literal(Literal::Boolean(true)));

        if !match_token!(self, TokenKind::Semicolon) {
            error!(self, ParserError::ExpectedSemicolon);
        }

        let increment = match_token!(peek: self, TokenKind::RightParen)
            .not()
            .then(|| self.expression())
            .transpose()?;

        if !match_token!(self, TokenKind::RightParen) {
            error!(self, ParserError::ExpectedRightParen);
        }

        let mut stmt = self.statement()?;

        if let Some(increment) = increment {
            stmt = Statement::Block([stmt, Statement::Expression(increment)].into());
        }

        stmt = Statement::While {
            condition,
            body: stmt.into(),
        };

        if let Some(initializer) = initializer {
            stmt = Statement::Block([initializer, stmt].into());
        }

        Ok(stmt)
    }

    /// `break_statement` -> "break" ";"
    fn break_statement(&mut self) -> Result<Statement, ParserError> {
        let token = self.previous().clone();

        if !match_token!(self, TokenKind::Semicolon) {
            error!(self, ParserError::ExpectedSemicolon);
        }

        Ok(Statement::Break {
            line: token.line,
            column: token.column,
        })
    }

    /// `continue_statement` -> "continue" ";"
    fn continue_statement(&mut self) -> Result<Statement, ParserError> {
        let token = self.previous().clone();

        if !match_token!(self, TokenKind::Semicolon) {
            error!(self, ParserError::ExpectedSemicolon);
        }

        Ok(Statement::Continue {
            line: token.line,
            column: token.column,
        })
    }

    /// `return_statement` -> "return" `expression`? ";"
    fn return_statement(&mut self) -> Result<Statement, ParserError> {
        let token = self.previous().clone();
        if match_token!(self, TokenKind::Semicolon) {
            return Ok(Statement::Return {
                line: token.line,
                column: token.column,
                expression: None,
            });
        }

        let expression = Some(self.expression()?);

        if !match_token!(self, TokenKind::Semicolon) {
            error!(self, ParserError::ExpectedSemicolon);
        }

        Ok(Statement::Return {
            line: token.line,
            column: token.column,
            expression,
        })
    }

    /// `block` -> "{" `declaration`* "}"
    fn block(&mut self) -> Result<Statement, ParserError> {
        let mut statements = vec![];

        while !match_token!(peek: self, TokenKind::RightCurly, TokenKind::Eof) {
            statements.push(self.declaration()?);
        }

        if !match_token!(self, TokenKind::RightCurly) {
            error!(self, ParserError::ExpectedRightCurly);
        }

        Ok(Statement::Block(statements.into()))
    }

    /// `expression_statement` -> `expression` ";"
    fn expression_statement(&mut self) -> Result<Statement, ParserError> {
        let expression = self.expression()?;

        if !match_token!(self, TokenKind::Semicolon) {
            error!(self, ParserError::ExpectedSemicolon);
        }

        Ok(Statement::Expression(expression))
    }

    /// `expression` -> `comma`
    fn expression(&mut self) -> Result<Expression, ParserError> {
        self.comma()
    }

    /// `assignment` -> (call ".")? `IDENTIFIER` "=" `assignment` | `ternary`
    fn assignment(&mut self) -> Result<Expression, ParserError> {
        let mut expression = self.ternary()?;

        if match_token!(self, TokenKind::Equals) {
            let value = self.assignment()?.into();

            expression = match expression {
                Expression::Variable(reference) => Expression::Assignment { reference, value },
                Expression::Get {
                    object,
                    identifier,
                    line,
                    column,
                } => Expression::Set {
                    object,
                    identifier,
                    value,
                    line,
                    column,
                },
                _ => error!(self, ParserError::InvalidAssignmentTarget),
            };
        }

        Ok(expression)
    }

    /// `ternary` -> `or` ("?" `ternary` ':' `ternary`)?
    fn ternary(&mut self) -> Result<Expression, ParserError> {
        if match_token!(self, TokenKind::QuestionMark) {
            error!(self, ParserError::ExpectedExpression);
        }

        let expression = self.or()?;

        if !match_token!(self, TokenKind::QuestionMark) {
            return Ok(expression);
        }

        let truthy = self.ternary()?.into();

        if !match_token!(self, TokenKind::Colon) {
            error!(self, ParserError::UnterminatedTernary);
        }

        let falsey = self.ternary()?.into();

        Ok(Expression::Ternary {
            condition: expression.into(),
            truthy,
            falsey,
        })
    }

    logical_operators!(
        self;

        /// `or` -> and ("or" and)*
        (or, and) {
            TokenKind::Or => LogicalOperatorKind::Or,
        },

        /// `and` -> equality ("and" equality)*
        (and, equality) {
            TokenKind::And => LogicalOperatorKind::And,
        },
    );

    binary_operators!(
        self;

        /// `comma` -> `assignment` ("," `assignment`)*
        (comma, assignment) {
            TokenKind::Comma => BinaryOperatorKind::Comma,
        },

        /// `equality` -> `comparison` (("==" | "!=") `comparison`)*
        (equality, comparison) {
            TokenKind::BangEqual => BinaryOperatorKind::BangEqual,
            TokenKind::DoubleEquals => BinaryOperatorKind::DoubleEquals,
        },

        /// `comparison` -> `term` (("<" | "<=" | ">" | ">=") `term`)*
        (comparison, term) {
            TokenKind::LessThan => BinaryOperatorKind::LessThan,
            TokenKind::LessEqual => BinaryOperatorKind::LessEqual,
            TokenKind::GreaterEqual => BinaryOperatorKind::GreaterEqual,
            TokenKind::GreaterThan => BinaryOperatorKind::GreaterThan,
        },

        /// `term` -> `factor` (("+" | "-") `factor`)*
        (term, factor) {
            TokenKind::Plus => BinaryOperatorKind::Plus,
            TokenKind::Minus => BinaryOperatorKind::Minus,
        },

        /// `factor` -> `unary` (("*" | "/") `unary`)*
        (factor, unary) {
            TokenKind::Star => BinaryOperatorKind::Star,
            TokenKind::Slash => BinaryOperatorKind::Slash,
        }
    );

    /// `unary` -> ("!" | "-") `unary` | `call`
    fn unary(&mut self) -> Result<Expression, ParserError> {
        if !match_token!(self, TokenKind::Bang, TokenKind::Minus) {
            return self.call();
        }

        let operator = self.previous().clone();
        let expression = self.unary()?.into();

        Ok(Expression::Unary {
            expression,
            operator: UnaryOperator {
                line: operator.line,
                column: operator.column,
                kind: match operator.kind {
                    TokenKind::Bang => UnaryOperatorKind::Bang,
                    TokenKind::Minus => UnaryOperatorKind::Minus,
                    _ => unreachable!(),
                },
            },
        })
    }

    /// `call` -> `primary` ( "(" `arguments` ")" | "." `IDENTIFIER` )*
    fn call(&mut self) -> Result<Expression, ParserError> {
        let mut expression = self.primary()?;

        loop {
            if match_token!(self, TokenKind::LeftParen) {
                let token = self.previous();
                expression = Expression::Call {
                    line: token.line,
                    column: token.column,
                    callee: expression.into(),
                    args: self.arguments()?,
                };

                if !match_token!(self, TokenKind::RightParen) {
                    error!(self, ParserError::ExpectedRightParen);
                }
            } else if match_token!(self, TokenKind::Dot) {
                let TokenKind::Identifier(identifier) = self.peek().kind.clone() else {
                    error!(self, ParserError::ExpectedIdentifier);
                };

                let token = self.next();

                expression = Expression::Get {
                    line: token.line,
                    column: token.column,
                    object: expression.into(),
                    identifier,
                }
            } else {
                break;
            }
        }

        Ok(expression)
    }

    /// `arguments` -> (
    ///     `assignment`
    ///     ("," `assignment`){0, `MAX_NUMBER_OF_ARGUMENTS - 1`}
    ///     ","?
    /// )?
    fn arguments(&mut self) -> Result<Box<[Expression]>, ParserError> {
        let mut args = Vec::with_capacity(MAX_NUMBER_OF_ARGUMENTS);

        loop {
            // This allows a trailing comma
            if match_token!(peek: self, TokenKind::RightParen) {
                break;
            }

            if args.len() == MAX_NUMBER_OF_ARGUMENTS {
                let token = self.peek().clone();

                // Report the error, but don't return it,
                // as the parser is still in a valid state
                report(
                    self.source,
                    &Error {
                        line: token.line,
                        column: token.column,
                        source: ParserError::ArgumentLimitExceeded,
                    },
                );
            }

            // Using `assignment` to bypass the `comma` operator,
            // which is not allowed in an argument list
            args.push(self.assignment()?);

            if !match_token!(self, TokenKind::Comma) {
                break;
            }
        }

        Ok(args.into())
    }

    /// `primary` ->
    ///     | `STRING`
    ///     | `NUMBER`
    ///     | `IDENTIFIER`
    ///     | "true"
    ///     | "false"
    ///     | "nil"
    ///     | "(" `expression` ")"
    ///     | "fun" `anonymous_function`
    ///     | "super" "." `IDENTIFIER`
    fn primary(&mut self) -> Result<Expression, ParserError> {
        if match_token!(self, TokenKind::Identifier(_)) {
            let token = self.previous();
            return Ok(Expression::Variable(Reference {
                line: token.line,
                column: token.column,
                identifier: match token.kind {
                    TokenKind::Identifier(ref ident) => Rc::clone(ident),
                    _ => unreachable!(),
                },
            }));
        }

        if match_token!(self, TokenKind::This) {
            let token = self.previous();
            return Ok(Expression::This {
                line: token.line,
                column: token.column,
            });
        }

        if match_token!(self, TokenKind::Super) {
            let token = self.previous().clone();
            if !match_token!(self, TokenKind::Dot) {
                error!(self, ParserError::ExpectedDotAfterSuper);
            }

            let TokenKind::Identifier(identifier) = self.peek().kind.clone() else {
                error!(self, ParserError::ExpectedIdentifier);
            };

            self.next();

            return Ok(Expression::Super {
                line: token.line,
                column: token.column,
                method: identifier,
            });
        }

        if match_token!(self, TokenKind::True) {
            return Ok(Expression::Literal(Literal::Boolean(true)));
        }

        if match_token!(self, TokenKind::False) {
            return Ok(Expression::Literal(Literal::Boolean(false)));
        }

        if match_token!(self, TokenKind::Nil) {
            return Ok(Expression::Literal(Literal::Nil));
        }

        if match_token!(self, TokenKind::Number { .. } | TokenKind::String(_)) {
            return Ok(Expression::Literal(match self.previous().kind {
                TokenKind::String(ref string) => Literal::String(Rc::clone(string)),
                TokenKind::Number { value, .. } => Literal::Number(value),
                _ => unreachable!(),
            }));
        }

        if match_token!(self, TokenKind::LeftParen) {
            let expression = self.expression()?.into();

            if !match_token!(self, TokenKind::RightParen) {
                error!(self, ParserError::ExpectedRightParen);
            }

            return Ok(Expression::GroupingExpression(expression));
        }

        if match_token!(self, TokenKind::Fun) {
            return self.anonymous_function();
        }

        error!(self, ParserError::ExpectedExpression);
    }

    fn sinchronyze(&mut self) {
        self.next();

        while !self.is_done() {
            if self.previous().kind == TokenKind::Semicolon {
                return;
            }

            if match_token!(
                peek: self,
                TokenKind::If,
                TokenKind::For,
                TokenKind::While,
                TokenKind::Fun,
                TokenKind::Return,
                TokenKind::Class,
                TokenKind::Var,
            ) {
                return;
            }

            self.next();
        }
    }

    const fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    const fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }

    fn next(&mut self) -> &Token {
        if !self.is_done() {
            self.current += 1;
        }

        &self.tokens[self.current - 1]
    }

    fn is_done(&self) -> bool {
        self.peek().kind == TokenKind::Eof
    }
}
