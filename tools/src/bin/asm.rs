use std::{fs::File, io::Read};

use tools::parser::Parser;

fn main() {
    let file = std::env::args().nth(1).expect("Usage: asm <file>");
    let mut source = String::new();
    File::open(file)
        .unwrap()
        .read_to_string(&mut source)
        .unwrap();
    let x = Parser::new(source);
    let tokens = x.parse();
    println!("{:#?}", tokens);
}
