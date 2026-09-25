use std::collections::HashMap;

#[derive(Debug, PartialEq, Clone)]
pub enum Instr {
    Cls,
    Ret,
    J(Addr),
    Call(Addr),
    /// Special unused
    Jri0(u16),
    Jri(u8, u8),

    Seqi(u8, u8),
    Snei(u8, u8),
    Seq(u8, u8),
    Sne(u8, u8),
    Sk(u8),
    Snk(u8),

    /// `seti $x, NN`
    Seti(u8, u8),
    /// `seti I, 0xNNN`
    SetAddrI(Addr),
    Addi(u8, u8),

    Set(u8, u8),
    Or(u8, u8),
    And(u8, u8),
    Xor(u8, u8),
    Add(u8, u8),
    Subf(u8, u8),
    /// `srlf $0, $1` or for unused `srlf $0`
    /// This parser will support both, and `srlf $0` -> `srlf $0 $0`
    Srlf(u8, u8),
    Subnf(u8, u8),
    /// See `srlf`
    Sllf(u8, u8),

    AddMemI(u8),

    Rand(u8, u8),
    Sprite(u8, u8, u8),
    Bcd(u8),

    /// `set $x, DELAY`
    GetDelay(u8),
    /// `set $x, KEY`
    GetKey(u8),
    /// `set DELAY, $x`
    SetDelay(u8),
    /// `set SOUND, $x`
    SetSound(u8),

    SetFont(u8),
    /// `set [I], $x`
    SetMemI(u8),
    /// `set $x, [I]`
    GetMemI(u8),

    Word(u16),
    Byte(u8),

    /// `label:`
    Label(String),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Addr {
    Val(u16),
    Label(String),
}

impl Addr {
    fn lookup(&self, lookup: &HashMap<String, u16>) -> u16 {
        match &self {
            Addr::Val(v) => *v,
            Addr::Label(s) => *lookup.get(&s).unwrap(),
        }
    }
}

impl Instr {
    fn to_bytes(&self, v: &HashMap<String, u16>) -> u16 {
        match self {
            Instr::Cls => 0x00E0,
            Instr::Ret => 0x00EE,
            Instr::J(addr) => 0x1000 | addr.lookup(&v),
            Instr::Call(addr) => todo!(),
            Instr::Jri0(_) => todo!(),
            Instr::Jri(_, _) => todo!(),
            Instr::Seqi(_, _) => todo!(),
            Instr::Snei(_, _) => todo!(),
            Instr::Seq(_, _) => todo!(),
            Instr::Sne(_, _) => todo!(),
            Instr::Sk(_) => todo!(),
            Instr::Snk(_) => todo!(),
            Instr::Seti(_, _) => todo!(),
            Instr::SetAddrI(addr) => todo!(),
            Instr::Addi(_, _) => todo!(),
            Instr::Set(_, _) => todo!(),
            Instr::Or(_, _) => todo!(),
            Instr::And(_, _) => todo!(),
            Instr::Xor(_, _) => todo!(),
            Instr::Add(_, _) => todo!(),
            Instr::Subf(_, _) => todo!(),
            Instr::Srlf(_, _) => todo!(),
            Instr::Subnf(_, _) => todo!(),
            Instr::Sllf(_, _) => todo!(),
            Instr::AddMemI(_) => todo!(),
            Instr::Rand(_, _) => todo!(),
            Instr::Sprite(_, _, _) => todo!(),
            Instr::Bcd(_) => todo!(),
            Instr::GetDelay(_) => todo!(),
            Instr::GetKey(_) => todo!(),
            Instr::SetDelay(_) => todo!(),
            Instr::SetSound(_) => todo!(),
            Instr::SetFont(_) => todo!(),
            Instr::SetMemI(_) => todo!(),
            Instr::GetMemI(_) => todo!(),
            Instr::Word(_) => todo!(),
            Instr::Byte(_) => todo!(),
            Instr::Label(_) => todo!(),
        }
    }
}
