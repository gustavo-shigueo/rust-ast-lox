use std::rc::Rc;

use lexer::{Token, TokenKind};
use lox_core::{Error, ErrorType, Result};

use crate::{
    expression::{BinaryOperator, Literal, UnaryOperator},
    Expression,
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
                $($tokens: pat => $operators: ident),+ $(,)?
            }
        ),+
        $(,)?
    ) => {
        $(
            $(#[doc = $doc])?
            fn $step(&mut $self) -> Result<Expression> {
                if match_token!($self, $($tokens),+) {
                    let token = $self.previous();
                    return Err(Error {
                        line: token.line,
                        column: token.column.saturating_sub(token.len()),
                        source: ErrorType::ExpectedExpression,
                    });
                }

                let mut expression = $self.$next()?;

                while match_token!($self, $($tokens),+) {
                    let operator = $self.previous().kind.clone();
                    let right = $self.$next()?.into();

                    expression = Expression::BinaryExpression {
                        left: expression.into(),
                        right,
                        operator: match operator {
                            $(
                                $tokens => BinaryOperator::$operators,
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

pub struct Parser<'a> {
    current: usize,
    source: &'a str,
    tokens: &'a [Token],
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a str, tokens: &'a [Token]) -> Self {
        Self {
            current: 0,
            source,
            tokens,
        }
    }

    pub fn parse(&mut self) -> Result<Expression> {
        self.expression()
    }

    /// expression -> ternary
    fn expression(&mut self) -> Result<Expression> {
        self.ternary()
    }

    /// ternary -> comma ("?" ternary ':' ternary)?
    fn ternary(&mut self) -> Result<Expression> {
        if match_token!(self, TokenKind::QuestionMark) {
            let token = self.previous();
            return Err(Error {
                line: token.line,
                column: token.column.saturating_sub(token.len()),
                source: ErrorType::ExpectedExpression,
            });
        }

        let expression = self.comma()?;

        if !match_token!(self, TokenKind::QuestionMark) {
            return Ok(expression);
        }

        let truthy = self.ternary()?.into();

        if !match_token!(self, TokenKind::Colon) {
            let token = self.previous();

            return Err(Error {
                line: token.line,
                column: token.column + token.len(),
                source: ErrorType::UnterminatedTernary,
            });
        }

        let falsy = self.ternary()?.into();

        Ok(Expression::TernaryExpression {
            condition: expression.into(),
            truthy,
            falsy,
        })
    }

    binary_operators!(
        self;

        /// comma -> equality ("," equality)*
        (comma, equality) {
            TokenKind::Comma => Comma,
        },

        /// equality -> comparison (("==" | "!=") comparison)*
        (equality, comparison) {
            TokenKind::BangEqual => BangEqual,
            TokenKind::DoubleEquals => DoubleEquals,
        },

        /// comparison -> term (("<" | "<=" | ">" | ">=") term)*
        (comparison, term) {
            TokenKind::LessThan => LessThan,
            TokenKind::LessEqual => LessEqual,
            TokenKind::GreaterEqual => GreaterEqual,
            TokenKind::GreaterThan => GreaterThan,
        },

        /// term -> factor (("+" | "-") factor)*
        (term, factor) {
            TokenKind::Plus => Plus,
            TokenKind::Minus => Minus,
        },

        /// factor -> unary (("*" | "/") unary)*
        (factor, unary) {
            TokenKind::Star => Star,
            TokenKind::Slash => Slash,
        }
    );

    /// unary -> ("!" | "-") unary | primary
    fn unary(&mut self) -> Result<Expression> {
        if !match_token!(self, TokenKind::Bang, TokenKind::Minus) {
            return self.primary();
        }

        let operator = self.previous().kind.clone();
        let expression = self.unary()?.into();

        Ok(Expression::UnaryExpression {
            expression,
            operator: match operator {
                TokenKind::Bang => UnaryOperator::Bang,
                TokenKind::Minus => UnaryOperator::Minus,
                _ => unreachable!(),
            },
        })
    }

    /// primary -> STRING | NUMBER | "true" | "false" | "nil" | "(" expression ")"
    fn primary(&mut self) -> Result<Expression> {
        if match_token!(self, TokenKind::True) {
            return Ok(Expression::Literal(Literal::LitBool(true)));
        }

        if match_token!(self, TokenKind::False) {
            return Ok(Expression::Literal(Literal::LitBool(false)));
        }

        if match_token!(self, TokenKind::Nil) {
            return Ok(Expression::Literal(Literal::LitNil));
        }

        if match_token!(self, TokenKind::Number { .. } | TokenKind::String(_)) {
            return Ok(Expression::Literal(match self.previous().kind {
                TokenKind::String(ref string) => Literal::LitStr(Rc::clone(string)),
                TokenKind::Number { value, .. } => Literal::LitNum(value),
                _ => unreachable!(),
            }));
        }

        if match_token!(self, TokenKind::LeftParen) {
            let expression = self.expression()?.into();

            if !match_token!(self, TokenKind::RightParen) {
                let token = self.previous();

                return Err(Error {
                    line: token.line,
                    column: token.column + token.len(),
                    source: ErrorType::UnclosedParen,
                });
            }

            return Ok(Expression::GroupingExpression(expression));
        }

        let token = match self.peek().kind {
            TokenKind::Eof => self.previous(),
            _ => self.peek(),
        };
        Err(Error {
            line: token.line,
            column: token.column + token.len(),
            source: ErrorType::ExpectedExpression,
        })
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
                TokenKind::Print,
                TokenKind::Var,
            ) {
                return;
            }

            self.next();
        }
    }

    fn peek(&mut self) -> &Token {
        &self.tokens[self.current]
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }

    fn next(&mut self) -> &Token {
        if !self.is_done() {
            self.current += 1;
        }

        &self.tokens[self.current - 1]
    }

    fn is_done(&mut self) -> bool {
        self.peek().kind == TokenKind::Eof
    }
}
