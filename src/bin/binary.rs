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

fn main() -> io::Result<()> {
    let file = std::env::args().next().expect("Usage: binary <file>");
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
