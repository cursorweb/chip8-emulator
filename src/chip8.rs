fn chip8() {
    // opcode: u16 (2bytes)
    // memory is in bytes, u8

    // 4096 bytes = 4 Kib
    let memory = [0u8; 4096];
    /*
    Memory map:
    0x000-0x1FF - Chip 8 interpreter (contains font set in emu)
    0x050-0x0A0 - Used for the built in 4x5 pixel font set (0-F)
    0x200-0xFFF - Program ROM and work RAM

    Drawing is using XOR (so drawing twice erases)
    If drawing twice, then VF=1 (index 15), used for collisions
    */

    // cpu register (not cache)
    let v = [0u8; 16];

    // index register (hold memory address)
    let i = 0u16;

    // program counter
    let pc = 0u16;

    // 64x32 display
    let gfx = [0u8; 64 * 32];

    // 60hz timers, game tick
    let delay_timer = 0u8;
    // play a sound, and then set the sound timer for how long the sound to play
    let sound_timer = 0u8;

    // separate stack because CHIP-8
    // stores memory addresses like function recursion
    let stack = [0u16; 16];
    // stack pointer
    let sp = 0u16;

    // store key(s) being pressed
    let key = [0u8; 16];
}
