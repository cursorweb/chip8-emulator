use crate::octo::token::TType;

#[derive(Debug)]
pub enum Reg {
    Ident(String),
    Val(u8),
}

#[derive(Debug)]
pub enum Num {
    Ident(String),
    Val(u16),
}

#[derive(Debug)]
pub enum Stmt {
    Return,
    Clear,
    /// `bcd reg`, etc.
    UnaryStmt(TType, Reg),
    /// `jump` or `jump0`
    JumpStmt(TType, Num),
    Sprite(Reg, Reg, Num),
    Alias(String, Reg),
    Const(String, Num),
    ExprStmt(Expr),
    Again,
}

#[derive(Debug)]
pub enum Expr {
    // assignment
    /// `delay := reg`
    SetDelay(Reg),
    /// `buzzer := reg`
    SetBuzzer(Reg),
    /// `i := num`
    SetINum(Num),
    /// `i := hex reg`
    SetIFont(Reg),
    /// `i += reg`
    IncrIReg(Reg),
    /// `reg := random num`
    SetRegRand(Reg, Num),
    /// `reg := delay`
    SetRegDelay(Reg),
    /// `reg := key`
    SetRegKey(Reg),
    /// `reg := num` or `reg += num`
    /// also `reg == num`
    BinaryRegNum(Reg, TType, Num),
    BinaryRegReg(Reg, TType, Reg),

    // control flow
    /// if expr then expr
    IfStmt(Box<Expr>, Box<Stmt>),
    /// "if" expr "begin" stmt* ( "else" stmt* )? "end"
    IfStmtBlock(Box<Expr>, Vec<Stmt>, Option<Vec<Stmt>>),
    /// "loop" stmt* "again" (again MUST be the last stmt)
    LoopStmt(Vec<Stmt>),
    WhileStmt(Box<Expr>),

    // comparison
    /// if v0 key
    RegKey(Reg),
    /// if v0 -key
    RegNotKey(Reg),
}
