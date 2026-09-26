use std::{
    fs::{File, OpenOptions},
    io::{Read, Write},
    path::Path,
};

use tools::{instr::Instr, parser::Parser};

fn main() {
    let file = std::env::args().nth(1).expect("Usage: asm <file>");
    let mut source = String::new();
    File::open(file.clone())
        .unwrap()
        .read_to_string(&mut source)
        .unwrap();

    let x = Parser::new(&source);
    let tokens = match x.parse() {
        Ok(tokens) => tokens,
        Err(e) => return e.show(&source),
    };

    let output_path = Path::new(&file).file_stem().unwrap();
    let mut output_file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(format!("{}.ch8", output_path.to_str().unwrap()))
        .unwrap();

    let labels = match Instr::label_lookup(&tokens) {
        Ok(l) => l,
        Err(errs) => {
            for err in errs {
                println!("{err}");
            }
            return;
        }
    };

    for token in tokens {
        let bytes = token.to_bytes(&labels);
        output_file.write(&bytes).unwrap();
    }
}
