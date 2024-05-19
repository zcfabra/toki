use crate::token::Token;
use crate::parser::ParseError;

pub trait Node {
    fn repr(&self) -> String;
    fn eval(self) -> Box<dyn Node>;
}

pub trait Expression {}

/*
Expressions:
1. Binary
2. Unary
3. Literal
4. Walrus?
5. Pipe
*/


pub struct IntegerNode {
    token: Token,
    value: i32,
}

impl IntegerNode {
    pub fn new(token: Token) -> Result<Self, ParseError> {
        let int_val: i32 = 
            token.val
            .parse::<i32>()
            .map_err(|_| ParseError::InvalidTypeData)?;
        return Ok(IntegerNode{
            value: int_val,
            token: token,
        });
    }

}

impl Node for IntegerNode {
    fn eval(self) -> Box<dyn Node> {
        return Box::new(self);
    }
    fn repr(&self) -> String {
        return self.value.to_string();
    }
}

pub struct BinaryExpr {
    op: Token,
    l: Box<dyn Node>,
    r: Box<dyn Node>,
}

impl BinaryExpr {
    pub fn new(op_token: Token, l: Box<dyn Node>, r: Box<dyn Node>) -> Self {
        return BinaryExpr{
            op: op_token,
            l: l,
            r: r,
        };
    }
}

impl Node for BinaryExpr {
    fn eval(self) -> Box<dyn Node> {
        return Box::new(self);
    }
    fn repr(&self) -> String {
        let l = self.l.as_ref().repr();
        let r = self.r.as_ref().repr();
        return format!("( {} {} {} )", l, self.op.val, r);
    }
}

/*
Statements:
1. Assignment
2. Block
3. If
4. Return
5. Switch
6. Fn declaration
*/
