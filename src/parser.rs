use crate::{
    ast::{
        AssignmentStmt, BinaryExpr, BlockStmt, CallStmt, ConditionalStmt,
        FnLiteral, Identifier, IntegerNode, Node, ReturnStmt,
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
    Pipe,
    EqNotEq,
    LtGt,
    AddSub,
    MulDiv,
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

    pub fn parse_stmt(
        &mut self,
        precedence: Precedence,
        terminator: TokenType,
        indent: usize,
    ) -> Result<Option<Box<dyn Node>>, ParseError> {
        let mut node: Option<Box<dyn Node>> = None;
        while self.r < self.n_tokens
            && !self.current_token_is(terminator)?
            && !self.current_token_is(TokenType::Eof)?
        {
            let tok = self.get_token();
            println!("[Indent {indent}] - {} {tok}", self.r);
            match tok.ttype {
                // TokenType::Pipe => {
                //     return Ok(Some(self.parse_pipe_expr()?));
                // }
                TokenType::Indent(_) => {
                    break;
                }
                TokenType::Add
                | TokenType::Sub
                | TokenType::Mul
                | TokenType::Div
                | TokenType::Eq
                | TokenType::Gt
                | TokenType::GtEq
                | TokenType::Pipe
                | TokenType::PipeMethod
                | TokenType::Lt
                | TokenType::LtEq => {
                    let new_precedence = Self::get_precedence(&tok.ttype);
                    if new_precedence <= precedence {
                        if node.is_none() {
                            node = self.get_operand_node()?;
                        }
                        return Ok(node);
                    }
                    if node.is_none() {
                        node = self.get_operand_node()?;
                    }
                    self.step();
                    node = Some(Self::get_binary_node(
                        tok.clone(),
                        node.unwrap(),
                        self.parse_stmt(new_precedence, terminator, indent)?
                            .expect("Unpack Binary"),
                    )?);
                }
                TokenType::LParen => {
                    self.incr_leading()?;
                    node = self.parse_stmt(
                        Precedence::Lowest,
                        terminator,
                        indent,
                    )?;
                }
                TokenType::RParen => {
                    if node.is_none() {
                        node = self.get_operand_node()?;
                    }
                    self.incr_leading()?;
                    return Ok(node);
                }
                TokenType::ReverseWalrus => {
                    let identifier: Identifier =
                        Identifier::new(self.tokens[self.l].clone());
                    self.step();
                    self.step();
                    let expr = self.parse_stmt(
                        Precedence::Lowest,
                        TokenType::Indent(indent),
                        indent,
                    )?;
                    return Ok(Some(Box::new(AssignmentStmt::new(
                        identifier,
                        expr.expect("Unwrap Assignment expr"),
                    ))));
                }
                TokenType::Assignment => {
                    let identifier: Identifier =
                        Identifier::new(self.tokens[self.l].clone());
                    self.step();
                    let expr = self.parse_stmt(
                        Precedence::Lowest,
                        // TokenType::Newline,
                        TokenType::Indent(indent),
                        indent,
                    )?;
                    return Ok(Some(Box::new(AssignmentStmt::new(
                        identifier,
                        expr.expect("Unwrap Assignment expr"),
                    ))));
                }
                TokenType::Def => {
                    // Get fn name
                    return Ok(Some(self.parse_fn(indent)?));
                }
                TokenType::Return => {
                    self.step();
                    return Ok(Some(Box::new(ReturnStmt::new(
                        self.parse_stmt(
                            Precedence::Lowest,
                            // TokenType::Newline,
                            TokenType::Indent(indent),
                            indent,
                        )?
                        .expect("Return Stmt"),
                    ))));
                }
                TokenType::If => {
                    return Ok(Some(self.parse_conditional_stmt(indent)?));
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
            node = self.get_operand_node()?;
        }
        return Ok(node);
    }

    fn parse_conditional_stmt(
        &mut self,
        indent: usize,
    ) -> Result<Box<ConditionalStmt>, ParseError> {
        // If -> Condition
        self.step();
        let cond = self.parse_stmt(Precedence::Lowest, TokenType::Colon, 0)?;
        // Condition suffix colon -> Indent
        self.step();

        let pass_block = self.parse_block(indent + 1)?;
        let mut fail_block: Option<Box<BlockStmt>> = None;
        if self.token_is_indent_of(indent)
            && self.peek_token_is(TokenType::Else)?
        {
            self.step();
            self.expect_peek(TokenType::Colon)?;
            self.step();
            self.step();
            fail_block = Some(self.parse_block(indent + 1)?);
        }
        return Ok(Box::new(ConditionalStmt::new(
            cond.expect("Unwrap conditional"),
            pass_block,
            fail_block,
        )));
    }

    fn token_is_indent_of(&self, indent: usize) -> bool {
        return match self.get_token().ttype {
            TokenType::Indent(indent_lvl) => indent_lvl == indent,
            _ => false,
        };
    }
    fn parse_call_args(&mut self) -> Result<Vec<Box<dyn Node>>, ParseError> {
        self.incr_leading()?;
        let mut args: Vec<Box<dyn Node>> = Vec::new();
        let arg = self.parse_stmt(Precedence::Lowest, TokenType::Comma, 0)?;
        if let Some(unwrap_arg) = arg {
            args.push(unwrap_arg);
        }

        while self.r < self.n_tokens
            && self.can_peek()
            && self.peek_token_is(TokenType::Comma)?
        {
            self.incr_leading()?;
            if let Some(arg) =
                self.parse_stmt(Precedence::Lowest, TokenType::Comma, 0)?
            {
                args.push(arg);
            }
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
        self.expect_peek(TokenType::Indent(indent + 1))?;

        self.step();
        let fn_body = Some(self.parse_block(indent + 1)?);
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
            }
            if let TokenType::Indent(new_indent) = *&tok.ttype {
                if new_indent < indent {
                    println!("{new_indent} <-- {indent}");
                    break;
                } else if new_indent > indent {
                    println!("{new_indent} --> {indent}");
                    stmts.push(self.parse_block(new_indent)?);
                } else {
                    self.step();
                    if let Some(stmt) = self.parse_stmt(
                        Precedence::Lowest,
                        TokenType::Indent(indent),
                        // TokenType::Newline,
                        indent,
                    )? {
                        stmts.push(stmt);
                    }
                }
            } else {
                println!("Here {}", self.r);
                return Err(ParseError::InvalidBlockStart(format!(
                    "{:?}",
                    self.get_token().ttype
                )));
            }
        }
        return Ok(stmts);
    }
    pub fn parse_block(
        &mut self,
        indent: usize,
    ) -> Result<Box<BlockStmt>, ParseError> {
        let tok = self.get_token();
        if let TokenType::Indent(ind_lvl) = *&tok.ttype {
            if ind_lvl != indent {
                return Err(ParseError::InvalidIndentLevel(format!(
                    "Expected {} - Found {}",
                    indent, ind_lvl
                )));
            }
            let stmts = self.parse_statements(ind_lvl)?;
            return Ok(Box::new(BlockStmt::new(indent, stmts)));
        }
        return Err(ParseError::InvalidBlockStart(format!(
            "Should be unreachable {} {}/{}",
            tok.val, self.r, self.n_tokens
        )));
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
            TokenType::Lt | TokenType::Gt => Precedence::LtGt,
            TokenType::Eq => Precedence::EqNotEq,
            TokenType::Pipe | TokenType::PipeMethod => Precedence::Pipe,
            _ => Precedence::Lowest,
        }
    }

    pub fn get_operand_node(
        &mut self,
    ) -> Result<Option<Box<dyn Node>>, ParseError> {
        while self.l < self.n_tokens
            && self.tokens[self.l].ttype == TokenType::LParen
        {
            self.incr_trailing();
        }
        let tok = self.tokens[self.l].clone();
        match tok.ttype {
            TokenType::Int => {
                return Ok(Some(Box::new(IntegerNode::new(tok)?)));
            }
            TokenType::Identifier => {
                return Ok(Some(Box::new(Identifier::new(tok))));
            }
            _ => {
                return Ok(None);
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
