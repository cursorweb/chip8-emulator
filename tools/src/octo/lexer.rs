use super::token::Token;
use crate::asmparser::ParseError;

pub struct Lexer<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self { input, pos: 0 }
    }
    pub fn tokenize(mut self) -> Result<Vec<Token>, ParseError> {
        let mut tokens = Vec::new();

        loop {
            let token = self.next_token()?;
            let done = matches!(token, Token::Eof);

            tokens.push(token);

            if done {
                break;
            }
        }

        Ok(tokens)
    }

    pub fn next_token(&mut self) -> Result<Token, ParseError> {
        self.skip_whitespace();

        let Some(c) = self.advance() else {
            return Ok(Token::Eof);
        };

        Ok(match c {
            // punctuation
            ';' => Token::Semi,
            ':' => Token::Colon,

            // operators
            '=' => {
                if self.consume('=') {
                    Token::EqEq
                } else {
                    Token::AssignEq
                }
            }
            '+' => {
                self.expect('=')?;
                Token::PlusEq
            }
            '-' => {
                self.expect('=')?;
                Token::MinusEq
            }
            '!' => {
                self.expect('=')?;
                Token::NotEq
            }
            '<' => {
                if self.consume('=') {
                    Token::LessEq
                } else if self.consume('<') {
                    self.expect('=')?;
                    Token::ShlEq
                } else {
                    Token::Less
                }
            }
            '>' => {
                if self.consume('=') {
                    Token::GreaterEq
                } else if self.consume('>') {
                    self.expect('=')?;
                    Token::ShrEq
                } else {
                    Token::Greater
                }
            }
            '|' => {
                self.expect('=')?;
                Token::OrEq
            }
            '&' => {
                self.expect('=')?;
                Token::AndEq
            }
            '^' => {
                self.expect('=')?;
                Token::XorEq
            }

            // TODO: numbers
            // TODO: identifiers / keywords
            _ => return Err(self.error(format!("Unexpected character '{c}'"))),
        })
    }

    fn peek(&mut self) -> Option<char> {
        self.input[self.pos..].chars().next()
    }

    fn advance(&mut self) -> Option<char> {
        let c = self.peek()?;
        self.pos += c.len_utf8();
        Some(c)
    }

    fn consume(&mut self, expected: char) -> bool {
        if self.peek() == Some(expected) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn expect(&mut self, c: char) -> Result<(), ParseError> {
        self.skip_whitespace();
        let p = self
            .peek()
            .ok_or_else(|| self.error(format!("expected `{c}`, got EOF")))?;

        if p != c {
            return Err(self.error(format!("expected `{c}`, got `{p}`")));
        }

        self.advance();
        Ok(())
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            if c.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn identifier(&mut self) -> String {
        let mut identifier = String::new();

        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || c == '_' || c == '-' {
                identifier.push(c);
                self.advance();
            } else {
                break;
            }
        }

        identifier
    }

    fn number(&mut self) -> String {
        let mut number = String::new();

        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                number.push(c);
                self.advance();
            } else {
                break;
            }
        }

        number
    }

    fn error(&mut self, message: impl Into<String>) -> ParseError {
        ParseError {
            pos: self.pos,
            message: message.into(),
        }
    }
}
