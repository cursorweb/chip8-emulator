#[derive(Debug)]
pub struct Token {
    pub ttype: TType,
    pub pos: usize,
}

impl Token {
    pub fn new(ttype: TType, pos: usize) -> Self {
        Self { ttype, pos }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum TType {
    // Registers / literals
    Register(u8),
    Number(u16),
    Ident(String),

    // Directives
    /// :alias
    Alias,
    /// :const
    Const,

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
    While,

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
