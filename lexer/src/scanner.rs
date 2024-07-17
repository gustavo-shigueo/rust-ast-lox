use std::{iter::Peekable, str::Bytes};

use crate::{
    token::{Token, TokenKind},
    Error, ErrorType, Result,
};

#[derive(Debug)]
pub struct Scanner<'a> {
    source: &'a str,
    bytes: Peekable<Bytes<'a>>,
    line: u32,
    column: u32,
    current: usize,
    lexeme_start: usize,
    tokens: Vec<Token>,
    errors: Vec<Error>,
}

impl<'a> Scanner<'a> {
    #[must_use]
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            bytes: source.bytes().peekable(),
            line: 0,
            column: 0,
            current: 0,
            lexeme_start: 0,
            tokens: vec![],
            errors: vec![],
        }
    }

    /// This function will consume all the characters in the source code
    /// and returns a collection of tokens
    ///
    /// # Errors
    /// If there is an unexpected character or an unterminated string,
    /// a collection will be returned containing all errors found
    pub fn scan(mut self) -> Result<Vec<Token>, Vec<Error>> {
        while self.peek().is_some() {
            self.lexeme_start = self.current;

            match self.scan_token() {
                Ok(Some(token)) => self.tokens.push(token),
                Ok(None) => (),
                Err(error) => self.errors.push(error),
            }
        }

        self.tokens.push(Token {
            line: self.line,
            column: self.column - 1,
            kind: TokenKind::Eof,
        });

        if self.errors.is_empty() {
            Ok(self.tokens)
        } else {
            Err(self.errors)
        }
    }

    fn scan_token(&mut self) -> Result<Option<Token>> {
        let character = self.next();

        Ok(Some(match character {
            token @ (b'(' | b')' | b'[' | b']' | b'{' | b'}' | b';' | b',' | b'.' | b'-' | b'+'
            | b'*') => Token {
                line: self.line,
                column: self.column - 1,
                kind: match token {
                    b'(' => TokenKind::LeftParen,
                    b')' => TokenKind::RightParen,
                    b'[' => TokenKind::LeftBracket,
                    b']' => TokenKind::RightBracket,
                    b'{' => TokenKind::LeftCurly,
                    b'}' => TokenKind::RightCurly,
                    b';' => TokenKind::Semicolon,
                    b',' => TokenKind::Comma,
                    b'.' => TokenKind::Dot,
                    b'+' => TokenKind::Plus,
                    b'-' => TokenKind::Minus,
                    b'*' => TokenKind::Star,
                    _ => unreachable!(),
                },
            },
            character @ (b'<' | b'>' | b'!' | b'=') => {
                let is_followed_by_equal = self.match_next(b'=');

                Token {
                    line: self.line,
                    column: self.column - 1,
                    kind: match character {
                        b'<' if is_followed_by_equal => TokenKind::LessEqual,
                        b'<' => TokenKind::LessThan,
                        b'>' if is_followed_by_equal => TokenKind::GreaterEqual,
                        b'>' => TokenKind::GreaterThan,
                        b'!' if is_followed_by_equal => TokenKind::BangEqual,
                        b'!' => TokenKind::Bang,
                        b'=' if is_followed_by_equal => TokenKind::DoubleEquals,
                        b'=' => TokenKind::Equals,
                        _ => unreachable!(),
                    },
                }
            }
            b'/' => {
                if self.match_next(b'/') {
                    self.scan_comment();
                    return Ok(None);
                }

                Token {
                    line: self.line,
                    column: self.column - 1,
                    kind: TokenKind::Slash,
                }
            }
            b'"' => self.scan_string_literal()?,
            b'0'..=b'9' => self.scan_number_literal(),
            b'a'..=b'z' | b'A'..=b'Z' | b'_' => self.scan_identifier(),
            b' ' | b'\t' | b'\r' => return Ok(None),
            b'\n' => {
                self.line += 1;
                self.column = 0;
                return Ok(None);
            }
            x => {
                return Err(Error {
                    line: self.line,
                    column: self.column - 1,
                    source: ErrorType::UnexpectedCharacter(x.into()),
                })
            }
        }))
    }

    fn scan_comment(&mut self) {
        while self.peek().is_some_and(|x| x != b'\n') {
            self.next();
        }

        // Only increase line count if not at EOF
        if self.peek().is_some() {
            self.column = 0;
            self.line += 1;
        }
    }

    fn scan_string_literal(&mut self) -> Result<Token> {
        let line = self.line;
        let column = self.column - 1;

        while let Some(c) = self.peek() {
            if c == b'"' {
                break;
            }

            if c == b'\n' {
                self.line += 1;
                self.column = 0;
            }

            self.next();
        }

        // Hit EOF without terminating string
        if self.peek().is_none() {
            return Err(Error {
                line,
                column,
                source: ErrorType::UnterminatedString,
            });
        }

        // Consume the closing double quotes
        self.next();

        let value = &self.source[self.lexeme_start + 1..self.current - 1];
        Ok(Token {
            line,
            column,
            kind: TokenKind::String(value.to_owned()),
        })
    }

    fn scan_number_literal(&mut self) -> Token {
        let line = self.line;
        let column = self.column - 1;

        while let Some(b'0'..=b'9') = self.peek() {
            self.next();
        }

        if self.match_next(b'.') && self.double_peek().is_some_and(|x| x.is_ascii_digit()) {
            self.next();

            while let Some(b'0'..=b'9') = self.peek() {
                self.next();
            }
        }

        Token {
            line,
            column,
            kind: TokenKind::Number(
                self.source[self.lexeme_start..self.current]
                    .parse()
                    .expect("Invalid numeric literal"),
            ),
        }
    }

    fn scan_identifier(&mut self) -> Token {
        let line = self.line;
        let column = self.column - 1;

        while let Some(b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_') = self.peek() {
            self.next();
        }

        let text = &self.source[self.lexeme_start..self.current];

        Token {
            line,
            column,
            kind: match text {
                "if" => TokenKind::If,
                "else" => TokenKind::Else,
                "for" => TokenKind::For,
                "while" => TokenKind::While,
                "var" => TokenKind::Var,
                "fun" => TokenKind::Fun,
                "return" => TokenKind::Return,
                "class" => TokenKind::Class,
                "this" => TokenKind::This,
                "super" => TokenKind::Super,
                "print" => TokenKind::Print,
                "nil" => TokenKind::Nil,
                "true" => TokenKind::True,
                "false" => TokenKind::False,
                "or" => TokenKind::Or,
                "and" => TokenKind::And,
                ident => TokenKind::Identifier(ident.to_owned()),
            },
        }
    }

    /// Checks if the next character is equal to the expected value,
    /// consuming it if it does
    fn match_next(&mut self, expected: u8) -> bool {
        match self.peek() {
            Some(x) if x == expected => {
                self.next();
                true
            }
            _ => false,
        }
    }

    fn next(&mut self) -> u8 {
        let c = self.bytes.next();
        self.current += 1;
        self.column += 1;

        c.expect("Unexpected EOF")
    }

    fn peek(&mut self) -> Option<u8> {
        self.bytes.peek().copied()
    }

    fn double_peek(&self) -> Option<u8> {
        self.source.as_bytes().get(self.current + 1).copied()
    }
}
