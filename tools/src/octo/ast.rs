use crate::octo::token::TType;

#[derive(Debug)]
/// For some grammars, with aliases
/// It is impossible to tell what it should be
/// So, it's up to the emitter to verify for us
/// Sometimes it's possible parse-time, but we don't care
/// since we'll have to verify it later anyways
pub enum Val {
    Ident(String),
    Num(u16),
    Reg(u8),
}

#[derive(Debug)]
pub enum Decl {
    Alias(String, Val),
    Const(String, Val),
    Stmt(Stmt),
}

#[derive(Debug)]
pub enum Stmt {
    // Instructions
    Return,
    Clear,
    /// `bcd reg`, etc.
    UnaryStmt(TType, Val),
    /// `jump` or `jump0`
    JumpStmt(TType, Val),
    Sprite(Val, Val, Val),
    Again,

    // assignment
    /// `delay := reg`
    SetDelay(Val),
    /// `buzzer := reg`
    SetBuzzer(Val),
    /// `i := num`
    SetINum(Val),
    /// `i := hex reg`
    SetIFont(Val),
    /// `i += reg`
    IncrIReg(Val),
    /// `reg := random num`
    SetRegRand(Val, Val),
    /// `reg := delay`
    SetRegDelay(Val),
    /// `reg := key`
    SetRegKey(Val),
    /// `reg := num` or `reg += num`
    /// also `reg == num`
    /// check validation during assembly time based on type
    Binary(Val, TType, Val),

    // control flow
    /// if expr then expr
    IfStmt(Box<Stmt>, Box<Stmt>),
    /// "if" expr "begin" stmt* ( "else" stmt* )? "end"
    IfStmtBlock(Box<Stmt>, Vec<Stmt>, Option<Vec<Decl>>),
    /// "loop" stmt* "again" (again MUST be the last stmt)
    LoopStmt(Vec<Decl>),
    WhileStmt(Box<Stmt>),

    // comparison
    /// if v0 key
    RegKey(Val),
    /// if v0 -key
    RegNotKey(Val),
}
