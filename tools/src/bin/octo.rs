use std::{
    fs::{File, OpenOptions},
    io::{Read, Write},
    path::Path,
    time::Instant,
};

use tools::octo::{CodeGen, Lexer, Parser};

fn main() {
    let file = std::env::args().nth(1).expect("Usage: asm <file>");
    let mut source = String::new();
    File::open(file.clone())
        .unwrap()
        .read_to_string(&mut source)
        .unwrap();

    let output_path = Path::new(&file).file_stem().unwrap();
    let mut output_file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(format!("{}.ch8", output_path.to_str().unwrap()))
        .unwrap();

    output_file.write(b"").unwrap();

    let start = Instant::now();
    let lexer = Lexer::new(&source);
    let out = lexer.lex();

    let tokens = match out {
        Ok(v) => v,
        Err(e) => {
            e.show(&source);
            return;
        }
    };

    let parser = Parser::new(tokens);
    let ast = parser.parse();

    let trees = match ast {
        Ok(v) => v,
        Err(e) => {
            e.show(&source);
            return;
        }
    };

    let codegen = CodeGen::new(trees);

    match codegen.gen_bytes() {
        Ok(instrs) => {
            let mut total_bytes = 0;

            for instr in instrs {
                // it's super easy to add labels...
                // and ship your own...
                // (todo)
                let bytes = instr.to_bytes(&std::collections::HashMap::new());
                total_bytes += bytes.len();
                output_file.write(&bytes).unwrap();
            }

            println!(
                "Assembled {total_bytes}/{} bytes in {:.2}secs",
                0x1000 - 0x200,
                start.elapsed().as_secs_f32()
            );
        }
        Err(e) => e.show(&source),
    }
}
