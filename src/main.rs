pub mod ast;
pub mod parser;
pub mod token;

use crate::ast::Node;
use parser::Parser;
use token::Tokenizer;

fn main() {
    let _ = String::from(
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
",
    );
    let src = String::from(
"a = (1 * (1 - 10)) * 90
def square(a):
    return a * a
out = square(10) + square(square(20) * square(30))

if out > 10:
    print(out)
else:
    print(out + out)

if out < 20:
    x = (10 + 90) * 1000
else:
    x = square(square(900 * square(120)))

if out == 90:
    if out > 300:
        print(10)
        if out > 20:
            print(100)
        else:
            print(200)
else:
    print(20)
if 10 == 10:
    out = 900 + 900
1 + 2 + 3 + 4
|> print
|> double
|> print 
",
    );
    let tokens = Tokenizer::new(src).tokenize().expect("Failed to tokenize");

    for (ix, token) in tokens.iter().enumerate() {
        println!("{ix} {}", token);
    }
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_block(0);
    match ast {
        Ok(val) => {
            println!("{}", &val.repr());
        }
        Err(e) => {
            println!("{:?}", e);
        }
    }
    // let result = Evaluator::new(ast).eval();
}
