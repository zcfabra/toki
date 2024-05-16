pub struct Tokenizer {
    src: Vec<char>,
    l: usize,
    r: usize,
}
#[derive(Debug)]
pub enum TokenType {
    Add,
    AddEq,
    Sub,
    SubEq,
    Mul,
    MulEq,
    Div,
    DivEq,
    Eq,
    NotEq,
    Or,
    And,

    Int,
    Identifier,

    LParen,
    RParen,
    LSquare,
    RSquare,
    LBrace,
    RBrace,

    Arrow,

    Bang,
    Assignment,
    Pipe,
    Bar,
    For,
    Return,
    Def,
    Walrus,
    Colon,
    Indent,
    Newline,
    If,
    In, 
    Range,
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
                ' ' => {
                    self.r += 1;
                    self.l = self.r
                }
                // Indent + Newlines
                '\n' => tokens.push(self.get_char_op(ch, TokenType::Newline)),
                '\t' => tokens.push(self.get_char_op(ch, TokenType::Indent)),
                // Single Char Operators
                '(' => tokens.push(self.get_char_op(ch, TokenType::LParen)),
                ')' => tokens.push(self.get_char_op(ch, TokenType::RParen)),
                '[' => tokens.push(self.get_char_op(ch, TokenType::LSquare)),
                ']' => tokens.push(self.get_char_op(ch, TokenType::RSquare)),
                '{' => tokens.push(self.get_char_op(ch, TokenType::RSquare)),
                '}' => tokens.push(self.get_char_op(ch, TokenType::RSquare)),

                '+' => {
                     let token = if self.next_char_is('=') {
                        self.get_long_op(TokenType::AddEq)
                    } else {
                        self.get_char_op(ch, TokenType::Add)
                    };
                    tokens.push(token);
                },
                '-' => {
                     let token = if self.next_char_is('=') {
                        self.get_long_op(TokenType::SubEq)
                    } else if self.next_char_is('>') {
                        self.get_long_op(TokenType::Arrow)
                    }else {
                        self.get_char_op(ch, TokenType::Sub)
                    };
                    tokens.push(token);
                },
                '*' => {
                     let token = if self.next_char_is('=') {
                        self.get_long_op(TokenType::MulEq)
                    } else {
                        self.get_char_op(ch, TokenType::Mul)
                    };
                    tokens.push(token);
                },
                '/' => {
                     let token = if self.next_char_is('=') {
                        self.get_long_op(TokenType::DivEq)
                    } else {
                        self.get_char_op(ch, TokenType::Div)
                    };
                    tokens.push(token);
                },
                '=' => {
                    let token = if self.next_char_is('=') {
                        self.get_long_op(TokenType::Eq)
                    } else {
                        self.get_char_op(ch, TokenType::Assignment)
                    };
                    tokens.push(token);
                }
                ':' => {
                    let token = if self.next_char_is('=') {
                        self.get_long_op(TokenType::Walrus)
                    } else {
                        self.get_char_op(ch, TokenType::Colon)
                    };
                    tokens.push(token);
                }
                '|' => {
                    let token = if self.next_char_is('>') {
                        self.get_long_op(TokenType::Pipe)
                    } else {
                        self.get_char_op(ch, TokenType::Bar)
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
                'a'..='z' | 'A'..='Z' | '_' => {
                    tokens.push(self.get_alpha_literal());
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

    pub fn get_alpha_literal(&mut self) -> Token {
        while self.r < self.src.len() 
            && (
                ('a'..='z').contains(&self.src[self.r])
                || ('A'..='Z').contains(&self.src[self.r])
                || '_' == self.src[self.r]
            )
        {
            self.r += 1;
        }
        let literal = self.src[self.l..self.r].iter().collect();
        self.l = self.r;
        let token_type =
            Tokenizer::get_keyword(&literal).unwrap_or(TokenType::Identifier);
        return Token::new(literal, token_type);
    }
    pub fn get_keyword(literal: &String) -> Option<TokenType>{
        let literal_str= literal.as_str();
        return match literal_str {
            "for" => Some(TokenType::For),
            "def" => Some(TokenType::Def),
            "or" => Some(TokenType::Or),
            "and" => Some(TokenType::And),
            "if" => Some(TokenType::If),
            "in" => Some(TokenType::In),
            "range" => Some(TokenType::Range),
            "return" => Some(TokenType::Return),
            _ => None
        };
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
