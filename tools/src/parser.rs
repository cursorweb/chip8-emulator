use crate::instr::{Addr, Instr};

#[derive(Debug, PartialEq)]
pub struct ParseError {
    pub pos: usize,
    pub message: String,
}

impl ParseError {
    pub fn show(&self, source: &str) {
        let before = &source[..self.pos];

        let line_num = before.matches('\n').count() + 1;

        let line_start = before.rfind('\n').map_or(0, |i| i + 1);
        let line_end = source[self.pos..]
            .find('\n')
            .map_or(source.len(), |i| self.pos + i);

        let line = &source[line_start..line_end];
        let column = self.pos - line_start;

        eprintln!("Line {line_num}, error: {}", self.message);
        eprintln!("{line}");
        eprintln!("{}^", " ".repeat(column));
    }
}

pub struct Parser<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Self {
        Self { input, pos: 0 }
    }

    pub fn parse(mut self) -> Result<Vec<Instr>, ParseError> {
        let mut instrs = Vec::new();

        while !self.eof() {
            self.skip_whitespace();

            if self.eof() {
                break;
            }

            if self.peek() == Some('#') {
                self.skip_comment();
                continue;
            }

            instrs.push(self.parse_instr()?);
        }

        Ok(instrs)
    }

    fn parse_instr(&mut self) -> Result<Instr, ParseError> {
        if self.consume('.') {
            let ident = self.ident()?;
            return match ident.as_str() {
                "word" => Ok(Instr::Word(self.word()?)),
                "byte" => Ok(Instr::Byte(self.byte()?)),
                _ => Err(self.error(format!("Unknown directive {ident}"))),
            };
        }

        let ident = self.ident()?;

        match ident.as_str() {
            "cls" => Ok(Instr::Cls),
            "ret" => Ok(Instr::Ret),

            "j" => Ok(Instr::J(self.addr()?)),
            "call" => Ok(Instr::Call(self.addr()?)),
            "jri0" => Ok(Instr::Jri0(self.number()?)),
            "jri" => Ok(Instr::Jri(self.register()?, self.comma_then(Self::byte)?)),

            "seqi" => Ok(Instr::Seqi(self.register()?, self.comma_then(Self::byte)?)),
            "snei" => Ok(Instr::Snei(self.register()?, self.comma_then(Self::byte)?)),
            "seq" => Ok(Instr::Seq(
                self.register()?,
                self.comma_then(Self::register)?,
            )),
            "sne" => Ok(Instr::Sne(
                self.register()?,
                self.comma_then(Self::register)?,
            )),
            "sk" => Ok(Instr::Sk(self.register()?)),
            "snk" => Ok(Instr::Snk(self.register()?)),

            "seti" => self.parse_seti(),
            "addi" => Ok(Instr::Addi(self.register()?, self.comma_then(Self::byte)?)),

            "set" => self.parse_set(),
            "or" => Ok(Instr::Or(
                self.register()?,
                self.comma_then(Self::register)?,
            )),
            "and" => Ok(Instr::And(
                self.register()?,
                self.comma_then(Self::register)?,
            )),
            "xor" => Ok(Instr::Xor(
                self.register()?,
                self.comma_then(Self::register)?,
            )),
            "add" => self.parse_add(),
            "subf" => Ok(Instr::Subf(
                self.register()?,
                self.comma_then(Self::register)?,
            )),
            "srlf" => self.parse_shift(true),
            "subnf" => Ok(Instr::Subnf(
                self.register()?,
                self.comma_then(Self::register)?,
            )),
            "sllf" => self.parse_shift(false),

            "rand" => Ok(Instr::Rand(self.register()?, self.comma_then(Self::byte)?)),
            "sprite" => Ok(Instr::Sprite(
                self.register()?,
                self.comma_then(Self::register)?,
                self.comma_then(Self::nibble)?,
            )),
            "bcd" => Ok(Instr::Bcd(self.register()?)),

            _ if self.peek() == Some(':') => {
                self.advance();
                Ok(Instr::Label(ident))
            }

            _ => Err(self.error(format!("unknown instruction `{ident}`"))),
        }
    }

    /// Parse `seti $x, NN` and `seti I, 0xNNN`
    fn parse_seti(&mut self) -> Result<Instr, ParseError> {
        if self.peek_is('I') {
            self.advance();
            self.comma()?;
            return Ok(Instr::SetAddrI(self.addr()?));
        }

        let x = self.register()?;
        self.comma()?;
        let byte = self.byte()?;

        Ok(Instr::Seti(x, byte))
    }

    fn parse_set(&mut self) -> Result<Instr, ParseError> {
        // cases:
        // set $x, DELAY
        // set $x, KEY
        // set $x, [I]
        // set $x, $y
        // set DELAY, $x
        // set SOUND, $x
        // set I, FONT[$x]
        // set [I], $x
        if self.peek_is('$') {
            // cases:
            // set $x, [I]
            // set $x, DELAY
            // set $x, KEY
            // set $x, $y
            let x = self.register()?;
            self.comma()?;

            if self.peek_is('$') {
                // set $x, $y
                let y = self.register()?;
                Ok(Instr::Set(x, y))
            } else if self.consume('[') {
                // set $x, [I]
                self.expect('I')?;
                self.expect(']')?;
                Ok(Instr::GetMemI(x))
            } else {
                // set $x, DELAY
                // set $x, KEY
                let prop = self.ident()?;
                Ok(match prop.as_str() {
                    "DELAY" => Instr::GetDelay(x),
                    "KEY" => Instr::GetKey(x),
                    _ => {
                        return Err(
                            self.error(format!("Expected either DELAY or KEY but got {prop}"))
                        );
                    }
                })
            }
        } else {
            // cases:
            // set DELAY, $x
            // set SOUND, $x
            // set I, FONT[$x]
            // set [I], $x

            if self.consume('[') {
                self.expect('I')?;
                self.expect(']')?;
                self.comma()?;
                let x = self.register()?;
                return Ok(Instr::SetMemI(x));
            }

            let prop = self.ident()?;
            self.comma()?;
            if prop == "I" {
                self.expect_ident("FONT")?;
                self.expect('[')?;
                let x = self.register()?;
                self.expect(']')?;
                return Ok(Instr::SetFont(x));
            }
            let x = self.register()?;
            Ok(match prop.as_str() {
                "DELAY" => Instr::SetDelay(x),
                "SOUND" => Instr::SetSound(x),
                _ => {
                    return Err(
                        self.error(format!("Expected either DELAY or SOUND but got {prop}"))
                    );
                }
            })
        }
    }

    fn parse_add(&mut self) -> Result<Instr, ParseError> {
        // cases:
        // add $x, $y
        // add I, $x
        if self.peek_is('$') {
            // add $x, $y
            let x = self.register()?;
            self.comma()?;
            let y = self.register()?;
            Ok(Instr::Add(x, y))
        } else {
            // add I, $x
            self.expect('I')?;
            self.comma()?;
            let x = self.register()?;
            Ok(Instr::AddMemI(x))
        }
    }

    /// `srlf $x` or `srlf $x, $y`
    /// The one-operand form is normalized to `$y = 0`.
    fn parse_shift(&mut self, right: bool) -> Result<Instr, ParseError> {
        let x = self.register()?;

        if self.comma().is_ok() {
            let y = self.register()?;

            return if right {
                Ok(Instr::Srlf(x, y))
            } else {
                Ok(Instr::Sllf(x, y))
            };
        }

        if right {
            Ok(Instr::Srlf(x, 0))
        } else {
            Ok(Instr::Sllf(x, 0))
        }
    }

    fn register(&mut self) -> Result<u8, ParseError> {
        self.expect('$')?;

        let c = self
            .advance()
            .ok_or_else(|| self.error("Expected register".into()))?;

        let n = c
            .to_digit(16)
            .ok_or_else(|| self.error(format!("Expected hex digit after `$`, got `{c}`")))?;

        if n > 0xF {
            return Err(self.error("Register must be 0-F".into()));
        }

        Ok(n as u8)
    }

    fn addr(&mut self) -> Result<Addr, ParseError> {
        self.skip_whitespace();
        if let Some(c) = self.peek() {
            Ok(if c.is_ascii_alphabetic() {
                Addr::Label(self.ident()?)
            } else {
                Addr::Val(self.address()?)
            })
        } else {
            Err(self.error("Expected address, but got EOF".into()))
        }
    }

    /// Support 0x, 0b, and decimal
    fn number(&mut self) -> Result<u16, ParseError> {
        self.skip_whitespace();
        let start = self.pos;

        let base = if self.consume('0') {
            match self.peek() {
                Some('x') | Some('X') => {
                    self.advance();
                    16
                }
                Some('b') | Some('B') => {
                    self.advance();
                    2
                }
                // also consume 0, so 01 -> 1 is okay!
                _ => 10,
            }
        } else {
            10
        };

        let digit_start = self.pos;

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
                message: "expected digits".into(),
            });
        }

        let text = &self.input[digit_start..self.pos];

        u16::from_str_radix(text, base).map_err(|_| self.error("Number does not fit in u16".into()))
    }

    fn byte(&mut self) -> Result<u8, ParseError> {
        let n = self.number()?;

        if n > 0xFF {
            return Err(self.error(format!("Value {n:#X} does not fit in a byte")));
        }

        Ok(n as u8)
    }

    fn nibble(&mut self) -> Result<u8, ParseError> {
        let n = self.number()?;

        if n > 0xF {
            return Err(self.error(format!("Value {n:#X} does not fit in a nibble")));
        }

        Ok(n as u8)
    }

    fn address(&mut self) -> Result<u16, ParseError> {
        let n = self.number()?;

        if n > 0xFFF {
            return Err(self.error(format!("Address {n:#X} does not fit in 12 bits")));
        }

        Ok(n)
    }

    fn word(&mut self) -> Result<u16, ParseError> {
        self.number()
    }

    fn comma(&mut self) -> Result<(), ParseError> {
        self.skip_whitespace();
        self.expect(',')?;
        Ok(())
    }

    fn skip_comment(&mut self) {
        while let Some(c) = self.peek() {
            self.advance();

            if c == '\n' {
                break;
            }
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            if !c.is_whitespace() {
                break;
            }

            self.advance();
        }
    }

    fn peek_is(&mut self, c: char) -> bool {
        self.skip_whitespace();
        self.peek().map_or(false, |p| p == c)
    }

    fn peek(&self) -> Option<char> {
        self.input[self.pos..].chars().next()
    }

    fn advance(&mut self) -> Option<char> {
        let c = self.peek()?;
        self.pos += c.len_utf8();
        Some(c)
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

    fn consume(&mut self, c: char) -> bool {
        if let Some(p) = self.peek() {
            if p == c {
                self.advance();
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    fn expect_ident(&mut self, x: &str) -> Result<(), ParseError> {
        let p = self.ident()?;
        if p == x {
            Ok(())
        } else {
            Err(self.error(format!("Expected {x} but got {p}")))
        }
    }

    fn ident(&mut self) -> Result<String, ParseError> {
        self.skip_whitespace();

        let c = self
            .peek()
            .ok_or_else(|| self.error("expected identifier".into()))?;

        if !is_ident_start(c) {
            return Err(self.error(format!("expected identifier, got `{c}`")));
        }

        let mut result = String::new();
        result.push(c);
        self.advance();

        while let Some(c) = self.peek() {
            if !is_ident_continue(c) {
                break;
            }

            result.push(c);
            self.advance();
        }

        Ok(result)
    }

    fn comma_then<T>(
        &mut self,
        f: impl FnOnce(&mut Self) -> Result<T, ParseError>,
    ) -> Result<T, ParseError> {
        self.comma()?;
        f(self)
    }

    fn eof(&self) -> bool {
        self.pos >= self.input.len()
    }

    fn error(&self, message: String) -> ParseError {
        ParseError {
            pos: self.pos,
            message,
        }
    }
}

fn is_ident_start(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_'
}

fn is_ident_continue(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}
