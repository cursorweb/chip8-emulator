use crate::{
    asmparser::ParseError,
    octo::{ast::Stmt, token::TType},
};

use super::token::Token;

type StmtResult = Result<Stmt, ParseError>;

pub struct Parser {
    tokens: Vec<Token>,
    i: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, i: 0 }
    }

    pub fn parse(mut self) -> Result<Vec<Stmt>, ParseError> {
        let mut out = Vec::new();

        while self.is_valid() {
            out.push(self.stmt()?);
        }

        Ok(out)
    }

    fn stmt(&mut self) -> StmtResult {
        if self.consume(TType::Return) {
            return Ok(Stmt::Return);
        }

        todo!()
    }

    fn consume(&mut self, ttype: TType) -> bool {
        if self.peek().ttype == ttype {
            self.advance();
            true
        } else {
            false
        }
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.i]
    }

    fn advance(&mut self) -> &Token {
        if self.is_valid() {
            self.i += 1;
        }

        // return EOF if is at end
        self.peek()
    }

    fn is_valid(&self) -> bool {
        // also can do self.peek().ttype == EOF
        self.i < self.tokens.len() - 1
    }
}
