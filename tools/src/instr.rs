use std::collections::{HashMap, HashSet, hash_map::Entry::*};

#[derive(Debug, PartialEq, Clone)]
pub enum Instr {
    Cls,
    Ret,
    J(Addr),
    Call(Addr),
    /// Special unused
    Jri0(u16),
    /// jri $x, NN
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

    /// add I, $x
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
    fn lookup(&self, labels: &HashMap<String, u16>) -> u16 {
        match &self {
            Addr::Val(v) => *v,
            Addr::Label(s) => *labels.get(s).expect("labels should exist"),
        }
    }
}

impl Instr {
    pub fn to_bytes(&self, labels: &HashMap<String, u16>) -> Vec<u8> {
        match self {
            Instr::Cls => 0x00E0,
            Instr::Ret => 0x00EE,
            Instr::J(addr) => 0x1000 | addr.lookup(&labels),
            Instr::Call(addr) => 0x2000 | addr.lookup(&labels),
            Instr::Jri0(offset) => 0xB000 | offset,
            Instr::Jri(x, nn) => 0xB000 | self.x(x) | *nn as u16,
            Instr::Seqi(x, nn) => 0x3000 | self.x(x) | *nn as u16,
            Instr::Snei(x, nn) => 0x4000 | self.x(x) | *nn as u16,
            Instr::Seq(x, y) => 0x5000 | self.x(x) | self.y(y),
            Instr::Sne(x, y) => 0x9000 | self.x(x) | self.y(y),
            Instr::Sk(x) => 0xE09E | self.x(x),
            Instr::Snk(x) => 0xE0A1 | self.x(x),
            Instr::SetAddrI(addr) => 0xA000 | addr.lookup(&labels),
            Instr::Seti(x, nn) => 0x6000 | self.x(x) | *nn as u16,
            Instr::Addi(x, nn) => 0x7000 | self.x(x) | *nn as u16,
            Instr::Set(x, y) => 0x8000 | self.x(x) | self.y(y),
            Instr::Or(x, y) => 0x8001 | self.x(x) | self.y(y),
            Instr::And(x, y) => 0x8002 | self.x(x) | self.y(y),
            Instr::Xor(x, y) => 0x8003 | self.x(x) | self.y(y),
            Instr::Add(x, y) => 0x8004 | self.x(x) | self.y(y),
            Instr::Subf(x, y) => 0x8005 | self.x(x) | self.y(y),
            Instr::Srlf(x, y) => 0x8006 | self.x(x) | self.y(y),
            Instr::Subnf(x, y) => 0x8007 | self.x(x) | self.y(y),
            Instr::Sllf(x, y) => 0x800E | self.x(x) | self.y(y),
            Instr::AddMemI(x) => 0xF01E | self.x(x),
            Instr::Rand(x, nn) => 0xC000 | self.x(x) | *nn as u16,
            Instr::Sprite(x, y, n) => 0xD000 | self.x(x) | self.y(y) | *n as u16,
            Instr::Bcd(x) => 0xF033 | self.x(x),
            Instr::GetDelay(x) => 0xF007 | self.x(x),
            Instr::GetKey(x) => 0xF00A | self.x(x),
            Instr::SetDelay(x) => 0xF015 | self.x(x),
            Instr::SetSound(x) => 0xF018 | self.x(x),
            Instr::SetFont(x) => 0xF029 | self.x(x),
            Instr::SetMemI(x) => 0xF055 | self.x(x),
            Instr::GetMemI(x) => 0xF065 | self.x(x),
            Instr::Word(word) => *word,
            Instr::Byte(byte) => return vec![*byte as u8],
            Instr::Label(_) => return vec![],
        }
        .to_be_bytes()
        .to_vec()
    }

    /// #X##
    fn x(&self, x: &u8) -> u16 {
        (*x as u16) << 8
    }

    // ##Y#
    fn y(&self, y: &u8) -> u16 {
        (*y as u16) << 4
    }

    pub fn label_lookup(instrs: &[Instr]) -> Result<HashMap<String, u16>, Vec<String>> {
        let mut labels = HashMap::new();
        let mut referenced = HashSet::new();
        let mut addr = 0x200;
        let mut errors = vec![];

        for instr in instrs {
            if let Self::Label(label) = instr {
                match labels.entry(label.clone()) {
                    Occupied(_) => errors.push(format!("Duplicate label {label}")),
                    Vacant(e) => {
                        e.insert_entry(addr);
                    }
                }
            } else {
                match instr {
                    Instr::J(Addr::Label(s))
                    | Instr::Call(Addr::Label(s))
                    | Instr::SetAddrI(Addr::Label(s)) => {
                        referenced.insert(s);
                    }
                    _ => {}
                }
                addr += 2;
            }
        }

        for label in referenced {
            if !labels.contains_key(label) {
                errors.push(format!("Undefined label {label}"));
            }
        }

        if errors.len() == 0 {
            Ok(labels)
        } else {
            Err(errors)
        }
    }
}
