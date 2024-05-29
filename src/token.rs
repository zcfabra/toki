use std::fmt::{format, Display};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
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
    Gt,
    GtEq,
    Lt,
    LtEq,

    Or,
    And,
    Dot,

    Int(String),
    Identifier(String),

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

    Bang,
    Assignment,
    Bar,
    For,
    Return,
    Def,
    Walrus,
    ReverseWalrus,
    Colon,

    Indent(usize),
    Eof,

    Newline,
    If,
    Else,
    In,
    Range,
    Struct,
    Protocol,
    Enum,

    Comma,
    Self_,
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Add => "+",
            Self::AddEq => "+=",
            Self::Sub => "-",
            Self::SubEq => "-=",
            Self::Mul => "*",
            Self::MulEq => "*=",
            Self::Div => "/",
            Self::DivEq => "/=",

            Self::Eq => "==",
            Self::NotEq => "!=",
            Self::Gt => ">",
            Self::GtEq => ">=",
            Self::Lt => "<",
            Self::LtEq => "<=",

            Self::Or => "or",
            Self::And => "and",
            Self::Dot => ".",

            Self::Int(i) =>i.as_str(),
            Self::Identifier(ident) => ident.as_str(),

            Self::LParen => "(",
            Self::RParen => ")",
            Self::LSquare => "[",
            Self::RSquare => "]",
            Self::LBrace => "{",
            Self::RBrace => "}",

            Self::Arrow => "->",

            Self::Pipe => "|>",
            Self::PipeMethod => "|.",
            Self::PipeErr => "|!",

            Self::Bang => "!",
            Self::Assignment => "=",
            Self::Bar => "|",
            Self::For => "for",
            Self::Return => "return",
            Self::Def => "def",
            Self::Walrus => ":=",
            Self::ReverseWalrus => "=:",
            Self::Colon => ":",

            Self::Indent(_) => ">>>>",
            Self::Eof => "[EOF]",

            Self::Newline => "\n",
            Self::If => "if",
            Self::Else => "else",
            Self::In => "in",
            Self::Range => "range",
            Self::Struct => "struct",
            Self::Protocol => "protocol",
            Self::Enum => "enum",

            Self::Comma => ",",
            Self::Self_ => "self",

        };
        write!(f, "{s}");
        return Ok(());
    }
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
                    while self.r < self.src_len && self.src[self.r] == ' ' {
                        self.r += 1;
                    }

                    let n_spaces = self.r - self.l;
                    if n_spaces >= 1 {
                        if n_spaces % 4 == 0 {
                            tokens.push(Token::Indent(n_spaces / 4));
                        }
                    }

                    self.l = self.r;
                }
                // Indent + Newlines
                '\n' => tokens.push(self.get_char_op(Token::Newline)),
                // Single Char Operators
                '(' => tokens.push(self.get_char_op(Token::LParen)),
                ')' => tokens.push(self.get_char_op(Token::RParen)),
                '[' => tokens.push(self.get_char_op(Token::LSquare)),
                ']' => tokens.push(self.get_char_op(Token::RSquare)),
                '{' => tokens.push(self.get_char_op(Token::RSquare)),
                '}' => tokens.push(self.get_char_op(Token::RSquare)),
                ',' => tokens.push(self.get_char_op(Token::Comma)),
                '.' => tokens.push(self.get_char_op(Token::Dot)),
                '<' => {
                    let token = if self.next_char_is('=') {
                        self.get_long_op(Token::LtEq)
                    } else {
                        self.get_char_op(Token::Lt)
                    };
                    tokens.push(token);
                }
                '>' => {
                    let token = if self.next_char_is('=') {
                        self.get_long_op(Token::GtEq)
                    } else {
                        self.get_char_op(Token::Gt)
                    };
                    tokens.push(token);
                }
                '+' => {
                    let token = if self.next_char_is('=') {
                        self.get_long_op(Token::AddEq)
                    } else {
                        self.get_char_op(Token::Add)
                    };
                    tokens.push(token);
                }
                '-' => {
                    let token = if self.next_char_is('=') {
                        self.get_long_op(Token::SubEq)
                    } else if self.next_char_is('>') {
                        self.get_long_op(Token::Arrow)
                    } else {
                        self.get_char_op(Token::Sub)
                    };
                    tokens.push(token);
                }
                '*' => {
                    let token = if self.next_char_is('=') {
                        self.get_long_op(Token::MulEq)
                    } else {
                        self.get_char_op(Token::Mul)
                    };
                    tokens.push(token);
                }
                '/' => {
                    let token = if self.next_char_is('=') {
                        self.get_long_op(Token::DivEq)
                    } else {
                        self.get_char_op(Token::Div)
                    };
                    tokens.push(token);
                }
                '=' => {
                    let token = match self.get_next_char() {
                        Some('=') => self.get_long_op(Token::Eq),
                        Some(':') => self.get_long_op(Token::ReverseWalrus),
                        _ => self.get_char_op(Token::Assignment),
                    };
                    tokens.push(token);
                }
                ':' => {
                    let token = if self.next_char_is('=') {
                        self.get_long_op(Token::Walrus)
                    } else {
                        self.get_char_op(Token::Colon)
                    };
                    tokens.push(token);
                }
                '|' => {
                    let token = match self.get_next_char() {
                        Some('>') => self.get_long_op(Token::Pipe),
                        Some('.') => self.get_long_op(Token::PipeMethod),
                        Some('!') => self.get_long_op(Token::PipeErr),
                        _ => self.get_char_op(Token::Bar),
                    };
                    tokens.push(token);
                }
                '!' => {
                    let token = if self.next_char_is('=') {
                        self.get_long_op(Token::NotEq)
                    } else {
                        self.get_char_op(Token::Bang)
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
        tokens.push(Token::Eof);
        return Ok(Self::make_nice_indents(tokens));
    }
    pub fn make_nice_indents(tokens: Vec<Token>) -> Vec<Token> {
        let mut nice_tokens: Vec<Token> = Vec::new();
        nice_tokens.push(Token::Indent(0));
        for (ix, tok) in tokens.iter().enumerate() {
            match tok {
                Token::Indent(_) => {
                    if ix + 1 < tokens.len() {
                        match tokens[ix + 1] {
                            Token::Pipe | Token::PipeMethod => {}
                            _ => {
                                nice_tokens.push(tok.clone());
                            }
                        }
                    }
                }
                Token::Newline => {
                    if ix + 1 < tokens.len() {
                        match tokens[ix + 1] {
                            Token::Pipe | Token::PipeMethod => {}
                            Token::Indent(_) => {}
                            _ => {
                                nice_tokens.push(Token::Indent(0));
                            }
                        }
                    }
                }
                _ => {
                    nice_tokens.push(tok.clone());
                }
            }
        }
        return nice_tokens;
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
        return Tokenizer::get_keyword(&literal)
            .unwrap_or(Token::Identifier(literal));
    }
    pub fn get_keyword(literal: &String) -> Option<Token> {
        let literal_str = literal.as_str();
        return match literal_str {
            "for" => Some(Token::For),
            "def" => Some(Token::Def),
            "or" => Some(Token::Or),
            "and" => Some(Token::And),
            "if" => Some(Token::If),
            "else" => Some(Token::Else),
            "in" => Some(Token::In),
            "range" => Some(Token::Range),
            "return" => Some(Token::Return),
            "struct" => Some(Token::Struct),
            "self" => Some(Token::Self_),
            "enum" => Some(Token::Enum),
            "protocol" => Some(Token::Protocol),
            _ => None,
        };
    }
    pub fn get_numerical_literal(&mut self) -> Token {
        while self.r < self.src_len && ('0'..='9').contains(&self.src[self.r])
        {
            self.r += 1;
        }
        let literal = self.src[self.l..self.r]
            .iter()
            .collect();
        self.l = self.r;
        return Token::Int(literal);
    }
    pub fn get_long_op(&mut self, tk: Token) -> Token {
        // Consume first char of operator
        self.r += 1;
        // Consume second char of operator
        self.r += 1;
        self.l = self.r;
        return tk;
    }
    pub fn get_char_op(&mut self, tk: Token) -> Token {
        self.r += 1;
        self.l = self.r;
        return tk;
    }
}
