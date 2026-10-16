use crate::{
    asmparser::ParseError,
    octo::{
        ast::{Decl, Stmt, Val},
        token::TType,
    },
};

use super::token::Token;

type DeclResult = Result<Decl, ParseError>;
type StmtResult = Result<Stmt, ParseError>;

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    pub fn parse(mut self) -> Result<Vec<Decl>, ParseError> {
        let mut out = Vec::new();

        while self.is_valid() {
            out.push(self.decl()?);
        }

        Ok(out)
    }

    fn decl(&mut self) -> DeclResult {
        Ok(if self.consume(TType::Alias) {
            match self.advance().ttype {
                TType::Ident(s) => Decl::Alias(s, self.val()?),
                _ => return Err(self.error(format!("Expected name but got {}", self.peek().ttype))),
            }
        } else if self.consume(TType::Const) {
            match self.advance().ttype {
                TType::Ident(s) => Decl::Const(s.clone(), self.val()?),
                _ => return Err(self.error(format!("Expected name but got {}", self.peek().ttype))),
            }
        } else {
            Decl::Stmt(self.stmt()?)
        })
    }

    fn stmt(&mut self) -> StmtResult {
        Ok(if self.consume(TType::Return) {
            Stmt::Return
        } else if self.consume(TType::Clear) {
            Stmt::Clear
        } else if self.consume(TType::Again) {
            Stmt::Again
        } else if self.consumes(&[TType::Bcd, TType::Save, TType::Load]) {
            Stmt::UnaryStmt(self.prev().ttype, self.val()?)
        } else if self.consume(TType::Sprite) {
            Stmt::Sprite(self.val()?, self.val()?, self.val()?)
        } else if self.consumes(&[TType::Jump, TType::Jump0]) {
            Stmt::JumpStmt(self.prev().ttype, self.val()?)
        } else if self.consume(TType::If) {
            self.if_stmt()?
        } else {
            println!("{}", self.peek().ttype);
            todo!()
        })
    }

    fn if_stmt(&mut self) -> StmtResult {
        let comp = self.comparison()?;
        self.expect(TType::Then)?;
        let stmt = self.stmt()?;
        Ok(Stmt::IfStmt(Box::new(comp), Box::new(stmt)))
    }

    fn comparison(&mut self) -> StmtResult {
        let reg = self.val()?;
        Ok(
            if self.consumes(&[
                TType::EqEq,
                TType::NotEq,
                TType::Greater,
                TType::Less,
                TType::GreaterEq,
                TType::LessEq,
            ]) {
                let tok = self.prev();
                Stmt::Binary(reg, tok.ttype, self.val()?)
            } else if self.consume(TType::Key) {
                Stmt::RegKey(reg)
            } else if self.consume(TType::Minus) {
                self.expect(TType::Key)?;
                Stmt::RegNotKey(reg)
            } else {
                return Err(self.error(format!("Unexpcted token {}", self.peek().ttype)));
            },
        )
    }

    /// Is it a register or a number?
    fn val(&mut self) -> Result<Val, ParseError> {
        Ok(match self.advance().ttype {
            TType::Register(r) => Val::Reg(r),
            TType::Number(n) => Val::Num(n),
            TType::Ident(i) => Val::Ident(i),
            t => return Err(self.error(format!("Expected register but got {t}"))),
        })
    }

    fn expect(&mut self, ttype: TType) -> Result<(), ParseError> {
        if self.consume(ttype.clone()) {
            Ok(())
        } else {
            Err(self.error(format!("Expected {ttype} but got {}", self.peek().ttype)))
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

    fn advance(&mut self) -> Token {
        if self.is_valid() {
            self.pos += 1;
        }

        self.prev()
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
