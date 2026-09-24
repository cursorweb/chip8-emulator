use crossterm::{
    cursor,
    event::{self, Event, KeyCode},
    execute,
    terminal::{self, ClearType},
};
use std::{
    fs,
    io::{self, Write},
};

/// opcode: u16 (2bytes)
/// memory is in bytes, u8
///
/// Memory map:
/// 0x000-0x1FF - Chip 8 interpreter (contains font set in emu=emulator)
/// 0x050-0x0A0 - Used for the built in 4x5 pixel font set (0-F)
/// 0x200-0xFFF - Program ROM and work RAM
///
/// Drawing is using XOR (so drawing twice erases)
/// If drawing twice, then VF=1 (index 15), used for collisions
fn main() -> io::Result<()> {
    let file = std::env::args().next().expect("Usage: disasm <file>");
    let bytes = fs::read(file).expect("File not found");
    let lines: Vec<String> = bytes
        .chunks(2)
        .enumerate()
        .filter_map(|(i, chunk)| {
            if chunk.len() != 2 {
                return None;
            }

            let pc = 0x200 + i * 2;
            let opcode = u16::from_be_bytes([chunk[0], chunk[1]]);

            Some(format!(
                "PC 0x{:03X} | {:02X} {:02X} | 0x{:04X}",
                pc, chunk[0], chunk[1], opcode
            ))
        })
        .collect();

    let mut stdout = io::stdout();

    terminal::enable_raw_mode()?;
    execute!(stdout, cursor::Hide)?;

    let (_, height) = terminal::size()?;
    let page_size = height.saturating_sub(1) as usize;

    let mut offset = 0;

    loop {
        execute!(
            stdout,
            terminal::Clear(ClearType::All),
            cursor::MoveTo(0, 0)
        )?;

        for line in lines.iter().skip(offset).take(page_size) {
            println!("{line}");
        }

        println!("\n↑↓ scroll | PgUp/PgDn | q quit");

        stdout.flush()?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => break,

                KeyCode::Up => {
                    offset = offset.saturating_sub(1);
                }

                KeyCode::Down => {
                    if offset + page_size < lines.len() {
                        offset += 1;
                    }
                }

                KeyCode::PageUp => {
                    offset = offset.saturating_sub(page_size);
                }

                KeyCode::PageDown => {
                    offset = (offset + page_size).min(lines.len().saturating_sub(page_size));
                }

                _ => {}
            }
        }
    }

    execute!(stdout, cursor::Show)?;
    terminal::disable_raw_mode()?;

    Ok(())
}

fn opcode_to_str(opcode: u8) -> String {
    match opcode & 0xF000 {
        0x0000 => {
            if opcode == 0x00E0 {
                // clear screen
                self.gfx = [false; WIDTH * HEIGHT];
            } else if opcode == 0x00EE {
                // return
                if self.sp == 0 {
                    panic!("Stack underflow (pc={})", self.pc);
                }

                // remember that sp points one AFTER the end
                // so sub 1 to get to end = top
                self.sp -= 1;
                let address = self.stack[self.sp];
                self.pc = address as usize;
            } else {
                panic!("Unknown opcode: {opcode} (pc={})", self.pc);
            }
        }
        0x1000 => {
            // jump (1NNN)
            let address = self.nnn(opcode);
            self.pc = address as usize;
        }
        0x2000 => {
            // call (2NNN) (jump but w/ a stack)
            let address = self.nnn(opcode);

            if self.sp == self.stack.len() {
                panic!("Stack overflow (pc={})", self.pc);
            }

            self.stack[self.sp] = self.pc as u16;
            self.sp += 1;
            self.pc = address as usize;
        }
        0x3000 => {
            // Skip (3XNN) if V[X] == NN
            let x = self.x(opcode);
            let nn = self.nn(opcode);
            let vx = self.v[x];
            if vx == nn {
                self.pc += 2; // skip 2 bytes
            }
        }
        0x4000 => {
            // Skip (4XNN) if V[X] != NN
            let x = self.x(opcode);
            let nn = self.nn(opcode);
            let vx = self.v[x];
            if vx != nn {
                self.pc += 2;
            }
        }
        0x5000 => {
            // Skip (5XY0) if V[X] == V[Y]
            let (x, y) = self.xy(opcode);
            if self.v[x] == self.v[y] {
                self.pc += 2;
            }
        }
        0x6000 => {
            // Set (6XNN)
            let x = self.x(opcode);
            let nn = self.nn(opcode);
            self.v[x] = nn;
        }
        0x7000 => {
            // Add (7XNN)
            let x = self.x(opcode);
            let nn = self.nn(opcode);
            self.v[x] = self.v[x].wrapping_add(nn);
        }
        0x8000 => {
            let op = opcode & 0x000F;
            match op {
                0x0 => {
                    // Set (8XY0) V[X] = V[Y]
                    let (x, y) = self.xy(opcode);
                    self.v[x] = self.v[y];
                }
                0x1 => {
                    // OR (8XY1) V[X] = V[X] | V[Y]
                    let (x, y) = self.xy(opcode);
                    self.v[x] = self.v[x] | self.v[y];
                    // quirk? idk
                    self.v[0xF] = 0;
                }
                0x2 => {
                    // AND (8XY2) V[X] = V[X] & V[Y]
                    let (x, y) = self.xy(opcode);
                    self.v[x] = self.v[x] & self.v[y];
                    self.v[0xF] = 0;
                }
                0x3 => {
                    // XOR (8XY3) V[X] = V[X] ^ V[Y]
                    let (x, y) = self.xy(opcode);
                    self.v[x] = self.v[x] ^ self.v[y];
                    self.v[0xF] = 0;
                }
                0x4 => {
                    // ADD (8XY4) V[X] = V[X] + V[Y]
                    let (x, y) = self.xy(opcode);
                    let (result, carry) = self.v[x].overflowing_add(self.v[y]);
                    self.v[x] = result;
                    self.v[0xF] = carry as u8;
                }
                0x5 => {
                    // SUB (8XY5) V[X] = V[X] - V[Y]
                    // if VX >= VY, VF = 1 (no borrow)
                    // if VX < VY, VF = 0 (opposite of what overflow is! -- underflow)
                    let (x, y) = self.xy(opcode);
                    let (result, borrow) = self.v[x].overflowing_sub(self.v[y]);
                    self.v[x] = result;
                    self.v[0xF] = !borrow as u8;
                }
                0x6 => {
                    // SHIFT (8XY6) V[X] >>= 1
                    let (x, y) = self.xy(opcode);
                    if self.shift_y {
                        self.v[x] = self.v[y];
                    }
                    self.v[0xF] = (self.v[x] & 1 == 1) as u8;
                    self.v[x] >>= 1;
                }
                0x7 => {
                    // SUB (8XY7) V[X] = V[Y] - V[X]
                    let (x, y) = self.xy(opcode);
                    let (result, borrow) = self.v[y].overflowing_sub(self.v[x]);
                    self.v[x] = result;
                    self.v[0xF] = !borrow as u8;
                }
                0xE => {
                    // SHIFT (8XYE) V[X] <<= 1
                    let (x, y) = self.xy(opcode);
                    if self.shift_y {
                        self.v[x] = self.v[y];
                    }
                    // VX & 0x80 is either 0x80 or 0x00 (0x8 is 1000)
                    self.v[0xF] = (self.v[x] & 0x80 != 0) as u8;
                    self.v[x] <<= 1;
                }
                _ => panic!("Unknown opcode: {opcode} (pc={})", self.pc),
            }
        }
        0x9000 => {
            // Skip (9XY0) if V[X] != V[Y]
            let x = self.x(opcode);
            let y = self.y(opcode);
            if self.v[x] != self.v[y] {
                self.pc += 2;
            }
        }
        0xA000 => {
            // Set Index (ANNN)
            let nnn = self.nnn(opcode);
            self.i = nnn;
        }
        0xB000 => {
            // Jump with offset (BNNN)
            // Jump NNN + V0
            if self.jump_bxnn {
                let nn = self.nn(opcode);
                let x = self.x(opcode);
                self.pc = nn as usize + self.v[x] as usize;
            } else {
                let nnn = self.nnn(opcode);
                let v0 = self.v[0];
                self.pc = nnn as usize + v0 as usize;
            }
        }
        0xC000 => {
            // Rand (CXNN)
            let x = self.x(opcode);
            let nn = self.nn(opcode);
            let num: u8 = self.rng.random();
            self.v[x] = num & nn;
        }
        0xD000 => {
            // Draw (DXYN)
            // draws N pixels tall sprite from memory location at i
            // at horizontal coordinate vX and vertical coordinate vY
            // recall sprites are always dimension 8 x N
            let vx = self.x(opcode);
            let vy = self.y(opcode);
            let n = self.n(opcode);

            let x = self.v[vx] % WIDTH as u8;
            let y = self.v[vy] % HEIGHT as u8;
            // note (vx % 64 == vx & 63)!
            // works only for powers of 2, 2^n and 2^n - 1...
            // (implicitly gets capped to 2^n so it works)

            // set VF (collision) to 0
            self.v[0xF] = 0;

            for dy in 0..n {
                if y as u16 + dy >= HEIGHT as u16 {
                    // clip the sprite
                    break;
                }

                // row of 8 pixels
                let row = self.memory[(self.i + dy) as usize];
                for dx in 0..8 {
                    if x + dx >= WIDTH as u8 {
                        break;
                    }

                    // xor each pixel that is 'on'
                    // if any pixels were turned off, set VF to 1
                    let pixel = self.get_bit(row, dx as u8);

                    if pixel == true {
                        let gfx_i = (y as u16 + dy) * WIDTH as u16 + (x + dx) as u16;

                        if self.gfx[gfx_i as usize] {
                            // same parity, so would switch off
                            self.v[0xF] = 1;
                        }

                        self.gfx[gfx_i as usize] ^= pixel;
                    }
                }
            }
        }
        0xE000 => {
            let op = opcode & 0x00FF;
            match op {
                0x9E => {
                    // Skip if key pressed (EX9E)
                    let x = self.x(opcode);
                    let vx = self.v[x];
                    if self.key[vx as usize] {
                        self.pc += 2;
                    }
                }
                0xA1 => {
                    // Skip if key NOT pressed (EXA1)
                    let x = self.x(opcode);
                    let vx = self.v[x];
                    if !self.key[vx as usize] {
                        self.pc += 2;
                    }
                }
                _ => panic!("Unknown opcode: {opcode} (pc={})", self.pc),
            }
        }
        0xF000 => {
            let op = opcode & 0x00FF;
            match op {
                0x07 => {
                    // Set VX to delay timer (FX07)
                    let x = self.x(opcode);
                    self.v[x] = self.delay_timer;
                }
                0x15 => {
                    // Set delay timer to VX (FX15)
                    let x = self.x(opcode);
                    self.delay_timer = self.v[x];
                }
                0x18 => {
                    // Set sound timer to VX (FX18)
                    let x = self.x(opcode);
                    self.sound_timer = self.v[x];
                }
                0x1E => {
                    // Index register += VX (FX1E)
                    let x = self.x(opcode);
                    self.i += self.v[x] as u16;
                    // 0x000 - 0x0FFF is valid memory, above this is an overflow
                    // of valid memory addresses
                    self.v[0xF] = (self.i > 0x0FFF) as u8;
                }
                0x0A => {
                    // Get key (FX0A)
                    let x = self.x(opcode);
                    let mut pressed = false;

                    for (i, &key) in self.key.iter().enumerate() {
                        if key {
                            self.v[x] = i as u8;
                            pressed = true;
                            break;
                        }
                    }

                    // loop
                    if !pressed {
                        self.pc -= 2;
                    }
                }
                0x29 => {
                    // Font character (FX29)
                    let x = self.x(opcode);
                    // only take the bottom nibble
                    let vx = self.v[x] & 0x0F;
                    let index = 0x050 + vx * 5;
                    self.i = index as u16;
                }
                0x33 => {
                    // Binary-coded decimal conversion (FX33)
                    // eg if VX = 156, I = 1, I+1 = 5, I+2 = 6
                    let x = self.x(opcode);
                    let mut vx = self.v[x];
                    for i in (0..3).rev() {
                        let digit = vx % 10;
                        vx /= 10;
                        self.memory[(self.i + i) as usize] = digit;
                    }
                }
                0x55 => {
                    // Store Memory (FX55)
                    // Store V0 to VX inclusive to I, I+1, ... I+X
                    let x = self.x(opcode);
                    for i in 0..=x {
                        let vi = self.v[i];
                        self.memory[self.i as usize + i] = vi;
                    }

                    if self.change_idx {
                        self.i += (x + 1) as u16;
                    }
                }
                0x65 => {
                    // Load Memory (FX65)
                    let x = self.x(opcode);
                    for i in 0..=x {
                        let mem = self.memory[self.i as usize + i];
                        self.v[i] = mem;
                    }

                    if self.change_idx {
                        self.i += (x + 1) as u16;
                    }
                }
                _ => panic!("Unknown opcode: {opcode} (pc={})", self.pc),
            }
        }

        _ => panic!("Unknown opcode: {opcode} (pc={})", self.pc),
    }
}
