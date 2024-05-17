trait Node {
    fn get_token() -> String;
    fn repr() -> String;
}

#[derive(Node)]
pub struct BinaryExpr{
    l: Box<impl Node>,
    r: Box<impl Node>
}

impl BinaryExpr {
    fn thing(&self) -> () {
        self.repr();
    }

}
