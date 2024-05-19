pub mod token;
pub mod parser;
pub mod ast;

use parser::{Parser, Precedence};
use crate::ast::Node;
use token::Tokenizer;

fn main() {
    let _ =
        String::from(
"struct Point:
    x: int
    y: int 
    def sum(self) -> int:
        return self.x + self.y

if x == 90:
    t: int = x + 10
    t += 10
    vals = [x for x in range(t + 10)]
else:
    x
    |> double
    |> print

point_sum =:
    Point(x=90, y=20)
    |.double
    |.square
    |.sum
    |? 


if good_val := could_return_none():
    print(good_val)
");
    let src = String::from(
"a = (1 * (1 - 10)) * 90
def square(a):
    return a * a
out = square(10) + square(10) 
"
    );
    let tokens =
        Tokenizer::new(src)
        .tokenize()
        .expect("Failed to tokenize");
        
    // for token in &tokens {
    //     println!("{}", token);
    // }
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_block(0);
    match ast {
        Ok(val) => {
            println!("{}", &val.repr());
        },
        Err(e) => {
            println!("{:?}", e);
        }

    }
    // let result = Evaluator::new(ast).eval();

}
