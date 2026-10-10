pub enum Token {
    // Registers / literals
    Register(u8),
    Number(u16),
    Ident(String),

    // Directives
    /// :alias x v1
    Alias(String, u8),

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

    // Punctuation
    Colon,
    Semi,

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
