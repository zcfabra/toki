use crate::{
    ast::{
        AssignmentStmt, BinaryExpr, BlockStmt, CallStmt, FnLiteral,
        Identifier, IntegerNode, Node, ReturnStmt,
    },
    token::{self, TokenType},
};
use token::Token;

#[derive(Debug)]
pub enum ParseError {
    InvalidTypeData(String),
    InvalidIndentLevel(String),
    InvalidTokenOrder(String),
    NotImplementedToken(String),
    ReachedEnd,
    InvalidBlockStart(String),
    UnclosedParen,
}

pub struct Parser {
    tokens: Vec<Token>,
    n_tokens: usize,
    l: usize,
    r: usize,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub enum Precedence {
    Lowest,
    AddSub,
    MulDiv,
    EqNotEq,
    Call,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        return Parser {
            n_tokens: tokens.len(),
            tokens: tokens,
            l: 0,
            r: 0,
        };
    }
    fn step(&mut self) {
        self.r += 1;
        self.l = self.r;
    }
    fn incr_leading(&mut self) -> Result<(), ParseError> {
        if self.r == self.n_tokens {
            return Err(ParseError::ReachedEnd);
        }

        self.r += 1;
        return Ok(());
    }

    fn incr_trailing(&mut self) {
        self.l += 1;
    }

    pub fn parse(
        &mut self,
        precedence: Precedence,
        terminator: TokenType,
        indent: usize,
    ) -> Result<Box<dyn Node>, ParseError> {
        let mut node: Option<Box<dyn Node>> = None;
        while self.r < self.n_tokens
            && !self.current_token_is(terminator)?
            && !self.current_token_is(TokenType::Eof)?
        {
            let tok = self.get_token();
            match tok.ttype {
                TokenType::Add
                | TokenType::Sub
                | TokenType::Mul
                | TokenType::Div => {
                    let new_precedence = Self::get_precedence(&tok.ttype);
                    if new_precedence < precedence {
                        if node.is_none() {
                            node = Some(self.get_operand_node()?);
                        }
                        return Ok(node.unwrap());
                    }
                    if node.is_none() {
                        node = Some(self.get_operand_node()?);
                    }
                    self.step();
                    node = Some(Self::get_binary_node(
                        tok.clone(),
                        node.unwrap(),
                        self.parse(new_precedence, terminator, indent)?,
                    )?);
                }
                TokenType::LParen => {
                    self.incr_leading()?;
                    node = Some(self.parse(
                        Precedence::Lowest,
                        terminator,
                        indent,
                    )?);
                }
                TokenType::RParen => {
                    if node.is_none() {
                        node = Some(self.get_operand_node()?);
                    }
                    self.incr_leading()?;
                    return Ok(node.unwrap());
                }
                TokenType::Assignment => {
                    let identifier: Identifier =
                        Identifier::new(self.tokens[self.l].clone());
                    self.step();
                    let expr =
                        self.parse(Precedence::Lowest, terminator, indent)?;
                    return Ok(Box::new(AssignmentStmt::new(
                        identifier, expr,
                    )));
                }
                TokenType::Def => {
                    // Get fn name
                    return self.parse_fn(indent);
                }
                TokenType::Return => {
                    self.step();
                    return Ok(Box::new(ReturnStmt::new(self.parse(
                        Precedence::Lowest,
                        TokenType::Newline,
                        indent,
                    )?)));
                }
                TokenType::Identifier => {
                    if self.can_peek()
                        && self.peek_token_is(TokenType::LParen)?
                    {
                        self.step();
                        let args = self.parse_call_args()?;
                        node = Some(Box::new(CallStmt::new(
                            Identifier::new(tok.clone()),
                            args,
                        )));
                    } else {
                        self.incr_leading()?;
                    }
                }
                _ => {
                    self.incr_leading()?;
                }
            }
        }
        if node.is_none() {
            node = Some(self.get_operand_node()?);
        }
        return Ok(node.unwrap());
    }

    fn parse_call_args(&mut self) -> Result<Vec<Box<dyn Node>>, ParseError> {
        self.incr_leading()?;
        let mut args = Vec::new();
        args.push(self.parse(Precedence::Lowest, TokenType::Comma, 0)?);

        while self.r < self.n_tokens
            && self.can_peek()
            && self.peek_token_is(TokenType::Comma)?
        {
            self.incr_leading()?;
            let arg = self.parse(Precedence::Call, TokenType::Comma, 0)?;
            args.push(arg);
        }
        return Ok(args);
    }
    fn parse_fn(
        &mut self,
        indent: usize,
    ) -> Result<Box<dyn Node>, ParseError> {
        self.expect_peek(TokenType::Identifier)?;
        self.step();
        let fn_name = Identifier::new(self.get_token());

        // Get args
        self.expect_peek(TokenType::LParen)?;
        self.step();
        let args = self.parse_args()?;
        self.step();
        self.expect_peek(TokenType::Colon)?;
        self.step();
        self.expect_peek(TokenType::Newline)?;
        self.step();
        self.step();
        let fn_body = Some(self.parse_block(indent)?);
        return Ok(Box::new(FnLiteral::new(fn_name, args, fn_body.unwrap())));
    }
    fn parse_statements(
        &mut self,
        indent: usize,
    ) -> Result<Vec<Box<dyn Node>>, ParseError> {
        let mut stmts: Vec<Box<dyn Node>> = Vec::new();
        while self.r < self.n_tokens {
            let tok = self.get_token();
            if tok.ttype == TokenType::Eof {
                break;
            } else if let TokenType::Indent(new_indent) = *&tok.ttype {
                if new_indent < indent {
                    break;
                } else if new_indent > indent {
                    let block_stmt = self.parse_block(new_indent)?;
                    stmts.push(block_stmt);
                } else {
                    let stmt = self.parse(
                        Precedence::Lowest,
                        TokenType::Newline,
                        indent,
                    )?;
                    self.step();
                    stmts.push(stmt);
                }
            } else if *&tok.ttype == TokenType::Newline {
                if self.can_peek() {
                    let next_tok = self.get_peek_token().unwrap();
                    if let TokenType::Indent(lvl) = next_tok.ttype {
                        if lvl != indent {
                            break;
                        } else {
                            self.step();
                        }
                    } else if indent != 0 {
                        break;
                    }
                }
                self.step();
            } else if indent == 0 {
                let node = self.parse(
                    Precedence::Lowest,
                    TokenType::Newline,
                    indent,
                )?;
                self.step();
                stmts.push(node);
            } else {
                break;
            }
        }
        return Ok(stmts);
    }
    pub fn parse_block(
        &mut self,
        indent: usize,
    ) -> Result<Box<BlockStmt>, ParseError> {
        if indent == 0 {
            return Ok(Box::new(BlockStmt::new(
                indent,
                self.parse_statements(indent)?,
            )));
        } else {
            let tok = self.get_token();
            if let TokenType::Indent(ind_lvl) = *&tok.ttype {
                if ind_lvl != indent {
                    return Err(ParseError::InvalidIndentLevel(format!(
                        "Expected {} - Found {}",
                        indent, ind_lvl
                    )));
                }
                let stmts = self.parse_statements(indent)?;
                return Ok(Box::new(BlockStmt::new(indent, stmts)));
            }
            return Err(ParseError::InvalidBlockStart(
                "Should be unreachable".to_string(),
            ));
        }
    }

    fn parse_args(&mut self) -> Result<Vec<Identifier>, ParseError> {
        let mut args = Vec::new();
        let mut tok = self.get_token();
        while *&tok.ttype != TokenType::RParen {
            self.expect_peek(TokenType::Identifier)?;
            self.step();
            tok = self.get_token();
            args.push(Identifier::new(tok.clone()));

            let next_tok =
                self.get_peek_token().ok_or(ParseError::ReachedEnd)?;

            if next_tok.ttype == TokenType::RParen {
                break;
            } else {
                self.expect_peek(TokenType::Comma)?;
            }
            self.step();
        }
        return Ok(args);
    }
    fn get_token(&self) -> Token {
        return self.tokens[self.r].clone();
    }
    fn get_peek_token(&self) -> Option<Token> {
        if self.r + 1 < self.n_tokens {
            return Some(self.tokens[self.r + 1].clone());
        }
        return None;
    }
    fn current_token_is(&self, tt: TokenType) -> Result<bool, ParseError> {
        if self.r >= self.n_tokens {
            return Err(ParseError::ReachedEnd);
        }
        return Ok(*&self.tokens[self.r].ttype == tt);
    }
    fn can_peek(&self) -> bool {
        return self.r + 1 < self.n_tokens;
    }
    fn peek_token_is(&self, tt: TokenType) -> Result<bool, ParseError> {
        if self.r == self.n_tokens {
            return Err(ParseError::ReachedEnd);
        }
        return Ok(*&self.tokens[self.r + 1].ttype == tt);
    }
    fn expect_peek(&self, tt: TokenType) -> Result<(), ParseError> {
        if self.r + 1 < self.n_tokens && *&self.tokens[self.r + 1].ttype == tt
        {
            return Ok(());
        }
        return Err(ParseError::InvalidTokenOrder(format!(
            "Expected {:?} Found {:?}",
            tt,
            self.get_peek_token()
        )));
    }
    fn get_precedence(token_type: &TokenType) -> Precedence {
        match token_type {
            TokenType::Add | TokenType::Sub => Precedence::AddSub,
            TokenType::Mul | TokenType::Div => Precedence::MulDiv,
            _ => Precedence::Lowest,
        }
    }

    pub fn get_operand_node(&mut self) -> Result<Box<dyn Node>, ParseError> {
        while self.l < self.n_tokens
            && self.tokens[self.l].ttype == TokenType::LParen
        {
            self.incr_trailing();
        }
        let tok = self.tokens[self.l].clone();
        match tok.ttype {
            TokenType::Int => {
                return Ok(Box::new(IntegerNode::new(tok)?));
            }
            TokenType::Identifier => {
                return Ok(Box::new(Identifier::new(tok)))
            }
            _ => {
                return Err(ParseError::InvalidTypeData(format!(
                    "Invalid Operand Node `{}`",
                    tok.val
                )));
            }
        }
    }

    pub fn get_binary_node(
        op_token: Token,
        l: Box<dyn Node>,
        r: Box<dyn Node>,
    ) -> Result<Box<dyn Node>, ParseError> {
        return Ok(Box::new(BinaryExpr::new(op_token, l, r)));
    }
}
