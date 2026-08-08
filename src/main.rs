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

fn pixel(x: i32, y: i32, on: bool) {
    draw_rectangle(
        (x * SIZE) as f32 + 1.0,
        (y * SIZE) as f32 + 1.0,
        SIZE as f32 - 2.0,
        SIZE as f32 - 2.0,
        if on { WHITE } else { BLACK },
    );
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut chip = Chip8::new();
    chip.load_rom("ibm.ch8");

    loop {
        clear_background(Color::from_hex(0x111111));

        chip.cycle();

        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                let i = y * WIDTH + x;
                pixel(x as i32, y as i32, chip.gfx[i]);
            }
        }

        next_frame().await
    }
}
