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

    pub fn gen_bytes(mut self) -> Result<Vec<Instr>, GenError> {
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
                        Some(v) => AliasVal::Reg(self.expect_alias_is_reg(*v)?),
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
                        Some(v) => AliasVal::Num(self.expect_alias_is_num(*v)?),
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
            Stmt::Sprite(vx, vy, n) => self.sprite(vx, vy, n)?,
            Stmt::Again => todo!(),
            Stmt::SetDelay(reg) => Instr::SetDelay(self.resolve_reg(reg)?),
            Stmt::SetBuzzer(reg) => Instr::SetSound(self.resolve_reg(reg)?),
            Stmt::SetINum(val) => Instr::SetAddrI(Addr::Val(self.resolve_num(val)?)),
            Stmt::SetIFont(reg) => Instr::SetFont(self.resolve_reg(reg)?),
            Stmt::IncrIReg(reg) => Instr::AddMemI(self.resolve_reg(reg)?),
            Stmt::SetRegRand(reg, num) => {
                Instr::Rand(self.resolve_reg(reg)?, self.resolve_num_u8(num)?)
            }
            Stmt::SetRegDelay(reg) => Instr::SetDelay(self.resolve_reg(reg)?),
            Stmt::SetRegKey(reg) => Instr::GetKey(self.resolve_reg(reg)?),
            Stmt::BinaryStmt(left, ttype, right) => self.binary(left, ttype, right)?,
            Stmt::IfStmt(stmt) => todo!(),
            Stmt::IfStmtBlock(stmt, stmts, decls) => todo!(),
            Stmt::LoopStmt(stmts) => todo!(),
            Stmt::WhileStmt(stmt) => todo!(),
            Stmt::Literal(val) => Instr::Word(val),
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

    fn sprite(&self, vx: Val, vy: Val, n: Val) -> Result<Instr, GenError> {
        let vx = self.resolve_reg(vx)?;
        let vy = self.resolve_reg(vy)?;
        let n = self.resolve_num(n)?;
        Ok(Instr::Draw(
            vx,
            vy,
            n.try_into().map_err(|_| format!("Sprite too high: {n}"))?,
        ))
    }

    fn binary(&self, left: Val, ttype: TType, right: Val) -> Result<Instr, GenError> {
        let left = self.resolve_reg(left)?;
        Ok(match ttype {
            TType::AssignEq => match self.resolve_any(right)? {
                AliasVal::Reg(right) => Instr::Set(left, right),
                AliasVal::Num(right) | AliasVal::Addr(right) => {
                    Instr::Seti(left, self.u16_to_u8(right)?)
                }
            },
            TType::PlusEq => match self.resolve_any(right)? {
                AliasVal::Reg(right) => Instr::Add(left, right),
                AliasVal::Num(right) | AliasVal::Addr(right) => {
                    Instr::Addi(left, self.u16_to_u8(right)?)
                }
            },
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

    fn emit(&mut self, instr: Instr) {
        self.out.push(instr);
    }

    /// Get either `num` or `reg` as an AliasVal
    fn resolve_any(&self, val: Val) -> Result<AliasVal, GenError> {
        match val {
            Val::Ident(ident) => self
                .aliases
                .get(&ident)
                .ok_or_else(|| self.error(format!("Undefined alias {ident}")))
                .copied(),
            Val::Num(n) => Ok(AliasVal::Num(n)),
            Val::Reg(r) => Ok(AliasVal::Reg(r)),
        }
    }

    /// Given a Val in AST, either typecheck it for reg, or lookup the value and then typecheck it
    fn resolve_reg(&self, val: Val) -> Result<u8, GenError> {
        match val {
            Val::Ident(ident) => self.expect_alias_is_reg(
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
            Val::Ident(ident) => self.expect_alias_is_num(
                *self
                    .aliases
                    .get(&ident)
                    .ok_or_else(|| self.error(format!("Undefined alias {ident}")))?,
            ),
            Val::Reg(r) => Err(self.error(format!("Expected number but got register v{r}"))),
            Val::Num(n) => Ok(n),
        }
    }

    fn resolve_num_u8(&self, val: Val) -> Result<u8, GenError> {
        let res = self.resolve_num(val)?;
        self.u16_to_u8(res)
    }

    fn u16_to_u8(&self, res: u16) -> Result<u8, GenError> {
        res.try_into()
            .map_err(|_| format!("Number {res} too large (needs to be a byte)"))
    }

    /// Typecheck lookup value to ensure it is a register
    fn expect_alias_is_reg(&self, val: AliasVal) -> Result<u8, GenError> {
        match val {
            AliasVal::Reg(v) => Ok(v),
            _ => Err(format!("Expected a register")),
        }
    }

    /// Typecheck lookup value to ensure it is a number
    fn expect_alias_is_num(&self, val: AliasVal) -> Result<u16, GenError> {
        match val {
            AliasVal::Num(v) => Ok(v),
            _ => Err(format!("Expected a number")),
        }
    }

    fn error(&self, message: impl Into<String>) -> GenError {
        message.into()
    }
}
