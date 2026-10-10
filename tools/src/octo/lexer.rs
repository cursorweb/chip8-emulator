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
    pub fn lex(mut self) -> Result<Vec<Token>, ParseError> {
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

        if self.peek().is_some_and(|x| x.is_ascii_digit()) {
            return Ok(self.number(false)?);
        } else if self.peek().is_some_and(|x| x.is_ascii_alphabetic()) {
            return Ok(self.parse_ident());
        }

        let Some(c) = self.advance() else {
            return Ok(Token::Eof);
        };

        Ok(match c {
            // punctuation
            ';' => Token::Semi,
            ':' => {
                if self.consume('=') {
                    Token::AssignEq
                } else if self.peek().is_some_and(|x| x.is_ascii_alphabetic()) {
                    let name = self.identifier();
                    match name.as_str() {
                        "alias" => Token::Alias,
                        _ => return Err(self.error(format!("Unknown directive {name}"))),
                    }
                } else {
                    Token::Colon
                }
            }

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
                if self.consume('=') {
                    Token::MinusEq
                } else if self.peek().is_some_and(|x| x.is_ascii_digit()) {
                    self.number(true)?
                } else {
                    Token::Minus
                }
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

    fn consume(&mut self, c: char) -> bool {
        if self.peek() == Some(c) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn expect(&mut self, c: char) -> Result<(), ParseError> {
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
            } else if c == '#' {
                while self.peek().is_some_and(|x| x != '\n') {
                    self.advance();
                }
            } else {
                break;
            }
        }
    }

    fn parse_ident(&mut self) -> Token {
        let ident = self.identifier();
        match ident.as_str() {
            "clear" | "cls" => Token::Clear,
            "return" => Token::Return,
            "bcd" => Token::Bcd,
            "save" => Token::Save,
            "load" => Token::Load,
            "sprite" | "draw" => Token::Sprite,
            "jump" => Token::Jump,
            "jump0" => Token::Jump0,
            "if" => Token::If,
            "then" => Token::Then,
            "else" => Token::Else,
            "begin" => Token::Begin,
            "end" => Token::End,
            "loop" => Token::Loop,
            "again" => Token::Again,
            "random" => Token::Random,
            "key" => Token::Key,
            "i" => Token::I,
            "buzzer" => Token::Buzzer,
            "delay" => Token::Delay,
            v => match v
                .to_lowercase()
                .strip_prefix('v')
                .and_then(|n| u8::from_str_radix(n, 16).ok())
            {
                Some(n @ 0..=15) => Token::V(n),
                _ => Token::Ident(ident),
            },
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

    // Parses u16 number
    fn number(&mut self, negative: bool) -> Result<Token, ParseError> {
        let start = self.pos;

        let (base, digit_start) = if self.consume('0') {
            match self.peek() {
                Some('x') | Some('X') => {
                    self.advance();
                    (16, self.pos)
                }
                Some('b') | Some('B') => {
                    self.advance();
                    (2, self.pos)
                }
                // 01 -> 1, no problem!
                _ => (10, self.pos - 1),
            }
        } else {
            (10, self.pos)
        };

        while let Some(c) = self.peek() {
            if c.is_digit(base) {
                self.advance();
            } else {
                break;
            }
        }

        if self.pos == digit_start {
            return Err(ParseError {
                pos: start,
                message: "Expected digits".into(),
            });
        }

        let text = &self.input[digit_start..self.pos];

        let number = u16::from_str_radix(text, base)
            .map_err(|_| self.error("Number does not fit in u16"))?;
        if negative {
            if number > u8::MAX as u16 {
                Err(self.error("Negative number would overflow"))
            } else {
                Ok(Token::Number(256 - number))
            }
        } else {
            Ok(Token::Number(number))
        }
    }

    fn error(&mut self, message: impl Into<String>) -> ParseError {
        ParseError {
            pos: self.pos,
            message: message.into(),
        }
    }
}
