// Interact with the OS.
// Based on the code generators in chris-morgan's rust-http

use std::io::{File, Open, Truncate, Read, Write};
use std::os;

mod mimegen;

fn main() {
    let args = os::args();
    match args.len() {
        3 => {
            let mime_list = Path::new(args.get(1).as_slice());
            let mime_mod = Path::new(args.get(2).as_slice());

            // Generate mimes
            ::mimegen::generate(mime_list, mime_mod).unwrap()
        },
        0 => {
            println!("usage: mimegen <input: mime list>.txt <output: mime module>.rs");
            os::set_exit_status(1);
        },
        _ => {
            println!("usage: {} <input: mime list>.txt <output: mime module>.rs", args.get(0));
            os::set_exit_status(1);
        }
    }
}

pub fn get_reader(path: Path) -> Box<Reader> {
  match File::open_mode(&path, Open, Read) {
    Ok(reader) => box reader as Box<Reader>,
    Err(e) => fail!("Unable to read file: {}", e.desc)
  }
}

pub fn get_writer(path: Path) -> Box<Writer> {
    match File::open_mode(&path, Truncate, Write) {
        Ok(writer) => box writer as Box<Writer>,
        Err(e) => fail!("Unable to write file: {}", e.desc)
    }
}
