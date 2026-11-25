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
/// A *span* is a region of source code corresponding to an AST node
pub struct Span<T> {
    pub val: T,
    pub pos: usize,
}

#[derive(Debug)]
pub enum Decl {
    /// :alias x v0
    Alias(String, Val),
    /// :const pi 3
    Const(String, Val),
    Label(String),
    Stmt(Span<Stmt>),
}

#[derive(Debug)]
pub enum Stmt {
    // Instructions
    Return,
    Clear,
    /// `bcd reg`, `save`, `load`, `j`, `j0`
    UnaryStmt(TType, Val),
    /// `draw vx vy n`
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
    /// check validation during assembly time based on type
    BinaryStmt(Val, TType, Val),

    // control flow
    /// if <comparison> then
    IfStmt(Span<CompExpr>),
    /// "if" expr "begin" stmt* ( "else" stmt* )? "end"
    /// TODO: implement
    IfStmtBlock(Span<CompExpr>, Vec<Span<Stmt>>, Option<Vec<Span<Decl>>>),
    /// "loop" stmt* "again" (again MUST be the last stmt)
    LoopStmt(Vec<Span<Stmt>>),
    WhileStmt(Span<CompExpr>),

    /// 0x67 MUST be a byte
    Literal(u8),
}

/// Comparisons that act like an 'abstraction' and aren't real instructions
#[derive(Debug)]
pub enum CompExpr {
    /// reg == num, reg != num, etc.
    BinaryExpr(Val, TType, Val),
    /// if v0 key
    RegKey(Val),
    /// if v0 -key
    RegNotKey(Val),
}
