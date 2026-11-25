use std::collections::HashMap;

use crate::{
    asmparser::ParseError,
    instr::{Addr, Instr},
    octo::{
        ast::{CompExpr, Decl, Span, Stmt},
        token::TType,
    },
};

use super::ast::Val;

/// Aliases/Consts/Addys all get put into the same
/// lookup table
#[derive(Clone, Copy)]
pub enum SymbolVal {
    Reg(u8),
    Num(u16),
    Addr(u16),
}

/// TODO: add a forward pass to resolve things like loops, agains, whiles, as well as labels
pub struct CodeGen {
    trees: Vec<Span<Decl>>,
    symbols: HashMap<String, SymbolVal>,
    stmt_addrs: HashMap<usize, u16>,
    out: Vec<Instr>,
    /// Current "line info" of the AST node being looked at
    curr_pos: usize,
}

impl CodeGen {
    pub fn new(
        trees: Vec<Span<Decl>>,
        (symbols, stmt_addrs): (HashMap<String, SymbolVal>, HashMap<usize, u16>),
    ) -> Self {
        Self {
            trees,
            symbols,
            stmt_addrs,
            out: vec![],
            curr_pos: 0,
        }
    }

    pub fn gen_bytes(mut self) -> Result<Vec<Instr>, ParseError> {
        for decl in std::mem::take(&mut self.trees) {
            let decl = self.unwrap_span(decl);
            self.decl(decl)?;
        }

        Ok(self.out)
    }

    fn decl(&mut self, decl: Decl) -> Result<(), ParseError> {
        match decl {
            Decl::Stmt(span) => {
                self.stmt(span)?;
            }
            Decl::Alias(_, _) => {}
            Decl::Const(_, _) => {}
            Decl::Label(_) => {}
        }

        Ok(())
    }

    fn stmt(&mut self, span: Span<Stmt>) -> Result<(), ParseError> {
        let stmt = self.unwrap_span(span);
        let instr = match stmt {
            Stmt::Return => Instr::Ret,
            Stmt::Clear => Instr::Cls,
            Stmt::UnaryStmt(ttype, val) => self.unary_stmt(ttype, val)?,
            Stmt::Sprite(vx, vy, n) => self.sprite(vx, vy, n)?,
            // curr_pos == stmt pos
            Stmt::Again => Instr::J(Addr::Val(self.stmt_addrs[&self.curr_pos])),
            Stmt::SetDelay(reg) => Instr::SetDelay(self.resolve_reg(reg)?),
            Stmt::SetBuzzer(reg) => Instr::SetSound(self.resolve_reg(reg)?),
            Stmt::SetINum(val) => Instr::SetAddrI(Addr::Val(self.resolve_num(val)?)),
            Stmt::SetIFont(reg) => Instr::SetFont(self.resolve_reg(reg)?),
            Stmt::IncrIReg(reg) => Instr::AddMemI(self.resolve_reg(reg)?),
            Stmt::SetRegRand(reg, num) => {
                Instr::Rand(self.resolve_reg(reg)?, self.resolve_num_u8(num)?)
            }
            Stmt::SetRegDelay(reg) => Instr::GetDelay(self.resolve_reg(reg)?),
            Stmt::SetRegKey(reg) => Instr::GetKey(self.resolve_reg(reg)?),
            Stmt::BinaryStmt(left, ttype, right) => self.binary(left, ttype, right)?,
            Stmt::IfStmt(comp) => self.if_stmt(comp)?,
            Stmt::IfStmtBlock(_, _, _) => todo!(),
            Stmt::LoopStmt(stmts) => return self.loop_stmt(stmts),
            Stmt::WhileStmt(_) => todo!(),
            Stmt::Literal(val) => Instr::Byte(val),
        };
        self.emit(instr);
        Ok(())
    }

    fn unary_stmt(&self, ttype: TType, val: Val) -> Result<Instr, ParseError> {
        Ok(match ttype {
            TType::Bcd => {
                let reg = self.resolve_reg(val)?;
                Instr::Bcd(reg)
            }
            TType::Save => {
                let reg = self.resolve_reg(val)?;
                Instr::SetMemI(reg)
            }
            TType::Load => {
                let reg = self.resolve_reg(val)?;
                Instr::GetMemI(reg)
            }
            TType::Jump => {
                let num = self.resolve_num(val)?;
                // TODO Addr::Label
                Instr::J(Addr::Val(num))
            }
            TType::Jump0 => {
                let num = self.resolve_num(val)?;
                Instr::Jri0(num)
            }
            ttype => unreachable!("AST should not emit {ttype} for unary statement"),
        })
    }

    fn sprite(&self, vx: Val, vy: Val, n: Val) -> Result<Instr, ParseError> {
        let vx = self.resolve_reg(vx)?;
        let vy = self.resolve_reg(vy)?;
        let n = self.resolve_num(n)?;
        Ok(Instr::Draw(
            vx,
            vy,
            n.try_into()
                .map_err(|_| self.error(format!("Sprite too high: {n}")))?,
        ))
    }

    fn binary(&self, left: Val, ttype: TType, right: Val) -> Result<Instr, ParseError> {
        let left = self.resolve_reg(left)?;
        Ok(match ttype {
            TType::AssignEq => self.resolve_binary(left, right, Instr::Set, Instr::Seti)?,
            TType::PlusEq => self.resolve_binary(left, right, Instr::Add, Instr::Addi)?,
            TType::MinusEq => Instr::Subf(left, self.resolve_reg(right)?),
            TType::EqMinus => Instr::Subnf(left, self.resolve_reg(right)?),
            TType::OrEq => Instr::Or(left, self.resolve_reg(right)?),
            TType::AndEq => Instr::And(left, self.resolve_reg(right)?),
            TType::XorEq => Instr::Xor(left, self.resolve_reg(right)?),
            TType::ShrEq => Instr::Srlf(left, self.resolve_reg(right)?),
            TType::ShlEq => Instr::Sllf(left, self.resolve_reg(right)?),
            _ => unreachable!("Shouldn't be able to reach {ttype}"),
        })
    }

    fn if_stmt(&mut self, comp: Span<CompExpr>) -> Result<Instr, ParseError> {
        let comp = self.unwrap_span(comp);
        Ok(match comp {
            CompExpr::BinaryExpr(left, ttype, right) => {
                let left = self.resolve_reg(left)?;
                match ttype {
                    TType::EqEq => self.resolve_binary(left, right, Instr::Seq, Instr::Seqi)?,
                    TType::NotEq => self.resolve_binary(left, right, Instr::Sne, Instr::Snei)?,
                    TType::Greater => todo!(),
                    TType::Less => todo!(),
                    TType::GreaterEq => todo!(),
                    TType::LessEq => todo!(),
                    _ => unreachable!("Shouldn't be able to reach {ttype}"),
                }
            }
            CompExpr::RegKey(val) => Instr::Sk(self.resolve_reg(val)?),
            CompExpr::RegNotKey(val) => Instr::Snk(self.resolve_reg(val)?),
        })
    }

    fn loop_stmt(&mut self, stmts: Vec<Span<Stmt>>) -> Result<(), ParseError> {
        for stmt in stmts {
            self.stmt(stmt)?;
        }
        Ok(())
    }

    fn emit(&mut self, instr: Instr) {
        self.out.push(instr);
    }

    fn resolve_binary(
        &self,
        reg: u8,
        right: Val,
        reg_op: impl FnOnce(u8, u8) -> Instr,
        imm_op: impl FnOnce(u8, u8) -> Instr,
    ) -> Result<Instr, ParseError> {
        match self.resolve_any(right)? {
            SymbolVal::Reg(right) => Ok(reg_op(reg, right)),
            SymbolVal::Num(right) | SymbolVal::Addr(right) => {
                Ok(imm_op(reg, self.u16_to_u8(right)?))
            }
        }
    }

    /// Get either `num` or `reg` as an AliasVal
    fn resolve_any(&self, val: Val) -> Result<SymbolVal, ParseError> {
        match val {
            Val::Ident(ident) => self
                .symbols
                .get(&ident)
                .ok_or_else(|| self.error(format!("Undefined alias {ident}")))
                .copied(),
            Val::Num(n) => Ok(SymbolVal::Num(n)),
            Val::Reg(r) => Ok(SymbolVal::Reg(r)),
        }
    }

    /// Given a Val in AST, either typecheck it for reg, or lookup the value and then typecheck it
    fn resolve_reg(&self, val: Val) -> Result<u8, ParseError> {
        match val {
            Val::Ident(ident) => self.expect_symbol_is_reg(
                *self
                    .symbols
                    .get(&ident)
                    .ok_or_else(|| self.error(format!("Undefined alias {ident}")))?,
            ),
            Val::Num(n) => Err(self.error(format!("Expected register but got number {n}"))),
            Val::Reg(r) => Ok(r),
        }
    }

    /// Given a Val in AST, either typecheck it for num, or lookup the value and then typecheck it
    fn resolve_num(&self, val: Val) -> Result<u16, ParseError> {
        match val {
            Val::Ident(ident) => self.expect_symbol_is_num(
                *self
                    .symbols
                    .get(&ident)
                    .ok_or_else(|| self.error(format!("Undefined alias {ident}")))?,
            ),
            Val::Reg(r) => Err(self.error(format!("Expected number but got register v{r}"))),
            Val::Num(n) => Ok(n),
        }
    }

    fn resolve_num_u8(&self, val: Val) -> Result<u8, ParseError> {
        let res = self.resolve_num(val)?;
        self.u16_to_u8(res)
    }

    fn u16_to_u8(&self, res: u16) -> Result<u8, ParseError> {
        res.try_into()
            .map_err(|_| self.error(format!("Number {res} too large (needs to be a byte)")))
    }

    /// Typecheck symbol to ensure it is a register
    fn expect_symbol_is_reg(&self, val: SymbolVal) -> Result<u8, ParseError> {
        match val {
            SymbolVal::Reg(v) => Ok(v),
            _ => Err(self.error(format!("Expected a register"))),
        }
    }

    /// Typecheck symbol to ensure it is a number
    fn expect_symbol_is_num(&self, val: SymbolVal) -> Result<u16, ParseError> {
        match val {
            SymbolVal::Num(v) => Ok(v),
            SymbolVal::Addr(v) => Ok(v),
            _ => Err(self.error(format!("Expected a number"))),
        }
    }

    fn unwrap_span<T>(&mut self, span: Span<T>) -> T {
        self.curr_pos = span.pos;
        span.val
    }

    fn error(&self, message: impl Into<String>) -> ParseError {
        ParseError {
            pos: self.curr_pos,
            message: message.into(),
        }
    }
}
