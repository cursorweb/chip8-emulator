use std::{
    fs::{File, OpenOptions},
    io::{Read, Write},
    path::Path,
};

use tools::{asmparser::AsmParser, instr::Instr};

fn main() {
    let file = std::env::args().nth(1).expect("Usage: asm <file>");
    let mut source = String::new();
    File::open(file.clone())
        .unwrap()
        .read_to_string(&mut source)
        .unwrap();

    let x = AsmParser::new(&source);
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

    let labels = match Instr::gen_label_lookup(&tokens) {
        Ok(l) => l,
        Err(errs) => {
            for err in errs {
                println!("{err}");
            }
            return;
        }
    };

    let mut total_bytes = 0;

    for token in tokens {
        let bytes = token.to_bytes(&labels);
        total_bytes += bytes.len();
        output_file.write(&bytes).unwrap();
    }

    println!("Assembled {total_bytes}/{} bytes", 0x1000 - 0x200);
}
