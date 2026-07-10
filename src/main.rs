// mod chip8;
use macroquad::prelude::*;

const WIDTH: i32 = 64;
const HEIGHT: i32 = 32;
const SIZE: i32 = 20;

fn window_conf() -> Conf {
    Conf {
        window_title: "CHIP-8".to_string(),
        window_width: WIDTH * SIZE,
        window_height: HEIGHT * SIZE,
        window_resizable: false,
        ..Default::default()
    }
}

fn pixel(x: i32, y: i32, white: bool) {
    draw_rectangle(
        (x * SIZE) as f32,
        (y * SIZE) as f32,
        SIZE as f32,
        SIZE as f32,
        if white {
            Color::from_hex(0xFFFFFF)
        } else {
            Color::from_hex(0)
        },
    );
}

fn get_bit<T: Into<u16>>(num: T, bit: u16) -> u8 {
    let num: u16 = num.into();
    // ((num / (1 << digit)) % 2) as u8
    // remove all the insignificant bits
    // then mask with 00000001 (get the last digit)
    ((num >> bit) & 1) as u8
}

#[macroquad::main(window_conf)]
async fn main() {
    // (16, 5)
    // note that the second half is UNUSED
    let font_data: Vec<u8> = vec![
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

    loop {
        clear_background(ORANGE);

        for y in 0..4 {
            for x in 0..4 {
                let i = y * 4 + x;
                print_sprite(&font_data, x, y, i as usize);
            }
        }

        next_frame().await
    }

    /// sprites are always 8xn!
    fn print_sprite(font_data: &[u8], ox: i32, oy: i32, data_start: usize) {
        for y in 0..5 {
            for x in 0..8 {
                // draw backwards order
                let data = get_bit(font_data[data_start * 5 + y], 7 - x) == 1;
                pixel(ox * 5 + x as i32, oy * 6 + y as i32, data);
            }
        }
    }
}
