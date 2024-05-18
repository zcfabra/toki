use crate::token::Token;

pub trait Node {
    fn repr(&self) -> &String;
    fn eval(&self) -> impl Node;
}

/*
Expressions:
1. Binary
2. Unary
3. Literal
4. Walrus?
5. Pipe
*/


/* 
Statements:
1. Assignment
2. Block 
3. If 
4. Return
5. Switch
*/
