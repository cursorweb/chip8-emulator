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
enum SymbolVal {
    Reg(u8),
    Num(u16),
    Addr(u16),
}

/// TODO: add a forward pass to resolve things like loops, agains, whiles, as well as labels
pub struct CodeGen {
    trees: Vec<Span<Decl>>,
    symbols: HashMap<String, SymbolVal>,
    /// Addr should start at 0x200. This is because the program is loaded at 0x200
    addr: u16,
    out: Vec<Instr>,
    /// Every time you nest a loop, this stack stores the loop label addy
    loop_stack: Vec<u16>,
    /// Current `span` of the AST node being looked at
    curr_pos: usize,
}

impl CodeGen {
    pub fn new(trees: Vec<Span<Decl>>) -> Self {
        let symbols = HashMap::from([
            ("OCTO_KEY_1".into(), SymbolVal::Num(0x1)),
            ("OCTO_KEY_2".into(), SymbolVal::Num(0x2)),
            ("OCTO_KEY_3".into(), SymbolVal::Num(0x3)),
            ("OCTO_KEY_4".into(), SymbolVal::Num(0xC)),
            ("OCTO_KEY_Q".into(), SymbolVal::Num(0x4)),
            ("OCTO_KEY_W".into(), SymbolVal::Num(0x5)),
            ("OCTO_KEY_E".into(), SymbolVal::Num(0x6)),
            ("OCTO_KEY_R".into(), SymbolVal::Num(0xD)),
            ("OCTO_KEY_A".into(), SymbolVal::Num(0x7)),
            ("OCTO_KEY_S".into(), SymbolVal::Num(0x8)),
            ("OCTO_KEY_D".into(), SymbolVal::Num(0x9)),
            ("OCTO_KEY_F".into(), SymbolVal::Num(0xE)),
            ("OCTO_KEY_Z".into(), SymbolVal::Num(0xA)),
            ("OCTO_KEY_X".into(), SymbolVal::Num(0x0)),
            ("OCTO_KEY_C".into(), SymbolVal::Num(0xB)),
            ("OCTO_KEY_V".into(), SymbolVal::Num(0xF)),
        ]);
        Self {
            trees,
            symbols,
            addr: 0x200,
            out: vec![],
            loop_stack: vec![],
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
            Decl::Alias(alias_name, val) => {
                let val = match val {
                    Val::Ident(name) => match self.symbols.get(&name) {
                        Some(v) => SymbolVal::Reg(self.expect_alias_is_reg(*v)?),
                        None => {
                            return Err(self.error(format!(
                                "Alias {alias_name} depends on another alias {name} which doesn't exist"
                            )));
                        }
                    },
                    Val::Reg(v) => SymbolVal::Reg(v),
                    Val::Num(_) => {
                        return Err(self.error(format!(
                            "Alias {alias_name} expects register but got number"
                        )));
                    }
                };

                self.symbols.insert(alias_name, val);
            }
            Decl::Const(const_name, val) => {
                let val = match val {
                    Val::Ident(name) => match self.symbols.get(&name) {
                        Some(v) => SymbolVal::Num(self.expect_alias_is_num(*v)?),
                        None => {
                            return Err(self.error(format!(
                                "Constant {const_name} depends on another alias {name} which doesn't exist"
                            )));
                        }
                    },
                    Val::Num(v) => SymbolVal::Num(v),
                    Val::Reg(_) => {
                        return Err(self.error(format!(
                            "Constant {const_name} expects a number but got register"
                        )));
                    }
                };

                self.symbols.insert(const_name, val);
            }
            Decl::Label(name) => {
                self.symbols.insert(name, SymbolVal::Addr(self.addr));
            }
            Decl::Stmt(stmt) => {
                self.stmt(stmt)?;
            }
        }

        Ok(())
    }

    fn stmt(&mut self, stmt: Stmt) -> Result<(), ParseError> {
        let instr = match stmt {
            Stmt::Return => Instr::Ret,
            Stmt::Clear => Instr::Cls,
            Stmt::UnaryStmt(ttype, val) => self.unary_stmt(ttype, val)?,
            Stmt::Sprite(vx, vy, n) => self.sprite(vx, vy, n)?,
            Stmt::Again => Instr::J(Addr::Val(
                *self
                    .loop_stack
                    .last()
                    .ok_or(self.error("again can only be used to close loops"))?,
            )),
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
            Stmt::Literal(val) => Instr::Word(val),
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
        self.loop_stack.push(self.addr);
        for stmt in stmts {
            let stmt = self.unwrap_span(stmt);
            self.stmt(stmt)?;
        }
        self.loop_stack.pop();
        Ok(())
    }

    fn emit(&mut self, instr: Instr) {
        self.out.push(instr);
        self.addr += 2;
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
            Val::Ident(ident) => self.expect_alias_is_reg(
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
            Val::Ident(ident) => self.expect_alias_is_num(
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

    /// Typecheck lookup value to ensure it is a register
    fn expect_alias_is_reg(&self, val: SymbolVal) -> Result<u8, ParseError> {
        match val {
            SymbolVal::Reg(v) => Ok(v),
            _ => Err(self.error(format!("Expected a register"))),
        }
    }

    /// Typecheck lookup value to ensure it is a number
    fn expect_alias_is_num(&self, val: SymbolVal) -> Result<u16, ParseError> {
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
