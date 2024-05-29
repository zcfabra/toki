use crate::{
    ast::{
        AssignmentStmt, BinaryExpr, BlockStmt, CallStmt, ConditionalStmt,
        FnLiteral, Identifier, IntegerNode, Node, ReturnStmt,
    },
    token::Token,
};

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
        terminator: Token,
        indent: usize,
    ) -> Result<Option<Box<dyn Node>>, ParseError> {
        let mut node: Option<Box<dyn Node>> = None;
        while self.r < self.n_tokens
            && !self.current_token_is(terminator.clone())?
            && !self.current_token_is(Token::Eof)?
        {
            let tok = self.get_token();
            println!("[Indent {indent}] - {} {tok}", self.r);
            match tok {
                // TokenType::Pipe => {
                //     return Ok(Some(self.parse_pipe_expr()?));
                // }
                Token::Indent(_) => {
                    break;
                }
                Token::Add
                | Token::Sub
                | Token::Mul
                | Token::Div
                | Token::Eq
                | Token::Gt
                | Token::GtEq
                | Token::Pipe
                | Token::PipeMethod
                | Token::Lt
                | Token::LtEq => {
                    let new_precedence = Self::get_precedence(&tok);
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
                        self.parse_stmt(
                            new_precedence,
                            terminator.clone(),
                            indent,
                        )?
                        .expect("Unpack Binary"),
                    )?);
                }
                Token::LParen => {
                    self.incr_leading()?;
                    node = self.parse_stmt(
                        Precedence::Lowest,
                        terminator.clone(),
                        indent,
                    )?;
                }
                Token::RParen => {
                    if node.is_none() {
                        node = self.get_operand_node()?;
                    }
                    self.incr_leading()?;
                    return Ok(node);
                }
                Token::ReverseWalrus => {
                    let identifier: Identifier =
                        Identifier::new(self.tokens[self.l].to_string());
                    self.step();
                    self.step();
                    let expr = self.parse_stmt(
                        Precedence::Lowest,
                        Token::Indent(indent),
                        indent,
                    )?;
                    return Ok(Some(Box::new(AssignmentStmt::new(
                        identifier,
                        expr.expect("Unwrap Assignment expr"),
                    ))));
                }
                Token::Assignment => {
                    let identifier: Identifier =
                        Identifier::new(self.tokens[self.l].to_string());
                    self.step();
                    let expr = self.parse_stmt(
                        Precedence::Lowest,
                        // TokenType::Newline,
                        Token::Indent(indent),
                        indent,
                    )?;
                    return Ok(Some(Box::new(AssignmentStmt::new(
                        identifier,
                        expr.expect("Unwrap Assignment expr"),
                    ))));
                }
                Token::Def => {
                    // Get fn name
                    return Ok(Some(self.parse_fn(indent)?));
                }
                Token::Return => {
                    self.step();
                    return Ok(Some(Box::new(ReturnStmt::new(
                        self.parse_stmt(
                            Precedence::Lowest,
                            // TokenType::Newline,
                            Token::Indent(indent),
                            indent,
                        )?
                        .expect("Return Stmt"),
                    ))));
                }
                Token::If => {
                    return Ok(Some(self.parse_conditional_stmt(indent)?));
                }
                Token::Identifier(ident) => {
                    if self.can_peek() && self.peek_token_is(Token::LParen)? {
                        self.step();
                        let args = self.parse_call_args()?;
                        node = Some(Box::new(CallStmt::new(
                            Identifier::new(ident),
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
        let cond = self.parse_stmt(Precedence::Lowest, Token::Colon, 0)?;
        // Condition suffix colon -> Indent
        self.step();

        let pass_block = self.parse_block(indent + 1)?;
        let mut fail_block: Option<Box<BlockStmt>> = None;
        if self.token_is_indent_of(indent)
            && self.peek_token_is(Token::Else)?
        {
            self.step();
            self.expect_peek(Token::Colon)?;
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
        return match self.get_token() {
            Token::Indent(indent_lvl) => indent_lvl == indent,
            _ => false,
        };
    }
    fn parse_call_args(&mut self) -> Result<Vec<Box<dyn Node>>, ParseError> {
        self.incr_leading()?;
        let mut args: Vec<Box<dyn Node>> = Vec::new();
        let arg = self.parse_stmt(Precedence::Lowest, Token::Comma, 0)?;
        if let Some(unwrap_arg) = arg {
            args.push(unwrap_arg);
        }

        while self.r < self.n_tokens
            && self.can_peek()
            && self.peek_token_is(Token::Comma)?
        {
            self.incr_leading()?;
            if let Some(arg) =
                self.parse_stmt(Precedence::Lowest, Token::Comma, 0)?
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
        self.step();
        if let Token::Identifier(name) = self.get_token() {
            let fn_name = Identifier::new(name);

            // Get args
            self.expect_peek(Token::LParen)?;
            self.step();

            let args = self.parse_args()?;
            self.step();
            self.expect_peek(Token::Colon)?;

            self.step();
            self.expect_peek(Token::Indent(indent + 1))?;

            self.step();
            let fn_body = Some(self.parse_block(indent + 1)?);
            return Ok(Box::new(FnLiteral::new(
                fn_name,
                args,
                fn_body.unwrap(),
            )));
        }
        return Err(ParseError::InvalidTypeData(format!(
            "Expected Identifier In Function Definition"
        )));
    }
    fn parse_statements(
        &mut self,
        indent: usize,
    ) -> Result<Vec<Box<dyn Node>>, ParseError> {
        let mut stmts: Vec<Box<dyn Node>> = Vec::new();
        while self.r < self.n_tokens {
            let tok = self.get_token();
            if tok == Token::Eof {
                break;
            }
            if let Token::Indent(new_indent) = *&tok {
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
                        Token::Indent(indent),
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
                    self.get_token()
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
        if let Token::Indent(ind_lvl) = *&tok {
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
            tok, self.r, self.n_tokens
        )));
    }

    fn parse_args(&mut self) -> Result<Vec<Identifier>, ParseError> {
        // TODO: Fix
        let mut args = Vec::new();
        let mut tok = self.get_token();
        while *&tok != Token::RParen {
            self.step();
            tok = self.get_token();
            args.push(Identifier::new(tok.to_string()));

            let next_tok =
                self.get_peek_token().ok_or(ParseError::ReachedEnd)?;

            if next_tok == Token::RParen {
                break;
            } else {
                self.expect_peek(Token::Comma)?;
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
    fn current_token_is(&self, tt: Token) -> Result<bool, ParseError> {
        if self.r >= self.n_tokens {
            return Err(ParseError::ReachedEnd);
        }
        return Ok(*&self.tokens[self.r] == tt);
    }
    fn can_peek(&self) -> bool {
        return self.r + 1 < self.n_tokens;
    }
    fn peek_token_is(&self, tt: Token) -> Result<bool, ParseError> {
        if self.r == self.n_tokens {
            return Err(ParseError::ReachedEnd);
        }
        return Ok(*&self.tokens[self.r + 1] == tt);
    }
    fn expect_peek(&self, tt: Token) -> Result<(), ParseError> {
        if self.r + 1 < self.n_tokens && *&self.tokens[self.r + 1] == tt {
            return Ok(());
        }
        return Err(ParseError::InvalidTokenOrder(format!(
            "Expected {:?} Found {:?}",
            tt,
            self.get_peek_token()
        )));
    }
    fn get_precedence(token_type: &Token) -> Precedence {
        match token_type {
            Token::Add | Token::Sub => Precedence::AddSub,
            Token::Mul | Token::Div => Precedence::MulDiv,
            Token::Lt | Token::Gt => Precedence::LtGt,
            Token::Eq => Precedence::EqNotEq,
            Token::Pipe | Token::PipeMethod => Precedence::Pipe,
            _ => Precedence::Lowest,
        }
    }

    pub fn get_operand_node(
        &mut self,
    ) -> Result<Option<Box<dyn Node>>, ParseError> {
        while self.l < self.n_tokens && self.tokens[self.l] == Token::LParen {
            self.incr_trailing();
        }
        let tok = self.tokens[self.l].clone();
        match tok {
            Token::Int(_) => {
                return Ok(Some(Box::new(IntegerNode::new(tok)?)));
            }
            Token::Identifier(i) => {
                return Ok(Some(Box::new(Identifier::new(i))));
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
