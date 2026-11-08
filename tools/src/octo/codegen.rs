use std::collections::HashMap;

use crate::octo::ast::Decl;

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
    out: Vec<u8>,
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

    fn gen_bytes(mut self) -> Result<Vec<u8>, GenError> {
        let mut out = vec![];

        for decl in std::mem::take(&mut self.trees) {
            self.decl(decl)?;
        }

        Ok(out)
    }

    fn decl(&mut self, decl: Decl) -> Result<(), GenError> {
        match decl {
            Decl::Alias(alias_name, val) => {
                let val = match val {
                    Val::Ident(name) => match self.aliases.get(&name) {
                        Some(v) => self.expect_reg(*v)?,
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
                        Some(v) => self.expect_num(*v)?,
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
            Decl::Stmt(stmt) => {}
        }

        Ok(())
    }

    fn emit(&mut self, bytes: &[u8]) {
        self.out.extend_from_slice(bytes);
        self.addr += bytes.len() as u16;
    }

    fn expect_reg(&self, val: AliasVal) -> Result<AliasVal, GenError> {
        match val {
            AliasVal::Reg(_) => Ok(val),
            _ => Err(format!("Expected a register")),
        }
    }

    fn expect_num(&self, val: AliasVal) -> Result<AliasVal, GenError> {
        match val {
            AliasVal::Num(_) => Ok(val),
            _ => Err(format!("Expected a number")),
        }
    }

    fn error(&self, message: impl Into<String>) -> GenError {
        message.into()
    }
}
