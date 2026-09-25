#[derive(Debug, PartialEq, Clone)]
pub enum Instr {
    Cls,
    Ret,
    J(u16),
    Call(u16),
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
    SetI(u16),
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

    /// `label:`
    Label(String),
}
