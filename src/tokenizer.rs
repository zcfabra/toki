pub struct Tokenizer {
    src: Vec<char>,
    l: usize,
    r: usize,
}
#[derive(Debug)]
pub enum TokenType {
    Add,
    Sub,
    Mul,
    Div,
    Int,
    Identifier,
    LParen,
    RParen,
    Bang,
    Eq,
    NotEq,
    Assignment,
}
#[derive(Debug)]
pub enum TokenizerError {
    InvalidChar,
}
impl Tokenizer {
    pub fn new(src: String) -> Self {
        return Tokenizer {
            src: src.chars().collect(),
            l: 0,
            r: 0,
        };
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, TokenizerError> {
        let mut tokens: Vec<Token> = Vec::new();
        while self.r < self.src.len() {
            let ch = self.src[self.r];
            match ch {
                ' ' => {self.r += 1; self.l = self.r},
                '+' => tokens.push(self.get_char_op(ch, TokenType::Add)),
                '-' => tokens.push(self.get_char_op(ch, TokenType::Sub)),
                '*' => tokens.push(self.get_char_op(ch, TokenType::Mul)),
                '/' => tokens.push(self.get_char_op(ch, TokenType::Div)),
                '(' => tokens.push(self.get_char_op(ch, TokenType::LParen)),
                ')' => tokens.push(self.get_char_op(ch, TokenType::RParen)),
                '=' => {
                    let token = if self.next_char_is('=') {
                        self.get_long_op(TokenType::Eq)
                    } else {
                        self.get_char_op(ch, TokenType::Assignment)
                    };
                    tokens.push(token);
                }
                '!' => {
                    let token = if self.next_char_is('=') {
                        self.get_long_op(TokenType::NotEq)
                    } else {
                        self.get_char_op(ch, TokenType::Bang)
                    };
                    tokens.push(token);
                }
                '0'..='9' => {
                    tokens.push(self.get_numerical_literal());
                }
                _ => {
                    println!("{:?}", ch);
                    return Err(TokenizerError::InvalidChar);
                }
            }
        }
        return Ok(tokens);
    }
    pub fn next_char_is(&self, ch: char) -> bool {
        return self.r + 1 < self.src.len() && self.src[self.r + 1] == ch;
    }

    pub fn get_numerical_literal(&mut self) -> Token {
        while self.r < self.src.len() && ('0'..='9').contains(&self.src[self.r]) {
            self.r += 1;
        }
        let literal = self.src[self.l..self.r].iter().collect();
        self.l = self.r;
        return Token::new(literal, TokenType::Int);
    }
    pub fn get_long_op(&mut self, tt: TokenType) -> Token {
        let literal = self.src[self.r..=self.r + 1].iter().collect();
        // Consume first char of operator
        self.r += 1;
        // Consume second char of operator
        self.r += 1;
        self.l = self.r;
        return Token::new(literal, tt);
    }
    pub fn get_char_op(&mut self, ch: char, tt: TokenType) -> Token {
        self.r += 1;
        self.l = self.r;
        return Token::new(ch.to_string(), tt);
    }
}

#[derive(Debug)]
pub struct Token {
    val: String,
    ttype: TokenType,
}

impl Token {
    pub fn new(literal: String, ttype: TokenType) -> Token {
        return Token {
            val: literal,
            ttype: ttype,
        };
    }
}
