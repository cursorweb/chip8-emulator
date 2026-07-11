mod chip8;
use macroquad::prelude::*;

use crate::chip8::{Chip8, HEIGHT, WIDTH};

const SIZE: i32 = 20;

fn window_conf() -> Conf {
    Conf {
        window_title: "CHIP-8".to_string(),
        window_width: WIDTH as i32 * SIZE,
        window_height: HEIGHT as i32 * SIZE,
        window_resizable: false,
        ..Default::default()
    }
}

fn pixel(x: i32, y: i32) {
    draw_rectangle(
        (x * SIZE) as f32,
        (y * SIZE) as f32,
        SIZE as f32,
        SIZE as f32,
        Color::from_hex(0xFFFFFF),
    );
}

#[macroquad::main(window_conf)]
async fn main() {
    // shape: (16, 5)
    // note that the second half is UNUSED
    // let font_data: Vec<u8> = vec![
    //     0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    //     0x20, 0x60, 0x20, 0x20, 0x70, // 1
    //     0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    //     0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    //     0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    //     0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    //     0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    //     0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    //     0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    //     0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    //     0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    //     0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    //     0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    //     0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    //     0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    //     0xF0, 0x80, 0xF0, 0x80, 0x80, // F
    // ];

    let mut chip = Chip8::new();
    chip.load_rom("ibm.ch8");

    loop {
        clear_background(BLACK);

        chip.cycle();

        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                let i = y * WIDTH + x;
                if chip.gfx[i] {
                    pixel(x as i32, y as i32);
                }
            }
        }

        next_frame().await
    }

    // sprites are always 8xn!
    // fn print_sprite(font_data: &[u8], ox: i32, oy: i32, data_start: usize) {
    //     for y in 0..5 {
    //         for x in 0..8 {
    //             // leftmost bit is leftmost pixel
    //             // leftmost bit is also most significant bit
    //             let data = get_bit(font_data[data_start * 5 + y], 7 - x) == 1;
    //             pixel(ox * 5 + x as i32, oy * 6 + y as i32, data);
    //         }
    //     }
    // }
}
