pub mod token;
pub mod parser;
pub mod ast;

use parser::{Parser, Precedence};
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
    let src = String::from("a = (1 * (1 - 10)) * 90");
    let tokens =
        Tokenizer::new(src)
        .tokenize()
        .expect("Failed to tokenize");
        
    let mut parser = Parser::new(tokens);
    let ast = parser.parse(Precedence::Lowest);
    match ast {
        Ok(val) => {
            println!("{:?}", val.as_ref().repr());
        },
        Err(e) => {
            println!("{:?}", e);
        }

    }
    // let result = Evaluator::new(ast).eval();

}
