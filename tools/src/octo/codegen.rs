use std::collections::HashMap;

use crate::{
    instr::{Addr, Instr},
    octo::{
        ast::{Decl, Stmt},
        token::TType,
    },
};

use super::ast::Val;

/// Aliases/Consts/Addys all get put into the same
/// lookup table
#[derive(Clone, Copy)]
enum AliasVal {
    Reg(u8),
    Num(u16),
    Addr(u16),
}

pub struct CodeGen {
    trees: Vec<Decl>,
    aliases: HashMap<String, AliasVal>,
    addr: u16,
    out: Vec<Instr>,
}

// TODO: have line info?
type GenError = String;

impl CodeGen {
    pub fn new(trees: Vec<Decl>) -> Self {
        Self {
            trees,
            aliases: HashMap::new(),
            addr: 0,
            out: vec![],
        }
    }

    fn gen_bytes(mut self) -> Result<Vec<Instr>, GenError> {
        for decl in std::mem::take(&mut self.trees) {
            self.decl(decl)?;
        }

        Ok(self.out)
    }

    fn decl(&mut self, decl: Decl) -> Result<(), GenError> {
        match decl {
            Decl::Alias(alias_name, val) => {
                let val = match val {
                    Val::Ident(name) => match self.aliases.get(&name) {
                        Some(v) => AliasVal::Reg(self.expect_reg(*v)?),
                        None => {
                            return Err(format!(
                                "Alias {alias_name} depends on another alias {name} which doesn't exist"
                            ));
                        }
                    },
                    Val::Reg(v) => AliasVal::Reg(v),
                    Val::Num(_) => {
                        return Err(format!(
                            "Alias {alias_name} expects register but got number"
                        ));
                    }
                };

                self.aliases.insert(alias_name, val);
            }
            Decl::Const(const_name, val) => {
                let val = match val {
                    Val::Ident(name) => match self.aliases.get(&name) {
                        Some(v) => AliasVal::Num(self.expect_num(*v)?),
                        None => {
                            return Err(format!(
                                "Constant {const_name} depends on another alias {name} which doesn't exist"
                            ));
                        }
                    },
                    Val::Num(v) => AliasVal::Num(v),
                    Val::Reg(_) => {
                        return Err(format!(
                            "Constant {const_name} expects a number but got register"
                        ));
                    }
                };

                self.aliases.insert(const_name, val);
            }
            Decl::Label(name) => {
                self.aliases.insert(name, AliasVal::Addr(self.addr));
            }
            Decl::Stmt(stmt) => {
                self.stmt(stmt)?;
            }
        }

        Ok(())
    }

    fn stmt(&mut self, stmt: Stmt) -> Result<(), GenError> {
        self.emit(match stmt {
            Stmt::Return => Instr::Ret,
            Stmt::Clear => Instr::Cls,
            Stmt::UnaryStmt(ttype, val) => self.unary_stmt(ttype, val)?,
            Stmt::Sprite(val, val1, val2) => todo!(),
            Stmt::Again => todo!(),
            Stmt::SetDelay(val) => todo!(),
            Stmt::SetBuzzer(val) => todo!(),
            Stmt::SetINum(val) => todo!(),
            Stmt::SetIFont(val) => todo!(),
            Stmt::IncrIReg(val) => todo!(),
            Stmt::SetRegRand(val, val1) => todo!(),
            Stmt::SetRegDelay(val) => todo!(),
            Stmt::SetRegKey(val) => todo!(),
            Stmt::Binary(val, ttype, val1) => todo!(),
            Stmt::IfStmt(stmt) => todo!(),
            Stmt::IfStmtBlock(stmt, stmts, decls) => todo!(),
            Stmt::LoopStmt(stmts) => todo!(),
            Stmt::WhileStmt(stmt) => todo!(),
            Stmt::RegKey(val) => todo!(),
            Stmt::RegNotKey(val) => todo!(),
            Stmt::Literal(_) => todo!(),
        });
        Ok(())
    }

    fn unary_stmt(&self, ttype: TType, val: Val) -> Result<Instr, GenError> {
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

    fn emit(&mut self, instr: Instr) {
        self.out.push(instr);
    }

    /// Given a Val in AST, either typecheck it for reg, or lookup the value and then typecheck it
    fn resolve_reg(&self, val: Val) -> Result<u8, GenError> {
        match val {
            Val::Ident(ident) => self.expect_reg(
                *self
                    .aliases
                    .get(&ident)
                    .ok_or_else(|| self.error(format!("Undefined alias {ident}")))?,
            ),
            Val::Num(n) => Err(self.error(format!("Expected register but got number {n}"))),
            Val::Reg(r) => Ok(r),
        }
    }

    /// Given a Val in AST, either typecheck it for num, or lookup the value and then typecheck it
    fn resolve_num(&self, val: Val) -> Result<u16, GenError> {
        match val {
            Val::Ident(ident) => self.expect_num(
                *self
                    .aliases
                    .get(&ident)
                    .ok_or_else(|| self.error(format!("Undefined alias {ident}")))?,
            ),
            Val::Reg(r) => Err(self.error(format!("Expected number but got register v{r}"))),
            Val::Num(n) => Ok(n),
        }
    }

    /// Typecheck lookup value to ensure it is a register
    fn expect_reg(&self, val: AliasVal) -> Result<u8, GenError> {
        match val {
            AliasVal::Reg(v) => Ok(v),
            _ => Err(format!("Expected a register")),
        }
    }

    /// Typecheck lookup value to ensure it is a number
    fn expect_num(&self, val: AliasVal) -> Result<u16, GenError> {
        match val {
            AliasVal::Num(v) => Ok(v),
            _ => Err(format!("Expected a number")),
        }
    }

    fn error(&self, message: impl Into<String>) -> GenError {
        message.into()
    }
}
