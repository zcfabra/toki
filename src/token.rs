use std::fmt::Display;

#[derive(Debug, Clone, PartialEq, Eq)]
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
    Dot,

    Int,
    Identifier,

    LParen,
    RParen,
    LSquare,
    RSquare,
    LBrace,
    RBrace,

    Arrow,

    // Pipe Module
    Pipe,
    PipeMethod,
    PipeErr,
    PipeOk,
    PipeDebug,
    PipeBreak,
    PipeMatch,

    Bang,
    Assignment,
    Bar,
    For,
    Return,
    Def,
    Walrus,
    ReverseWalrus,
    Colon,
    Indent,
    Newline,
    If,
    In,
    Range,
    Struct,
    Protocol,
    Enum,

    Comma,
    Self_,
}
#[derive(Debug)]
pub enum TokenizerError {
    InvalidChar(char, usize),
}
pub struct Tokenizer {
    src: Vec<char>,
    src_len: usize,
    l: usize,
    r: usize,
}

impl Tokenizer {
    pub fn new(src: String) -> Self {
        let chars: Vec<char> = src.chars().collect();
        return Tokenizer {
            src: chars.clone(),
            src_len: chars.len(),
            l: 0,
            r: 0,
        };
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, TokenizerError> {
        let mut tokens: Vec<Token> = Vec::new();
        while self.r < self.src_len {
            let ch = self.src[self.r];
            match ch {
                ' ' => {
                    let mut space_count = 0;
                    while self.r < self.src_len && self.src[self.r] == ' ' {
                        if space_count == 4 {
                            space_count = 0;
                            let tab = self.src[self.l..self.r].iter().collect();
                            tokens.push(Token::new(tab, TokenType::Indent));
                            self.r += 1;
                            self.l = self.r;
                        } else {
                            space_count += 1;
                            self.r += 1;
                        }
                    }
                    self.l = self.r;
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
                ',' => tokens.push(self.get_char_op(ch, TokenType::Comma)),
                '.' => tokens.push(self.get_char_op(ch, TokenType::Dot)),

                '+' => {
                    let token = if self.next_char_is('=') {
                        self.get_long_op(TokenType::AddEq)
                    } else {
                        self.get_char_op(ch, TokenType::Add)
                    };
                    tokens.push(token);
                }
                '-' => {
                    let token = if self.next_char_is('=') {
                        self.get_long_op(TokenType::SubEq)
                    } else if self.next_char_is('>') {
                        self.get_long_op(TokenType::Arrow)
                    } else {
                        self.get_char_op(ch, TokenType::Sub)
                    };
                    tokens.push(token);
                }
                '*' => {
                    let token = if self.next_char_is('=') {
                        self.get_long_op(TokenType::MulEq)
                    } else {
                        self.get_char_op(ch, TokenType::Mul)
                    };
                    tokens.push(token);
                }
                '/' => {
                    let token = if self.next_char_is('=') {
                        self.get_long_op(TokenType::DivEq)
                    } else {
                        self.get_char_op(ch, TokenType::Div)
                    };
                    tokens.push(token);
                }
                '=' => {
                    let token = match self.get_next_char() {
                        Some('=') => self.get_long_op(TokenType::Eq),
                        Some(':') => self.get_long_op(TokenType::ReverseWalrus),
                        _ => self.get_char_op(ch, TokenType::Assignment),
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
                    let token = match self.get_next_char() {
                        Some('>') => self.get_long_op(TokenType::Pipe),
                        Some('.') => self.get_long_op(TokenType::PipeMethod),
                        Some('?') => self.get_long_op(TokenType::PipeDebug),
                        Some('*') => self.get_long_op(TokenType::PipeOk),
                        Some('!') => self.get_long_op(TokenType::PipeErr),
                        _ => self.get_char_op(ch, TokenType::Bar),
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
                    return Err(TokenizerError::InvalidChar(ch, self.r));
                }
            }
        }
        return Ok(tokens);
    }
    pub fn get_next_char(&self) -> Option<char> {
        if self.r + 1 < self.src_len {
            return Some(self.src[self.r + 1]);
        }
        return None;
    }
    pub fn next_char_is(&self, ch: char) -> bool {
        return self.r + 1 < self.src_len && self.src[self.r + 1] == ch;
    }

    pub fn get_alpha_literal(&mut self) -> Token {
        while self.r < self.src_len
            && (('a'..='z').contains(&self.src[self.r])
                || ('A'..='Z').contains(&self.src[self.r])
                || '_' == self.src[self.r])
        {
            self.r += 1;
        }
        let literal = self.src[self.l..self.r].iter().collect();
        self.l = self.r;
        let token_type = Tokenizer::get_keyword(&literal).unwrap_or(TokenType::Identifier);
        return Token::new(literal, token_type);
    }
    pub fn get_keyword(literal: &String) -> Option<TokenType> {
        let literal_str = literal.as_str();
        return match literal_str {
            "for" => Some(TokenType::For),
            "def" => Some(TokenType::Def),
            "or" => Some(TokenType::Or),
            "and" => Some(TokenType::And),
            "if" => Some(TokenType::If),
            "in" => Some(TokenType::In),
            "range" => Some(TokenType::Range),
            "return" => Some(TokenType::Return),
            "struct" => Some(TokenType::Struct),
            "self" => Some(TokenType::Self_),
            "enum" => Some(TokenType::Enum),
            "protocol" => Some(TokenType::Protocol),
            _ => None,
        };
    }
    pub fn get_numerical_literal(&mut self) -> Token {
        while self.r < self.src_len && ('0'..='9').contains(&self.src[self.r]) {
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

#[derive(Debug, Clone)]
pub struct Token {
    pub val: String,
    pub ttype: TokenType,
}

impl Token {
    pub fn new(literal: String, ttype: TokenType) -> Token {
        return Token {
            val: literal,
            ttype: ttype,
        };
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return f.write_fmt(format_args!("[ {:?}: {:?} ]", self.ttype, self.val));
    }
}
