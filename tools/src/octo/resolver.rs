use std::{collections::HashMap, debug_assert_matches};

use crate::{
    asmparser::ParseError,
    octo::{
        ast::{Decl, Span, Stmt, Val},
        codegen::SymbolVal,
    },
};

pub struct Resolver<'a> {
    trees: &'a Vec<Span<Decl>>,
    /// Lookup tabel for `:const` `:alias` etc
    pub symbols: HashMap<String, SymbolVal>,
    /// Lookup table for statements, encoded based on line info
    pub stmt_addrs: HashMap<usize, u16>,
    /// Addr should start at 0x200. This is because the program is loaded at 0x200
    addr: u16,
    /// Current "line info" of the ast looked at
    curr_pos: usize,
    in_loop: bool,
}

impl<'a> Resolver<'a> {
    pub fn new(trees: &'a Vec<Span<Decl>>) -> Self {
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
            stmt_addrs: HashMap::new(),
            addr: 0x200,
            curr_pos: 0,
            in_loop: false,
        }
    }

    /// Returns (symbols, stmt_addrs)
    pub fn resolve(
        mut self,
    ) -> Result<(HashMap<String, SymbolVal>, HashMap<usize, u16>, Option<u16>), ParseError> {
        for decl in self.trees {
            let decl = self.unwrap_span(decl);
            self.decl(decl)?;
        }

        let Some(SymbolVal::Addr(main_label)) = self.symbols.get("main").copied() else {
            debug_assert_matches!(self.symbols.get("main"), None);
            return Err(self.error("Program needs a main label"));
        };

        let main_addr = if main_label != 0x200 {
            // need to add a jump statement to main at the start, so shift everything by 2:
            for val in self.symbols.values_mut() {
                if let SymbolVal::Addr(addr) = val {
                    *addr += 2;
                }
            }

            for addr in self.stmt_addrs.values_mut() {
                *addr += 2;
            }

            Some(main_label + 2)
        } else {
            None
        };

        Ok((self.symbols, self.stmt_addrs, main_addr))
    }

    fn decl(&mut self, decl: &Decl) -> Result<(), ParseError> {
        match decl {
            Decl::Alias(alias_name, val) => {
                let val = match val {
                    Val::Ident(name) => match self.symbols.get(name) {
                        Some(v) => SymbolVal::Reg(self.expect_symbol_is_reg(*v)?),
                        None => {
                            return Err(self.error(format!(
                                "Alias {alias_name} depends on another alias {name} which doesn't exist"
                            )));
                        }
                    },
                    Val::Reg(v) => SymbolVal::Reg(*v),
                    Val::Num(_) => {
                        return Err(self.error(format!(
                            "Alias {alias_name} expects register but got number"
                        )));
                    }
                };

                self.symbols.insert(alias_name.clone(), val);
            }
            Decl::Const(const_name, val) => {
                let val = match val {
                    Val::Ident(name) => match self.symbols.get(name) {
                        Some(v) => SymbolVal::Num(self.expect_symbol_is_num(*v)?),
                        None => {
                            return Err(self.error(format!(
                                "Constant {const_name} depends on another alias {name} which doesn't exist"
                            )));
                        }
                    },
                    Val::Num(v) => SymbolVal::Num(*v),
                    Val::Reg(_) => {
                        return Err(self.error(format!(
                            "Constant {const_name} expects a number but got register"
                        )));
                    }
                };

                self.symbols.insert(const_name.clone(), val);
            }
            Decl::Label(name) => {
                self.symbols
                    .insert(name.clone(), SymbolVal::Addr(self.addr));
            }
            Decl::Stmt(stmt) => {
                let stmt = self.unwrap_span(stmt);
                self.stmt(stmt)?
            }
        }
        Ok(())
    }

    fn stmt(&mut self, stmt: &Stmt) -> Result<(), ParseError> {
        match stmt {
            // todo: custom instructions require some helpers
            // Stmt::IfStmt(span) => {}
            Stmt::IfStmtBlock(_, _, _) => todo!(),
            Stmt::LoopStmt(spans) => return self.loop_stmt(spans),
            Stmt::Again => {
                if !self.in_loop {
                    return Err(self.error("Again can only be used to close loops"));
                }
            }
            Stmt::Literal(_) => {
                self.addr += 1;
                return Ok(());
            }
            _ => {}
        }

        // i'm scared...
        // ensure that this is right...
        self.addr += 2;
        Ok(())
    }

    fn loop_stmt(&mut self, spans: &Vec<Span<Stmt>>) -> Result<(), ParseError> {
        let in_loop = self.in_loop;
        self.in_loop = true;

        let loop_start = self.addr;

        // loop
        let mut whiles = vec![];
        for span in spans {
            let stmt = self.unwrap_span(span);
            if matches!(stmt, &Stmt::WhileStmt(_)) {
                whiles.push(span);
            }
            self.stmt(stmt)?;
        }

        // again
        debug_assert_matches!(spans.last().unwrap().val, Stmt::Again);
        let again = spans.last().unwrap();
        self.stmt_addrs.insert(again.pos, loop_start);

        // resolve whiles
        let loop_end = self.addr + 2; // one past the again
        for w in whiles {
            self.stmt_addrs.insert(w.pos, loop_end);
        }

        self.in_loop = in_loop;
        Ok(())
    }

    fn unwrap_span<'b, T>(&mut self, span: &'b Span<T>) -> &'b T {
        self.curr_pos = span.pos;
        &span.val
    }

    fn error(&self, message: impl Into<String>) -> ParseError {
        ParseError {
            pos: self.curr_pos,
            message: message.into(),
        }
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
}
