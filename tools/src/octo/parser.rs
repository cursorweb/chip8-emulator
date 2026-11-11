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
                TType::Ident(s) => Decl::Const(s, self.val()?),
                _ => return Err(self.error(format!("Expected name but got {}", self.peek().ttype))),
            }
        } else if self.consume(TType::Colon) {
            match self.advance().ttype {
                TType::Ident(s) => Decl::Label(s),
                _ => return Err(self.error(format!("Expected name but got {}", self.peek().ttype))),
            }
        } else {
            Decl::Stmt(self.stmt()?)
        })
    }

    fn stmt(&mut self) -> StmtResult {
        Ok(if self.consumes(&[TType::Return, TType::Semi]) {
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
            Stmt::UnaryStmt(self.prev().ttype, self.val()?)
        } else if self.consume(TType::If) {
            self.if_stmt()?
        } else if self.consume(TType::Loop) {
            self.loop_stmt()?
        } else if self.consume(TType::While) {
            self.while_stmt()?
        } else {
            match self.peek().ttype {
                TType::Number(n) => {
                    self.advance();
                    Stmt::Literal(n)
                }
                _ => self.assignment()?,
            }
        })
    }

    fn if_stmt(&mut self) -> StmtResult {
        // TODO: we must finish the if statement for begin end
        let comp = self.comparison()?;
        self.expect(TType::Then)?;
        Ok(Stmt::IfStmt(Box::new(comp)))
    }

    fn loop_stmt(&mut self) -> StmtResult {
        let mut stmts = vec![];
        loop {
            let stmt = self.stmt()?;
            let has_again = matches!(stmt, Stmt::Again);
            stmts.push(stmt);
            if has_again {
                break;
            }
        }
        Ok(Stmt::LoopStmt(stmts))
    }

    fn while_stmt(&mut self) -> StmtResult {
        let comp = self.comparison()?;
        Ok(Stmt::WhileStmt(Box::new(comp)))
    }

    fn assignment(&mut self) -> StmtResult {
        Ok(if self.consume(TType::Delay) {
            self.expect(TType::AssignEq)?;
            Stmt::SetDelay(self.val()?)
        } else if self.consume(TType::Buzzer) {
            self.expect(TType::AssignEq)?;
            Stmt::SetBuzzer(self.val()?)
        } else if self.consume(TType::I) {
            if self.consume(TType::AssignEq) {
                if self.consume(TType::Hex) {
                    Stmt::SetIFont(self.val()?)
                } else {
                    Stmt::SetINum(self.val()?)
                }
            } else if self.consume(TType::PlusEq) {
                Stmt::IncrIReg(self.val()?)
            } else {
                return Err(self.error(format!(
                    "Expected ':=' or '+=' after 'i' but got {}",
                    self.peek().ttype
                )));
            }
        } else {
            let left = self.val()?;
            if self.consume(TType::AssignEq) {
                if self.consume(TType::Random) {
                    Stmt::SetRegRand(left, self.val()?)
                } else if self.consume(TType::Delay) {
                    Stmt::SetRegDelay(left)
                } else if self.consume(TType::Key) {
                    Stmt::SetRegKey(left)
                } else {
                    Stmt::Binary(left, TType::AssignEq, self.val()?)
                }
            } else if self.consumes(&[
                TType::PlusEq,
                TType::MinusEq,
                TType::EqMinus,
                TType::OrEq,
                TType::AndEq,
                TType::XorEq,
                TType::ShrEq,
                TType::ShlEq,
            ]) {
                Stmt::Binary(left, self.prev().ttype, self.val()?)
            } else {
                return Err(self.error(format!(
                    "Unexpected token {} (trying to parse assignment)",
                    self.peek().ttype
                )));
            }
        })
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

    /// Return a `Val` type: Reg, Num, or Ident (for you to check later)
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
