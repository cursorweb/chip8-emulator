#[derive(Debug, PartialEq)]
pub enum Token {
    Cls,
    Ret,
    J,
    Call,
    /// Special unused
    Jri0,
    Jri,

    Seqi,
    Snei,
    Seq,
    Sne,
    Sk,
    Snk,

    Seti,
    Addi,
    Set,
    Or,
    And,
    Xor,
    Add,
    Subf,
    Srlf,
    Subnf,
    Sllf,

    Rand,
    Sprite,
    Bcd,

    /// I
    I,
    Delay,
    Key,
    Sound,
    /// FONT[Vx]
    Font(u8),

    Register(u8),
    Number(u16),
    /// [I]
    MemI,

    Word,

    Comma,
    Newline,
}
