use std::{fs::File, io::Read};

use tools::parser::Parser;

fn main() {
    let file = std::env::args().nth(1).expect("Usage: asm <file>");
    let mut source = String::new();
    File::open(file)
        .unwrap()
        .read_to_string(&mut source)
        .unwrap();

    let x = Parser::new(&source);
    match x.parse() {
        Ok(tokens) => println!("{:#?}", tokens),
        Err(e) => e.show(&source),
    }
}
