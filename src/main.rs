pub mod ast;
pub mod lexer;
pub mod parser;
pub mod token;

use crate::lexer::Lexer;

fn main() {
    for tok in Lexer::new("a_var_name + a_var_name_2 + c + * 90") {
        match tok {
            Ok(t) => println!("{}", t),
            Err(e) => println!("{}", e),
        }
    }
}
