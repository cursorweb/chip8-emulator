#[derive(Debug)]
pub enum Token {
    // Registers / literals
    Register(u8),
    Number(u16),
    Ident(String),

    // Directives
    /// :alias
    Alias,

    // Keywords / Instructions
    Clear,
    Return,
    Bcd,
    Save,
    Load,
    Sprite,
    Jump,
    Jump0,

    If,
    Then,
    Else,
    Begin,
    End,

    Loop,
    Again,

    Random,
    Key,

    // Registers
    // Vx
    V(u8),
    // I
    I,
    Buzzer,
    Delay,

    // Punctuation
    Colon,
    Semi,
    Minus,

    // Operators
    /// :=
    AssignEq,
    /// +=
    PlusEq,
    /// -=
    MinusEq,
    /// ==
    EqEq,
    /// !=
    NotEq,
    /// <
    Less,
    /// >
    Greater,
    /// <=
    LessEq,
    /// >=
    GreaterEq,

    /// |=
    OrEq,
    /// &=
    AndEq,
    /// ^=
    XorEq,
    /// >>=
    ShrEq,
    /// <<=
    ShlEq,

    // Other
    Eof,
}
