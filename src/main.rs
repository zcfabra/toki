pub mod tokenizer;
use tokenizer::Tokenizer;

fn main() {
    let src =
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
    let tokens =
        Tokenizer::new(src)
        .tokenize()
        .expect("Failed to tokenize");
    for tok in tokens{
        println!("{:?}", tok);
    }
    // let ast = Parser::new(tokens).parse();
    // let result = Evaluator::new(ast).eval();

}
