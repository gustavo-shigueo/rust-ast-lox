use std::{iter::Peekable, str::Chars};

use crate::{
    token::{Token, TokenType},
    Error, ErrorType, Result,
};

#[derive(Debug)]
pub struct Scanner<'a> {
    source: &'a str,
    chars: Peekable<Chars<'a>>,
    line: u32,
    column: u32,
    current: usize,
    lexeme_start: usize,
    tokens: Vec<Token>,
    errors: Vec<Error>,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            chars: source.chars().peekable(),
            line: 0,
            column: 0,
            current: 0,
            lexeme_start: 0,
            tokens: vec![],
            errors: vec![],
        }
    }

    pub fn scan(mut self) -> Result<Vec<Token>, Vec<Error>> {
        while !self.is_done() {
            self.lexeme_start = self.current;

            match self.scan_token() {
                Ok(Some(token)) => self.tokens.push(token),
                Ok(None) => (),
                Err(error) => self.errors.push(error),
            }
        }

        if self.errors.is_empty() {
            Ok(self.tokens)
        } else {
            Err(self.errors)
        }
    }

    fn scan_token(&mut self) -> Result<Option<Token>> {
        let character = self.advance();

        Ok(Some(match character {
            token @ ('(' | ')' | '[' | ']' | '{' | '}' | ';' | ',' | '.' | '-' | '+' | '*') => {
                Token {
                    line: self.line,
                    column: self.column - 1,
                    token_type: match token {
                        '(' => TokenType::LeftParen,
                        ')' => TokenType::RightParen,
                        '[' => TokenType::LeftBracket,
                        ']' => TokenType::RightBracket,
                        '{' => TokenType::LeftCurly,
                        '}' => TokenType::RightCurly,
                        ';' => TokenType::Semicolon,
                        ',' => TokenType::Comma,
                        '.' => TokenType::Dot,
                        '+' => TokenType::Plus,
                        '-' => TokenType::Minus,
                        '*' => TokenType::Star,
                        _ => unreachable!(),
                    },
                }
            }
            character @ ('<' | '>' | '!' | '=') => {
                let is_followed_by_equal = self.match_next('=');

                Token {
                    line: self.line,
                    column: self.column - 1,
                    token_type: match character {
                        '<' if is_followed_by_equal => TokenType::LessEqual,
                        '<' => TokenType::LessThan,
                        '>' if is_followed_by_equal => TokenType::GreaterEqual,
                        '>' => TokenType::GreaterThan,
                        '!' if is_followed_by_equal => TokenType::BangEqual,
                        '!' => TokenType::Bang,
                        '=' if is_followed_by_equal => TokenType::DoubleEquals,
                        '=' => TokenType::Equals,
                        _ => unreachable!(),
                    },
                }
            }
            '/' => {
                if self.match_next('/') {
                    while let Some(&x) = self.chars.peek() {
                        if x != '\n' {
                            self.advance();
                        } else {
                            self.column = 0;
                            self.line += 1;
                            break;
                        }
                    }
                    return Ok(None);
                } else {
                    Token {
                        line: self.line,
                        column: self.column - 1,
                        token_type: TokenType::Slash,
                    }
                }
            }
            '"' => {
                let line = self.line;
                let column = self.column - 1;

                while let Some(&c) = self.chars.peek() {
                    if c == '"' || self.is_done() {
                        break;
                    }

                    if c == '\n' {
                        self.line += 1;
                        self.column = 0;
                    }

                    self.advance();
                }

                if self.is_done() {
                    return Err(Error {
                        line,
                        column,
                        source: ErrorType::UnterminatedString,
                    });
                }

                self.advance();

                let value = &self.source[self.lexeme_start + 1..self.current - 1];
                Token {
                    line,
                    column,
                    token_type: TokenType::String(value.to_owned()),
                }
            }
            '0'..='9' => {
                let line = self.line;
                let column = self.column - 1;

                while let Some('0'..='9') = self.chars.peek() {
                    self.advance();
                }

                if self.chars.peek().is_some_and(|&x| x == '.')
                    && self.peek_next().is_some_and(|x| x.is_ascii_digit())
                {
                    self.advance();

                    while let Some('0'..='9') = self.chars.peek() {
                        self.advance();
                    }
                }

                Token {
                    line,
                    column,
                    token_type: TokenType::Number(
                        self.source[self.lexeme_start..self.current]
                            .parse()
                            .expect("Invalid numeric literal"),
                    ),
                }
            }
            'a'..='z' | 'A'..='Z' | '_' => {
                let line = self.line;
                let column = self.column - 1;

                while let Some('a'..='z' | 'A'..='Z' | '0'..='9' | '_') = self.chars.peek() {
                    self.advance();
                }

                let text = &self.source[self.lexeme_start..self.current];

                Token {
                    line,
                    column,
                    token_type: match text {
                        "if" => TokenType::If,
                        "else" => TokenType::Else,
                        "for" => TokenType::For,
                        "while" => TokenType::While,
                        "var" => TokenType::Var,
                        "fun" => TokenType::Fun,
                        "return" => TokenType::Return,
                        "class" => TokenType::Class,
                        "this" => TokenType::This,
                        "super" => TokenType::Super,
                        "print" => TokenType::Print,
                        "nil" => TokenType::Nil,
                        "true" => TokenType::True,
                        "false" => TokenType::False,
                        "or" => TokenType::Or,
                        "and" => TokenType::And,
                        ident => TokenType::Identifier(ident.to_owned()),
                    },
                }
            }
            ' ' | '\t' | '\r' => return Ok(None),
            '\n' => {
                self.line += 1;
                self.column = 0;
                return Ok(None);
            }
            x => {
                return Err(Error {
                    line: self.line,
                    column: self.column - 1,
                    source: ErrorType::UnexpectedCharacter(x),
                })
            }
        }))
    }

    fn match_next(&mut self, expected: char) -> bool {
        if self.is_done() {
            return false;
        }

        match self.chars.peek() {
            Some(&x) if x == expected => {
                self.chars.next();
                self.column += 1;
                self.current += 1;

                true
            }
            _ => false,
        }
    }

    fn advance(&mut self) -> char {
        let c = self.chars.next();
        self.current += 1;
        self.column += 1;

        c.expect("Unexpected EOF")
    }

    fn is_done(&self) -> bool {
        self.current >= self.source.len()
    }

    fn peek_next(&self) -> Option<char> {
        self.source.chars().nth(self.current + 1)
    }
}
