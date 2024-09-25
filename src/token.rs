use crate::parser::Precedence;

pub type SpannedToken<'src> = (usize, Token<'src>);

#[derive(Debug, PartialEq)]
pub enum Token<'src> {
    // Have to give special treatment to sequences of spaces b/c
    Indent,
    Dedent,

    IntLiteral(i32),
    FloatLiteral(f32),
    StrLiteral(&'src str),
    Ident(&'src str),

    Newline,

    LParen,
    RParen,

    Add,
    Sub,
    Mul,
    Div,

    AddEq,
    SubEq,
    MulEq,
    DivEq,

    Bang,
    BangEq,

    Eq,
    DoubleEq,

    Not,
    And,
    Or,

    Arrow,
    Colon,
    Semicolon,
    Walrus,
    Comma,

    // Non-Operator Keywords
    Mut,
    If,
    Else,
    Return,
    Def,
    // Struct,
    // Enum,
}

impl Token<'_> {
    pub fn src_len(&self) -> usize {
        format!("{}", self).chars().count()
    }
}

impl std::fmt::Display for Token<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::IntLiteral(i) => return write!(f, "{}", i),
                Self::FloatLiteral(fl) => return write!(f, "{}", fl),
                Self::StrLiteral(s) => return write!(f, "{}", s),
                Self::Ident(id) => return write!(f, "{}", id),
                Self::Indent => return write!(f, "INDENT",),
                Self::Dedent => return write!(f, "DEDENT",),

                Self::LParen => "(",
                Self::RParen => ")",

                Self::Add => "+",
                Self::Sub => "-",
                Self::Mul => "*",
                Self::Div => "/",

                Self::AddEq => "+=",
                Self::SubEq => "-=",
                Self::MulEq => "*=",
                Self::DivEq => "/=",

                Self::Eq => "=",
                Self::DoubleEq => "==",
                Self::Bang => "!",
                Self::BangEq => "!=",

                Self::Not => "not",
                Self::And => "and",
                Self::Or => "or",

                Self::Newline => "\\n",

                Self::Comma => ",",
                Self::Arrow => "->",

                Self::Colon => ":",
                Self::Semicolon => ";",
                Self::Walrus => ":=",

                Self::Mut => "mut",
                Self::If => "If",
                Self::Else => "Else",
                Self::Return => "return",
                Self::Def => "def",
            }
        )
    }
}

impl Token<'_> {
    pub fn as_operator(&self) -> Option<Operator> {
        Some(match self {
            Self::Add => Operator::Add,
            Self::Sub => Operator::Sub,
            Self::Mul => Operator::Mul,
            Self::Div => Operator::Div,
            Self::DoubleEq => Operator::Equals,
            _ => return None,
        })
    }
}

#[derive(Debug, PartialEq)]
pub enum Operator {
    Add,
    Sub,
    Div,
    Mul,
    Equals,
}

impl Operator {
    pub fn precedence(&self) -> Precedence {
        match self {
            Self::Add | Self::Sub => Precedence::AddSub,
            Self::Mul | Self::Div => Precedence::MulDiv,
            Self::Equals => Precedence::Equality,
        }
    }
}
impl std::fmt::Display for Operator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Add => "+",
                Self::Sub => "-",
                Self::Mul => "*",
                Self::Div => "/",
                Self::Equals => "==",
            }
        )
    }
}
