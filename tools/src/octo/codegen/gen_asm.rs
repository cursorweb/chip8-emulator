use std::fs::File;
use std::io::Write;

use crate::instr::{Addr, Instr};

/// Helpers to organize the assembly from the codegen
pub fn org_asm(instrs: &Vec<Instr>, mut file: File) {
    let mut addr = 0x200u16;

    for (i, instr) in instrs.iter().enumerate() {
        if let Instr::Label(label) = &instr {
            if i > 0 {
                writeln!(file).unwrap();
            }
            writeln!(file, "{label}:").unwrap();
            continue;
        }

        let asm = match &instr {
            Instr::Cls => "cls".into(),
            Instr::Ret => "ret".into(),
            Instr::J(addr) => format!("j {}", show_addr(addr)),
            Instr::Call(addr) => format!("call {}", show_addr(addr)),
            Instr::Jri0(addr) => format!("jri0 0x{addr:03x}"),
            Instr::Jri(x, nn) => format!("jri ${x:x}, {nn}"),

            Instr::Seqi(x, nn) => format!("seqi ${x:x}, {nn}"),
            Instr::Snei(x, nn) => format!("snei ${x:x}, {nn}"),
            Instr::Seq(x, y) => format!("seq ${x:x}, ${y:x}"),
            Instr::Sne(x, y) => format!("sne ${x:x}, ${y:x}"),
            Instr::Sk(x) => format!("sk ${x:x}"),
            Instr::Snk(x) => format!("snk ${x:x}"),

            Instr::Seti(x, nn) => format!("seti ${x:x}, {nn}"),
            Instr::SetAddrI(addr) => format!("seti I, {}", show_addr(addr)),
            Instr::Addi(x, nn) => format!("addi ${x:x}, {nn}"),

            Instr::Set(x, y) => format!("set ${x:x}, ${y:x}"),
            Instr::Or(x, y) => format!("or ${x:x}, ${y:x}"),
            Instr::And(x, y) => format!("and ${x:x}, ${y:x}"),
            Instr::Xor(x, y) => format!("xor ${x:x}, ${y:x}"),
            Instr::Add(x, y) => format!("add ${x:x}, ${y:x}"),
            Instr::Subf(x, y) => format!("subf ${x:x}, ${y:x}"),
            Instr::Srlf(x, y) => format!("srlf ${x:x}, ${y:x}"),
            Instr::Subnf(x, y) => format!("subnf ${x:x}, ${y:x}"),
            Instr::Sllf(x, y) => format!("sllf ${x:x}, ${y:x}"),

            Instr::AddMemI(x) => format!("add I, ${x:x}"),

            Instr::Rand(x, nn) => format!("rand ${x:x}, {nn}"),
            Instr::Draw(x, y, n) => format!("draw ${x:x}, ${y:x}, {n}"),
            Instr::Bcd(x) => format!("bcd ${x:x}"),

            Instr::GetDelay(x) => format!("set ${x:x}, DELAY"),
            Instr::GetKey(x) => format!("set ${x:x}, KEY"),
            Instr::SetDelay(x) => format!("set DELAY, ${x:x}"),
            Instr::SetSound(x) => format!("set SOUND, ${x:x}"),

            Instr::SetFont(x) => format!("set I, FONT[${x:x}]"),
            Instr::SetMemI(x) => format!("set [I], ${x:x}"),
            Instr::GetMemI(x) => format!("set ${x:x}, [I]"),

            Instr::Word(word) => format!(".word 0x{word:04x}"),
            Instr::Byte(byte) => format!(".byte 0x{byte:02x}"),

            Instr::Label(_) => unreachable!(),
        };

        writeln!(file, "    {asm:<26} # 0x{addr:03x}").unwrap();

        addr += match instr {
            Instr::Byte(_) => 1,
            Instr::Word(_) => 2,
            _ => 2,
        };
    }
}

fn show_addr(addr: &Addr) -> String {
    match addr {
        Addr::Val(val) => format!("0x{val:03x}"),
        Addr::Label(label) => label.clone(),
    }
}
