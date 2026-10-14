use crate::{
    asmparser::ParseError,
    octo::{
        ast::{Num, Reg, Stmt},
        token::TType,
    },
};

use super::token::Token;

type StmtResult = Result<Stmt, ParseError>;

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    pub fn parse(mut self) -> Result<Vec<Stmt>, ParseError> {
        let mut out = Vec::new();

        while self.is_valid() {
            out.push(self.stmt()?);
        }

        Ok(out)
    }

    fn stmt(&mut self) -> StmtResult {
        Ok(if self.consume(TType::Return) {
            Stmt::Return
        } else if self.consume(TType::Clear) {
            Stmt::Clear
        } else if self.consume(TType::Again) {
            Stmt::Again
        } else if self.consumes(&[TType::Bcd, TType::Save, TType::Load]) {
            Stmt::UnaryStmt(self.prev().ttype, self.reg()?)
        } else if self.consume(TType::Sprite) {
            Stmt::Sprite(self.reg()?, self.reg()?, self.num()?)
        } else if self.consumes(&[TType::Jump, TType::Jump0]) {
            Stmt::JumpStmt(self.prev().ttype, self.num()?)
        } else {
            todo!()
        })
    }

    fn reg(&mut self) -> Result<Reg, ParseError> {
        match &self.peek().ttype {
            TType::Register(v) => {
                let out = Reg::Val(*v);
                self.advance();
                Ok(out)
            }
            TType::Ident(x) => {
                let out = Reg::Ident(x.clone());
                self.advance();
                Ok(out)
            }
            _ => Err(self.error("Expected register")),
        }
    }

    fn num(&mut self) -> Result<Num, ParseError> {
        match &self.peek().ttype {
            TType::Number(v) => {
                let out = Num::Val(*v);
                self.advance();
                Ok(out)
            }
            TType::Ident(x) => {
                let out = Num::Ident(x.clone());
                self.advance();
                Ok(out)
            }
            _ => Err(self.error("Expected number")),
        }
    }

    fn consumes(&mut self, ttype: &[TType]) -> bool {
        for t in ttype {
            // acts like copy
            if self.consume(t.clone()) {
                return true;
            }
        }

        false
    }

    fn consume(&mut self, ttype: TType) -> bool {
        if self.peek().ttype == ttype {
            self.advance();
            true
        } else {
            false
        }
    }

    /// Assume that you use prev
    fn prev(&self) -> Token {
        self.tokens[self.pos - 1].clone()
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.pos]
    }

    fn advance(&mut self) -> &Token {
        if self.is_valid() {
            self.pos += 1;
        }

        // return EOF if is at end
        self.peek()
    }

    fn is_valid(&self) -> bool {
        // also can do self.peek().ttype == EOF
        self.pos < self.tokens.len() - 1
    }

    fn error(&self, message: impl Into<String>) -> ParseError {
        ParseError {
            pos: self.peek().pos,
            message: message.into(),
        }
    }
}
