use std::fs;

pub const WIDTH: usize = 64;
pub const HEIGHT: usize = 32;

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
pub struct Chip8 {
    /// 4096 bytes = 4 Kib
    memory: [u8; 4096],
    /// cpu registers (V0-VF)
    v: [u8; 16],
    /// index register (holds memory address)
    /// pointer to get `memory`
    i: u16,
    /// program counter 'u16'
    pc: usize,
    /// 64x32 display 'u8' but no bit packing
    pub gfx: [bool; WIDTH * HEIGHT],
    /// 60hz timers, game tick
    delay_timer: u8,
    /// play a sound, and then set the sound timer for how long the sound to play
    sound_timer: u8,
    /// separate stack because CHIP-8
    /// stores memory addresses like function recursion
    stack: [u16; 16],
    /// stack pointer 'u16'
    /// Essentially it's the length, where the *next* item will be stored
    sp: usize,
    /// store key(s) being pressed
    key: [u8; 16],
}

impl Chip8 {
    pub fn new() -> Self {
        Self {
            memory: [0; 4096],
            v: [0; 16],
            i: 0,
            pc: 0x200, // program counter starts at ROM
            gfx: [false; WIDTH * HEIGHT],
            delay_timer: 0,
            sound_timer: 0,
            stack: [0; 16],
            sp: 0,
            key: [0; 16],
        }
    }

    pub fn load_rom(&mut self, file: &str) {
        let bytes = self.read_bytes(file);
        let mut i = 0x200;
        if bytes.len() > self.memory.len() - 0x200 {
            panic!("ROM too large!");
        }
        for byte in bytes {
            self.memory[i] = byte;
            i += 1;
        }
    }

    fn read_bytes(&self, file: &str) -> Vec<u8> {
        fs::read(file).unwrap()
    }

    pub fn cycle(&mut self) {
        let opcode = self.next_opcode();

        // mask to only get the first nibble (4 bits)
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
                    let address = self.stack[self.sp];
                    self.sp -= 1;
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
            0x6000 => {
                // Set (6XNN)
                let x = self.x(opcode);
                let nn = self.nn(opcode);
                self.v[x as usize] = nn;
            }
            0x7000 => {
                // Add (7XNN)
                let x = self.x(opcode);
                let nn = self.nn(opcode);
                self.v[x] = self.v[x as usize].wrapping_add(nn);
            }
            0xA000 => {
                // Set Index (ANNN)
                let nnn = self.nnn(opcode);
                self.i = nnn;
            }
            0xD000 => {
                // Draw (DXYN)
                // draws N pixels tall sprite from memory location at i
                // at horizontal coordinate vX and vertical coordinate vY
                // recall sprites are always 8xN
                let vx = self.x(opcode);
                let vy = self.y(opcode);
                let n = self.n(opcode);

                let x = self.v[vx] % WIDTH as u8;
                let y = self.v[vy] % HEIGHT as u8;
                // note (vx % 64 == vx % 63)!
                // works only for powers of 2, 2^n and 2^n - 1...
                // (implicitly gets capped to 2^n so it works)

                // set VF (collision) to 0
                self.v[0xF] = 0;

                for dy in 0..n {
                    if y as u16 + dy >= 32 {
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

                            if pixel & self.gfx[gfx_i as usize] == true {
                                // same parity, so would switch off
                                self.v[0xF] = 1;
                            }

                            self.gfx[gfx_i as usize] ^= pixel;
                        }
                    }
                }
            }
            _ => panic!("Unknown opcode: {opcode} (pc={})", self.pc),
        }

        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        // todo sound timer
        // if self.sound_timer > 0 {
        // }
    }

    /// Consumes opcode
    fn next_opcode(&mut self) -> u16 {
        // Assume the following:
        // memory[pc]     == 0xA2 (8 bits)
        // memory[pc + 1] == 0xF0 (8 bits)
        // then we should return 0xA2F0
        let out = (self.memory[self.pc] as u16) << 8 | (self.memory[self.pc + 1] as u16);
        // read 2 bytes
        self.pc += 2;
        out
    }

    // bit operations
    fn nnn(&self, opcode: u16) -> u16 {
        opcode & 0x0FFF
    }

    fn nn(&self, opcode: u16) -> u8 {
        (opcode & 0x00FF) as u8
    }

    fn n(&self, opcode: u16) -> u16 {
        opcode & 0x000F
    }

    fn x(&self, opcode: u16) -> usize {
        ((opcode & 0x0F00) >> 8) as usize
    }

    fn y(&self, opcode: u16) -> usize {
        ((opcode & 0x00F0) >> 4) as usize
    }

    /// bit 0 corresponds to the most significant bit
    fn get_bit(&self, num: u8, bit: u8) -> bool {
        // remove all the insignificant bits
        // then mask with 00000001 (get the last digit)
        ((num >> (7 - bit)) & 1) == 1
    }
}
