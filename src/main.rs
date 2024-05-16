pub mod tokenizer;
use tokenizer::Tokenizer;

fn main() {
    let src =
        String::from(
        "if x := 90:
            t: int = x + 10
            t += 10
            vals = [x for x in range(t + 10)]
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
