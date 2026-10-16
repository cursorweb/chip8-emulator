use std::fmt::Display;

#[derive(Debug, Clone)]
pub struct Token {
    pub ttype: TType,
    pub pos: usize,
}

impl Token {
    pub fn new(ttype: TType, pos: usize) -> Self {
        Self { ttype, pos }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum TType {
    // Registers / literals
    /// Vx
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

impl Display for TType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TType::Register(u) => write!(f, "v{u}"),
            TType::Number(u) => write!(f, "{u}"),
            TType::Ident(x) => write!(f, "{x}"),
            TType::Alias => write!(f, ":alias"),
            TType::Const => write!(f, ":const"),
            TType::Clear => write!(f, "clear"),
            TType::Return => write!(f, "return"),
            TType::Bcd => write!(f, "bcd"),
            TType::Save => write!(f, "save"),
            TType::Load => write!(f, "load"),
            TType::Sprite => write!(f, "sprite"),
            TType::Jump => write!(f, "jump"),
            TType::Jump0 => write!(f, "jump0"),
            TType::If => write!(f, "if"),
            TType::Then => write!(f, "then"),
            TType::Else => write!(f, "else"),
            TType::Begin => write!(f, "begin"),
            TType::End => write!(f, "end"),
            TType::Loop => write!(f, "loop"),
            TType::Again => write!(f, "again"),
            TType::While => write!(f, "while"),
            TType::Random => write!(f, "random"),
            TType::Key => write!(f, "key"),
            TType::I => write!(f, "i"),
            TType::Buzzer => write!(f, "buzzer"),
            TType::Delay => write!(f, "delay"),
            TType::Colon => write!(f, "`:`"),
            TType::Semi => write!(f, "`;`"),
            TType::Minus => write!(f, "`-`"),
            TType::AssignEq => write!(f, "`=`"),
            TType::PlusEq => write!(f, "`+=`"),
            TType::MinusEq => write!(f, "`-=`"),
            TType::EqEq => write!(f, "`==`"),
            TType::NotEq => write!(f, "`!=`"),
            TType::Less => write!(f, "`<`"),
            TType::Greater => write!(f, "`>`"),
            TType::LessEq => write!(f, "`<=`"),
            TType::GreaterEq => write!(f, "`>=`"),
            TType::OrEq => write!(f, "`|=`"),
            TType::AndEq => write!(f, "`&=`"),
            TType::XorEq => write!(f, "`^=`"),
            TType::ShrEq => write!(f, "`>>=`"),
            TType::ShlEq => write!(f, "`<<=`"),
            TType::Eof => write!(f, "end of input"),
        }
    }
}
