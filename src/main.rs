pub mod ast;
pub mod lexer;
pub mod parser;
pub mod reporter;
pub mod token;

use crate::lexer::Lexer;
use crate::parser::parse;
use crate::reporter::report;

fn main() {
    let src = "10 + 90 * 20 + 10";
    match report(parse(Lexer::new(src)), src) {
        Ok(parsed) => println!("{}", parsed),
        Err(e) => println!("{}", e),
    }
}
