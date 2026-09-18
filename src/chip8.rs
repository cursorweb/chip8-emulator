use macroquad::input::KeyCode;
use rand::prelude::*;
use std::fs;

pub const WIDTH: usize = 64;
pub const HEIGHT: usize = 32;
/// 10 cycles per frame = 10/f * 60 fps = 600 hz
pub const CYCLES_PER_FRAME: usize = 10;
/// ```txt
/// CHIP-8       Keyboard
/// 1 2 3 C      1 2 3 4
/// 4 5 6 D  ->  Q W E R
/// 7 8 9 E      A S D F
/// A 0 B F      Z X C V
/// ```
pub const KEYMAP: [(usize, KeyCode); 16] = [
    (0x1, KeyCode::Key1),
    (0x2, KeyCode::Key2),
    (0x3, KeyCode::Key3),
    (0xC, KeyCode::Key4),
    (0x4, KeyCode::Q),
    (0x5, KeyCode::W),
    (0x6, KeyCode::E),
    (0xD, KeyCode::R),
    (0x7, KeyCode::A),
    (0x8, KeyCode::S),
    (0x9, KeyCode::D),
    (0xE, KeyCode::F),
    (0xA, KeyCode::Z),
    (0x0, KeyCode::X),
    (0xB, KeyCode::C),
    (0xF, KeyCode::V),
];
const MEMORY_SIZE: usize = 4096;

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
    memory: [u8; MEMORY_SIZE],
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
    pub sound_timer: u8,
    /// separate stack because CHIP-8
    /// stores memory addresses like function recursion
    stack: [u16; 16],
    /// stack pointer 'u16'
    /// Essentially it's the length, where the *next* item will be stored
    sp: usize,
    /// store key(s) being pressed 'u8'
    key: [bool; 16],
    /// Whether or not 8XY6 and 8XYE uses VY
    shift_y: bool,
    /// Whether or not to treat jump with offset as BNNN or BXNN
    jump_bxnn: bool,
    /// Whether or not to change index as you set memory
    change_idx: bool,
    rng: ThreadRng,
}

impl Chip8 {
    pub fn new() -> Self {
        Self {
            memory: [0; MEMORY_SIZE],
            v: [0; 16],
            i: 0,
            pc: 0x200, // program counter starts at ROM
            gfx: [false; WIDTH * HEIGHT],
            delay_timer: 0,
            sound_timer: 0,
            stack: [0; 16],
            sp: 0,
            key: [false; 16],
            shift_y: true,
            jump_bxnn: false,
            change_idx: true,
            rng: rand::rng(),
        }
    }

    pub fn load_rom(&mut self, file: &str) {
        let bytes = self.read_bytes(file);
        let mut i = 0x200;
        if bytes.len() > self.memory.len() - 0x200 {
            panic!("ROM too large! Size: {}", bytes.len());
        }
        self.load_font();
        for byte in bytes {
            self.memory[i] = byte;
            i += 1;
        }
    }

    fn load_font(&mut self) {
        let font = [
            0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
            0x20, 0x60, 0x20, 0x20, 0x70, // 1
            0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
            0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
            0x90, 0x90, 0xF0, 0x10, 0x10, // 4
            0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
            0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
            0xF0, 0x10, 0x20, 0x40, 0x40, // 7
            0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
            0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
            0xF0, 0x90, 0xF0, 0x90, 0x90, // A
            0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
            0xF0, 0x80, 0x80, 0x80, 0xF0, // C
            0xE0, 0x90, 0x90, 0x90, 0xE0, // D
            0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
            0xF0, 0x80, 0xF0, 0x80, 0x80, // F
        ];

        for i in 0x050..=0x09F {
            self.memory[i] = font[i - 0x050];
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

    pub fn key_down(&mut self, k: usize) {
        self.key[k] = true;
    }

    pub fn key_up(&mut self, k: usize) {
        self.key[k] = false;
    }

    /// Call tick() every frame, delay_timer is display clock
    pub fn tick(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
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
    /// 0x#NNN
    fn nnn(&self, opcode: u16) -> u16 {
        opcode & 0x0FFF
    }

    /// 0x##NN
    fn nn(&self, opcode: u16) -> u8 {
        (opcode & 0x00FF) as u8
    }

    /// 0x###N
    fn n(&self, opcode: u16) -> u16 {
        opcode & 0x000F
    }

    /// 0x#X##
    fn x(&self, opcode: u16) -> usize {
        ((opcode & 0x0F00) >> 8) as usize
    }

    /// 0x##Y#
    fn y(&self, opcode: u16) -> usize {
        ((opcode & 0x00F0) >> 4) as usize
    }

    /// 0x#XY# -> (X, Y)
    fn xy(&self, opcode: u16) -> (usize, usize) {
        (self.x(opcode), self.y(opcode))
    }

    /// bit 0 corresponds to the most significant bit
    fn get_bit(&self, num: u8, bit: u8) -> bool {
        // remove all the insignificant bits
        // then mask with 00000001 (get the last digit)
        ((num >> (7 - bit)) & 1) == 1
    }
}
