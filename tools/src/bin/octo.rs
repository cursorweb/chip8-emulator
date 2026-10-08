use std::{
    fs::{File, OpenOptions},
    io::{Read, Write},
    path::Path,
};

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

    // for token in tokens {
    //     let bytes = token.to_bytes(&labels);
    //     output_file.write(&bytes).unwrap();
    // }
}
