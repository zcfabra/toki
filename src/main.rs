pub mod tokenizer;
use tokenizer::Tokenizer;

fn main() {
    let src = String::from("(10 != 10)");
    let tokens =
        Tokenizer::new(src)
        .tokenize()
        .expect("Failed to tokenize");
    println!("{:?}", tokens);
    // let ast = Parser::new(tokens).parse();
    // let result = Evaluator::new(ast).eval();

}
