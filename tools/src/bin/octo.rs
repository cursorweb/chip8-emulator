use std::{
    fs::{File, OpenOptions},
    io::{Read, Write},
    path::Path,
    time::Instant,
};

use tools::octo::{CodeGen, Lexer, Parser, Resolver, org_asm};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    let emit_asm = args.contains(&"-s".to_string());
    let file = args
        .iter()
        .find(|arg| !arg.starts_with('-'))
        .expect("Usage: octo [-s] <file>");

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

    let resolver = Resolver::new(&trees);
    let data = match resolver.resolve() {
        Ok(data) => data,
        Err(e) => {
            e.show(&source);
            return;
        }
    };

    let codegen = CodeGen::new(trees, data);

    match codegen.gen_bytes() {
        Ok(instrs) => {
            let mut total_bytes = 0;

            if emit_asm {
                let asm_file = OpenOptions::new()
                    .create(true)
                    .write(true)
                    .truncate(true)
                    .open(format!("{}.s", output_path.to_str().unwrap()))
                    .unwrap();
                org_asm(&instrs, asm_file);
                println!("Emitted asm file ({} instrs)", instrs.len());
            }

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

    if emit_asm {}
}
