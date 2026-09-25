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
                format!("seqi V{x}, {nn}")
            }
            0x4000 => {
                // Skip (4XNN) if V[X] != NN
                let x = self.x();
                let nn = self.nn();
                format!("snei V{x}, {nn}")
            }
            0x5000 => {
                // Skip (5XY0) if V[X] == V[Y]
                let (x, y) = self.xy();
                format!("seq V{x}, V{y}")
            }
            0x6000 => {
                // Set (6XNN)
                let x = self.x();
                let nn = self.nn();
                format!("ldi V{x}, {nn}")
            }
            0x7000 => {
                // Add (7XNN)
                let x = self.x();
                let nn = self.nn();
                format!("addi V{x}, {nn}")
            }
            0x8000 => {
                let op = opcode & 0x000F;
                match op {
                    0x0 => {
                        // Set (8XY0) V[X] = V[Y]
                        let (x, y) = self.xy();
                        format!("set V{x}, V{y}")
                    }
                    0x1 => {
                        // OR (8XY1) V[X] = V[X] | V[Y]
                        let (x, y) = self.xy();
                        format!("or V{x}, V{y}")
                    }
                    0x2 => {
                        // AND (8XY2) V[X] = V[X] & V[Y]
                        let (x, y) = self.xy();
                        format!("and V{x}, V{y}")
                    }
                    0x3 => {
                        // XOR (8XY3) V[X] = V[X] ^ V[Y]
                        let (x, y) = self.xy();
                        format!("xor V{x}, V{y}")
                    }
                    0x4 => {
                        // ADD (8XY4) V[X] = V[X] + V[Y]
                        let (x, y) = self.xy();
                        format!("add V{x}, V{y}")
                    }
                    0x5 => {
                        // SUB (8XY5) V[X] = V[X] - V[Y]
                        // if VX >= VY, VF = 1 (no borrow)
                        // if VX < VY, VF = 0 (opposite of what overflow is! -- underflow)
                        let (x, y) = self.xy();
                        format!("subf V{x}, V{y}")
                    }
                    0x6 => {
                        // SHIFT (8XY6) V[X] >>= 1
                        let (x, y) = self.xy();
                        if shift_y {
                            format!("srlf V{x}, V{y}")
                        } else {
                            format!("srlf V{x}")
                        }
                    }
                    0x7 => {
                        // SUB (8XY7) V[X] = V[Y] - V[X]
                        let (x, y) = self.xy();
                        format!("subnf V{x}, V{y}")
                    }
                    0xE => {
                        // SHIFT (8XYE) V[X] <<= 1
                        let (x, y) = self.xy();
                        if shift_y {
                            format!("sllf V{x}, V{y}")
                        } else {
                            format!("sllf V{x}")
                        }
                    }
                    _ => self.format_word(),
                }
            }
            0x9000 => {
                // Skip (9XY0) if V[X] != V[Y]
                let (x, y) = self.xy();
                format!("sne V{x}, V{y}")
            }
            0xA000 => {
                // Set Index (ANNN)
                let nnn = self.nnn();
                format!("ldi I, 0x{nnn:x}")
            }
            0xB000 => {
                // Jump with offset (BNNN)
                // Jump NNN + V0
                if jump_bxnn {
                    let nn = self.nn();
                    let x = self.x();
                    format!("jir V{x}, {nn}")
                } else {
                    let nnn = self.nnn();
                    format!("ji0 {nnn}")
                }
            }
            0xC000 => {
                // Rand (CXNN)
                let x = self.x();
                let nn = self.nn();
                format!("rand V{x}, {nn}")
            }
            0xD000 => {
                // Draw (DXYN)
                // draws N pixels tall sprite from memory location at i
                // at horizontal coordinate vX and vertical coordinate vY
                // recall sprites are always dimension 8 x N
                let (x, y) = self.xy();
                let n = self.n();

                format!("sprite V{x}, V{y}, {n}")
            }
            0xE000 => {
                let op = opcode & 0x00FF;
                match op {
                    0x9E => {
                        // Skip if key pressed (EX9E)
                        let x = self.x();
                        format!("sk V{x}")
                    }
                    0xA1 => {
                        // Skip if key NOT pressed (EXA1)
                        let x = self.x();
                        format!("snk V{x}")
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
                        format!("ld V{x}, delay")
                    }
                    0x15 => {
                        // Set delay timer to VX (FX15)
                        let x = self.x();
                        format!("ld delay, V{x}")
                    }
                    0x18 => {
                        // Set sound timer to VX (FX18)
                        let x = self.x();
                        format!("ld sound, V{x}")
                    }
                    0x1E => {
                        // Index register += VX (FX1E)
                        let x = self.x();
                        format!("add I, V{x}")
                    }
                    0x0A => {
                        // Get key (FX0A)
                        let x = self.x();
                        format!("ld V{x}, key")
                    }
                    0x29 => {
                        // Font character (FX29)
                        let x = self.x();
                        format!("ld I, font[V{x}]")
                    }
                    0x33 => {
                        // Binary-coded decimal conversion (FX33)
                        // eg if VX = 156, I = 1, I+1 = 5, I+2 = 6
                        let x = self.x();
                        format!("bcd V{x}")
                    }
                    0x55 => {
                        // Store Memory (FX55)
                        // Store V0 to VX inclusive to I, I+1, ... I+X
                        let x = self.x();
                        format!("sm I, V{x}")
                    }
                    0x65 => {
                        // Load Memory (FX65)
                        let x = self.x();
                        format!("lm I, V{x}")
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
    let file = std::env::args()
        .skip(1)
        .next()
        .expect("Usage: disasm <file>");
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
        let x = line.format_inst(true, false);

        if x.starts_with(".word") {
            show_word = true;
        }

        let inst = if show_word {
            line.format_word()
        } else {
            line.format_inst(true, false)
        };

        file.write_all(format!("{:<30} # 0x{:03x}\n", inst, line.pc).as_bytes())
            .unwrap();
    }
}
