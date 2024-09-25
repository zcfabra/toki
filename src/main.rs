pub mod ast;
pub mod lexer;
pub mod parser;
pub mod reporter;
pub mod token;

use std::fs::{File, OpenOptions};
use std::io::{Read, Result, Write};

use crate::lexer::Lexer;
use crate::parser::parse;
use crate::reporter::report;

fn main() {
    let mut args = std::env::args();
    args.next();

    let file = args.next().expect("Must Provide A File Name");
    let mut f = std::fs::File::open(&file).expect("Could Not Open File");
    let mut src = String::new();
    f.read_to_string(&mut src).expect("Couldn't Read String");

    for tok in Lexer::new(&src) {
        if tok.is_ok() {
            let (_, t) = tok.expect("");
            print!("[{}]", t);
        }
    }
    println!();

    match report(parse(Lexer::new(&src)), &src) {
        Ok(parsed) => {
            println!("{}", parsed);
            // let mut write_file = OpenOptions::new()
            //     .write(true)
            //     .truncate(true)
            //     .open(&file)
            //     .expect("Failed To Save");

            // write_file.write_all(format!("{}", parsed).as_bytes());
        }
        Err(e) => println!("{}", e),
    }
}
