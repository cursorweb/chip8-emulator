use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
};

#[derive(Debug, Clone, Copy)]
struct Opcode {
    opcode: u16,
    pc: usize,
}

impl Opcode {
    fn new(opcode: u16, pc: usize) -> Self {
        Self { opcode, pc }
    }

    /// Print a word (here, we define as 2 bytes)
    fn format_word(&self) -> String {
        format!(".word 0x{:04x}", self.opcode)
    }

    /// Print an instruction, optionally print a word if invalid instr
    fn format_inst(&self, shift_y: bool, jump_bxnn: bool) -> String {
        let opcode = self.opcode;
        match opcode & 0xF000 {
            0x0000 => {
                if opcode == 0x00E0 {
                    "cls".into()
                } else if opcode == 0x00EE {
                    "ret".into()
                } else {
                    self.format_word()
                }
            }
            0x1000 => {
                // jump (1NNN)
                let address = self.nnn();
                format!("j 0x{address:x}")
            }
            0x2000 => {
                // call (2NNN) (jump but w/ a stack)
                let address = self.nnn();
                format!("call {address}")
            }
            0x3000 => {
                // Skip (3XNN) if V[X] == NN
                let x = self.x();
                let nn = self.nn();
                format!("seqi ${x:x}, {nn}")
            }
            0x4000 => {
                // Skip (4XNN) if V[X] != NN
                let x = self.x();
                let nn = self.nn();
                format!("snei ${x:x}, {nn}")
            }
            0x5000 => {
                // Skip (5XY0) if V[X] == V[Y]
                let (x, y) = self.xy();
                format!("seq ${x:x}, ${y:x}")
            }
            0x6000 => {
                // Set (6XNN)
                let x = self.x();
                let nn = self.nn();
                format!("seti ${x:x}, {nn}")
            }
            0x7000 => {
                // Add (7XNN)
                let x = self.x();
                let nn = self.nn();
                format!("addi ${x:x}, {nn}")
            }
            0x8000 => {
                let op = opcode & 0x000F;
                match op {
                    0x0 => {
                        // Set (8XY0) V[X] = V[Y]
                        let (x, y) = self.xy();
                        format!("set ${x:x}, ${y:x}")
                    }
                    0x1 => {
                        // OR (8XY1) V[X] = V[X] | V[Y]
                        let (x, y) = self.xy();
                        format!("or ${x:x}, ${y:x}")
                    }
                    0x2 => {
                        // AND (8XY2) V[X] = V[X] & V[Y]
                        let (x, y) = self.xy();
                        format!("and ${x:x}, ${y:x}")
                    }
                    0x3 => {
                        // XOR (8XY3) V[X] = V[X] ^ V[Y]
                        let (x, y) = self.xy();
                        format!("xor ${x:x}, ${y:x}")
                    }
                    0x4 => {
                        // ADD (8XY4) V[X] = V[X] + V[Y]
                        let (x, y) = self.xy();
                        format!("add ${x:x}, ${y:x}")
                    }
                    0x5 => {
                        // SUB (8XY5) V[X] = V[X] - V[Y]
                        // if VX >= VY, VF = 1 (no borrow)
                        // if VX < VY, VF = 0 (opposite of what overflow is! -- underflow)
                        let (x, y) = self.xy();
                        format!("subf ${x:x}, ${y:x}")
                    }
                    0x6 => {
                        // SHIFT (8XY6) V[X] >>= 1
                        let (x, y) = self.xy();
                        if shift_y {
                            format!("srlf ${x:x}, ${y:x}")
                        } else {
                            format!("srlf ${x:x}")
                        }
                    }
                    0x7 => {
                        // SUB (8XY7) V[X] = V[Y] - V[X]
                        let (x, y) = self.xy();
                        format!("subnf ${x:x}, ${y:x}")
                    }
                    0xE => {
                        // SHIFT (8XYE) V[X] <<= 1
                        let (x, y) = self.xy();
                        if shift_y {
                            format!("sllf ${x:x}, ${y:x}")
                        } else {
                            format!("sllf ${x:x}")
                        }
                    }
                    _ => self.format_word(),
                }
            }
            0x9000 => {
                // Skip (9XY0) if V[X] != V[Y]
                let (x, y) = self.xy();
                format!("sne ${x:x}, ${y:x}")
            }
            0xA000 => {
                // Set Index (ANNN)
                let nnn = self.nnn();
                format!("seti I, 0x{nnn:x}")
            }
            0xB000 => {
                // Jump with offset (BNNN)
                // Jump NNN + V0
                if jump_bxnn {
                    let nn = self.nn();
                    let x = self.x();
                    format!("jri ${x:x}, {nn}")
                } else {
                    let nnn = self.nnn();
                    format!("jri0 {nnn}")
                }
            }
            0xC000 => {
                // Rand (CXNN)
                let x = self.x();
                let nn = self.nn();
                format!("rand ${x:x}, {nn}")
            }
            0xD000 => {
                // Draw (DXYN)
                // draws N pixels tall sprite from memory location at i
                // at horizontal coordinate vX and vertical coordinate vY
                // recall sprites are always dimension 8 x N
                let (x, y) = self.xy();
                let n = self.n();

                format!("sprite ${x:x}, ${y:x}, {n}")
            }
            0xE000 => {
                let op = opcode & 0x00FF;
                match op {
                    0x9E => {
                        // Skip if key pressed (EX9E)
                        let x = self.x();
                        format!("sk ${x:x}")
                    }
                    0xA1 => {
                        // Skip if key NOT pressed (EXA1)
                        let x = self.x();
                        format!("snk ${x:x}")
                    }
                    _ => self.format_word(),
                }
            }
            0xF000 => {
                let op = opcode & 0x00FF;
                match op {
                    0x07 => {
                        // Set VX to delay timer (FX07)
                        let x = self.x();
                        format!("set ${x:x}, DELAY")
                    }
                    0x15 => {
                        // Set delay timer to VX (FX15)
                        let x = self.x();
                        format!("set DELAY, ${x:x}")
                    }
                    0x18 => {
                        // Set sound timer to VX (FX18)
                        let x = self.x();
                        format!("set SOUND, ${x:x}")
                    }
                    0x1E => {
                        // Index register += VX (FX1E)
                        let x = self.x();
                        format!("add I, ${x:x}")
                    }
                    0x0A => {
                        // Get key (FX0A)
                        let x = self.x();
                        format!("set ${x:x}, KEY")
                    }
                    0x29 => {
                        // Font character (FX29)
                        let x = self.x();
                        format!("set I, FONT[${x:x}]")
                    }
                    0x33 => {
                        // Binary-coded decimal conversion (FX33)
                        // eg if VX = 156, I = 1, I+1 = 5, I+2 = 6
                        let x = self.x();
                        format!("bcd ${x:x}")
                    }
                    0x55 => {
                        // Store Memory (FX55)
                        // Store V0 to VX inclusive to I, I+1, ... I+X
                        let x = self.x();
                        format!("set [I], ${x:x}")
                    }
                    0x65 => {
                        // Load Memory (FX65)
                        let x = self.x();
                        format!("set ${x:x}, [I]")
                    }
                    _ => self.format_word(),
                }
            }

            _ => self.format_word(),
        }
    }

    fn nnn(&self) -> u16 {
        self.opcode & 0x0FFF
    }

    fn nn(&self) -> u16 {
        self.opcode & 0x00FF
    }

    fn n(&self) -> u16 {
        self.opcode & 0x000F
    }

    /// 0x#X##
    fn x(&self) -> u16 {
        (self.opcode & 0x0F00) >> 8
    }

    /// 0x##Y#
    fn y(&self) -> u16 {
        (self.opcode & 0x00F0) >> 4
    }

    /// 0x#XY# -> (X, Y)
    fn xy(&self) -> (u16, u16) {
        (self.x(), self.y())
    }
}

/// 0x000-0x1FF - Chip 8 interpreter (contains font set in emu=emulator)
/// 0x050-0x0A0 - Used for the built in 4x5 pixel font set (0-F)
/// 0x200-0xFFF - Program ROM and work RAM
fn main() {
    let file = std::env::args().nth(1).expect("Usage: disasm <file>");
    let bytes = fs::read(file.clone()).expect("File not found");

    if bytes.len() > 0xFFF - 0x200 + 1 {
        panic!("ROM too large! Size: {}", bytes.len());
    }

    let lines: Vec<Opcode> = bytes
        .chunks(2)
        .enumerate()
        .map(|(i, chunk)| {
            if chunk.len() != 2 {
                panic!("Not aligned");
            }

            let pc = 0x200 + i * 2;
            let opcode = u16::from_be_bytes([chunk[0], chunk[1]]);

            Opcode::new(opcode, pc)
        })
        .collect::<Vec<Opcode>>()
        .into();

    let path = Path::new(&file)
        .file_stem()
        .map_or(file.clone(), |v| v.to_string_lossy().to_string());

    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(format!("{path}.s"))
        .unwrap();

    let mut show_word = false;

    for line in lines {
        let inst = if show_word {
            line.format_word()
        } else {
            let inst = line.format_inst(true, false);

            if inst.starts_with(".word") {
                show_word = true;
            }

            inst
        };

        file.write_all(format!("{:<30} # 0x{:x}\n", inst, line.pc).as_bytes())
            .unwrap();
    }
}
